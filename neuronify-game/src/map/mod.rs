//! SVG scenario map parser.
//!
//! Parses a Neuronify RTS scenario SVG file into structured data.
//! Pure data — no ECS world mutation. Callers convert the result into entities.

pub mod hex;

use std::collections::HashMap;
use std::path::Path;

// ── Terrain ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerrainType {
    Open,
    EcmSparse,
    EcmDense,
    Csf,
    GlialScar,
    Vessel,
}

impl TerrainType {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "open"       => Some(TerrainType::Open),
            "ecm_sparse" => Some(TerrainType::EcmSparse),
            "ecm_dense"  => Some(TerrainType::EcmDense),
            "csf"        => Some(TerrainType::Csf),
            "glial_scar" => Some(TerrainType::GlialScar),
            "vessel"     => Some(TerrainType::Vessel),
            _            => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct HexTerrain {
    pub col: i32,
    pub row: i32,
    pub terrain: TerrainType,
    pub passable: bool,
    /// Movement speed multiplier. 1.0 for impassable terrain (unused).
    pub speed_mult: f32,
    /// True if this hex is a glucose resource source (vessel hexes only).
    pub resource_glucose: bool,
}

// ── Entities ──────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct NeuronDef {
    pub id: String,
    pub team: u32,
    pub neuron_type: NeuronDefType,
    pub x: f32,
    pub y: f32,
    pub col: i32,
    pub row: i32,
    pub auto_fire: bool,
    pub fire_hz: f32,
    pub is_origin: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NeuronDefType {
    Excitatory,
    Inhibitory,
}

#[derive(Clone, Debug)]
pub struct AxonDef {
    pub id: String,
    pub from: String,
    pub to: String,
    pub excitatory: bool,
}

#[derive(Clone, Debug)]
pub struct MacrophageDef {
    pub id: String,
    pub team: u32,
    pub x: f32,
    pub y: f32,
    pub driver_id: String,
    pub patrol_radius: f32,
    pub state: MacrophageState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MacrophageState {
    Active,
    Dormant,
}

#[derive(Clone, Debug)]
pub struct ToxinSpawnerDef {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub rate: f32,
}

// ── Scenario metadata ─────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ScenarioMeta {
    pub id: String,
    pub name: String,
    pub objective: String,
    pub victory: String,
    pub briefing: String,
}

// ── Top-level result ──────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct ScenarioMap {
    pub meta: ScenarioMeta,
    /// Terrain lookup by (col, row).
    pub terrain: HashMap<(i32, i32), HexTerrain>,
    pub neurons: Vec<NeuronDef>,
    pub axons: Vec<AxonDef>,
    pub macrophages: Vec<MacrophageDef>,
    pub toxin_spawners: Vec<ToxinSpawnerDef>,
}

impl ScenarioMap {
    pub fn grid_cols(&self) -> i32 {
        self.terrain.keys().map(|(c, _)| *c).max().unwrap_or(0) + 1
    }

    pub fn grid_rows(&self) -> i32 {
        self.terrain.keys().map(|(_, r)| *r).max().unwrap_or(0) + 1
    }

    pub fn terrain_counts(&self) -> HashMap<String, usize> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for hex in self.terrain.values() {
            let key = format!("{:?}", hex.terrain);
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
    }
}

// ── Parser ────────────────────────────────────────────────────────────────────

/// Load and parse a scenario SVG file.
pub fn load_scenario(path: &Path) -> Result<ScenarioMap, Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(path)?;
    parse_scenario_svg(&text)
}

/// Parse a scenario SVG from a string.
pub fn parse_scenario_svg(text: &str) -> Result<ScenarioMap, Box<dyn std::error::Error>> {
    let doc = roxmltree::Document::parse(text)?;

    let mut meta: Option<ScenarioMeta> = None;
    let mut terrain: HashMap<(i32, i32), HexTerrain> = HashMap::new();
    let mut neurons: Vec<NeuronDef> = Vec::new();
    let mut axons: Vec<AxonDef> = Vec::new();
    let mut macrophages: Vec<MacrophageDef> = Vec::new();
    let mut toxin_spawners: Vec<ToxinSpawnerDef> = Vec::new();

    for node in doc.descendants() {
        if !node.is_element() {
            continue;
        }

        match node.tag_name().name() {
            "g" => {
                // Scenario metadata
                if let Some(scenario_id) = node.attribute("data-scenario") {
                    meta = Some(ScenarioMeta {
                        id: scenario_id.to_string(),
                        name: node.attribute("data-name").unwrap_or("").to_string(),
                        objective: node.attribute("data-objective").unwrap_or("").to_string(),
                        victory: node.attribute("data-victory").unwrap_or("").to_string(),
                        briefing: node.attribute("data-briefing").unwrap_or("").to_string(),
                    });
                    continue;
                }

                // Entities
                match node.attribute("data-entity") {
                    Some("neuron") => {
                        if let Some(n) = parse_neuron(&node) {
                            neurons.push(n);
                        }
                    }
                    Some("axon") => {
                        if let Some(a) = parse_axon(&node) {
                            axons.push(a);
                        }
                    }
                    Some("macrophage") => {
                        if let Some(m) = parse_macrophage(&node) {
                            macrophages.push(m);
                        }
                    }
                    Some("toxin_spawner") => {
                        if let Some(t) = parse_toxin_spawner(&node) {
                            toxin_spawners.push(t);
                        }
                    }
                    _ => {}
                }
            }
            "polygon" => {
                if let Some(hex) = parse_hex_terrain(&node) {
                    terrain.insert((hex.col, hex.row), hex);
                }
            }
            _ => {}
        }
    }

    let meta = meta.ok_or("No <g data-scenario> element found in SVG")?;

    Ok(ScenarioMap { meta, terrain, neurons, axons, macrophages, toxin_spawners })
}

// ── Element parsers ───────────────────────────────────────────────────────────

fn parse_hex_terrain(node: &roxmltree::Node) -> Option<HexTerrain> {
    let terrain_str = node.attribute("data-terrain")?;
    let terrain = TerrainType::from_str(terrain_str)?;
    let col = node.attribute("data-col")?.parse().ok()?;
    let row = node.attribute("data-row")?.parse().ok()?;
    let passable = node.attribute("data-passable") == Some("true");
    let speed_mult = node.attribute("data-speed-mult")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1.0);
    let resource_glucose = node.attribute("data-resource") == Some("glucose");

    Some(HexTerrain { col, row, terrain, passable, speed_mult, resource_glucose })
}

fn parse_neuron(node: &roxmltree::Node) -> Option<NeuronDef> {
    let id = node.attribute("data-id")?.to_string();
    let team = node.attribute("data-team")?.parse().ok()?;
    let neuron_type = match node.attribute("data-type")? {
        "inhibitory" => NeuronDefType::Inhibitory,
        _            => NeuronDefType::Excitatory,
    };
    let x = node.attribute("data-x")?.parse().ok()?;
    let y = node.attribute("data-y")?.parse().ok()?;
    let col = node.attribute("data-col")?.parse().ok()?;
    let row = node.attribute("data-row")?.parse().ok()?;
    let auto_fire = node.attribute("data-auto-fire") == Some("true");
    let fire_hz = node.attribute("data-fire-hz")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    let is_origin = node.attribute("data-is-origin") == Some("true");

    Some(NeuronDef { id, team, neuron_type, x, y, col, row, auto_fire, fire_hz, is_origin })
}

fn parse_axon(node: &roxmltree::Node) -> Option<AxonDef> {
    let id = node.attribute("data-id")?.to_string();
    let from = node.attribute("data-from")?.to_string();
    let to = node.attribute("data-to")?.to_string();
    let excitatory = node.attribute("data-excitatory") != Some("false");

    Some(AxonDef { id, from, to, excitatory })
}

fn parse_macrophage(node: &roxmltree::Node) -> Option<MacrophageDef> {
    let id = node.attribute("data-id")?.to_string();
    let team = node.attribute("data-team")?.parse().ok()?;
    let x = node.attribute("data-x")?.parse().ok()?;
    let y = node.attribute("data-y")?.parse().ok()?;
    let driver_id = node.attribute("data-driver-id")?.to_string();
    let patrol_radius = node.attribute("data-patrol-radius")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100.0);
    let state = match node.attribute("data-state") {
        Some("active") => MacrophageState::Active,
        _              => MacrophageState::Dormant,
    };

    Some(MacrophageDef { id, team, x, y, driver_id, patrol_radius, state })
}

fn parse_toxin_spawner(node: &roxmltree::Node) -> Option<ToxinSpawnerDef> {
    let id = node.attribute("data-id")?.to_string();
    let x = node.attribute("data-x")?.parse().ok()?;
    let y = node.attribute("data-y")?.parse().ok()?;
    let rate = node.attribute("data-rate")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);

    Some(ToxinSpawnerDef { id, x, y, rate })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn load_test_map() -> ScenarioMap {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("maps/excitotoxic-wave.svg");
        load_scenario(&path).expect("failed to load scenario SVG")
    }

    #[test]
    fn test_scenario_metadata() {
        let map = load_test_map();
        assert_eq!(map.meta.id, "excitotoxic_wave");
        assert_eq!(map.meta.name, "Excitotoxic Wave");
        assert!(!map.meta.objective.is_empty());
        assert_eq!(map.meta.victory, "all_neurons_dead:e1,e2,e3");
        assert!(!map.meta.briefing.is_empty());
    }

    #[test]
    fn test_grid_dimensions() {
        let map = load_test_map();
        assert_eq!(map.grid_cols(), 22, "expected 22 columns");
        assert_eq!(map.grid_rows(), 18, "expected 18 rows");
    }

    #[test]
    fn test_terrain_counts() {
        let map = load_test_map();
        let counts = map.terrain_counts();
        let open = *counts.get("Open").unwrap_or(&0);
        // Expect roughly 194 open hexes (allow ±10 for map edits)
        assert!(open >= 184 && open <= 204,
            "expected ~194 open hexes, got {}", open);
    }

    #[test]
    fn test_entity_counts() {
        let map = load_test_map();
        assert_eq!(map.neurons.len(), 16, "expected 16 neurons");
        assert_eq!(map.axons.len(), 18, "expected 18 axons");
        assert_eq!(map.macrophages.len(), 2, "expected 2 macrophages");
        assert_eq!(map.toxin_spawners.len(), 1, "expected 1 toxin spawner");
    }

    #[test]
    fn test_neuron_origin() {
        let map = load_test_map();
        let origin = map.neurons.iter().find(|n| n.is_origin).expect("no origin neuron");
        assert_eq!(origin.id, "p_origin");
        assert_eq!(origin.team, 1);
        assert_eq!(origin.col, 3);
        assert_eq!(origin.row, 15);
        assert!(origin.auto_fire);
    }

    #[test]
    fn test_axon_references_valid_neurons() {
        let map = load_test_map();
        let neuron_ids: std::collections::HashSet<&str> =
            map.neurons.iter().map(|n| n.id.as_str()).collect();
        for axon in &map.axons {
            assert!(
                neuron_ids.contains(axon.from.as_str()),
                "axon {} has unknown from={}", axon.id, axon.from
            );
            // `to` may reference a macrophage (driver axons) — check neurons + macrophages
            let macro_ids: std::collections::HashSet<&str> =
                map.macrophages.iter().map(|m| m.id.as_str()).collect();
            assert!(
                neuron_ids.contains(axon.to.as_str()) || macro_ids.contains(axon.to.as_str()),
                "axon {} has unknown to={}", axon.id, axon.to
            );
        }
    }

    #[test]
    fn test_vessel_hexes_have_glucose() {
        let map = load_test_map();
        for hex in map.terrain.values() {
            if hex.terrain == TerrainType::Vessel {
                assert!(hex.resource_glucose, "vessel hex ({},{}) missing data-resource=glucose", hex.col, hex.row);
                assert!(!hex.passable, "vessel hex ({},{}) should be impassable", hex.col, hex.row);
            }
        }
    }

    #[test]
    fn test_passable_hexes_have_speed_mult() {
        let map = load_test_map();
        for hex in map.terrain.values() {
            if hex.passable {
                assert!(hex.speed_mult > 0.0 && hex.speed_mult <= 1.0,
                    "passable hex ({},{}) has invalid speed_mult {}", hex.col, hex.row, hex.speed_mult);
            }
        }
    }

    #[test]
    fn test_print_summary() {
        let map = load_test_map();
        let counts = map.terrain_counts();
        println!("Scenario: {} ({})", map.meta.name, map.meta.id);
        println!("Grid: {}×{}", map.grid_cols(), map.grid_rows());
        println!("Terrain counts:");
        let mut sorted: Vec<_> = counts.iter().collect();
        sorted.sort_by_key(|(k, _)| k.as_str());
        for (t, n) in sorted {
            println!("  {:12} {}", t, n);
        }
        println!("Neurons:        {}", map.neurons.len());
        println!("Axons:          {}", map.axons.len());
        println!("Macrophages:    {}", map.macrophages.len());
        println!("Toxin spawners: {}", map.toxin_spawners.len());
        for n in &map.neurons {
            println!("  neuron {:12} team={} type={:?} col={} row={} auto_fire={} origin={}",
                n.id, n.team, n.neuron_type, n.col, n.row, n.auto_fire, n.is_origin);
        }
    }
}
