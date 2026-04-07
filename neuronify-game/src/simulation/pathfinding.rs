use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;

use hecs::Entity;

use crate::map::{HexTerrain, TerrainType};

/// Axial hex coordinate.
pub type HexCoord = (i32, i32);

// ── Terrain helpers ───────────────────────────────────────────────────────────

/// Integer step cost (×10 base) for moving through a terrain cell.
/// Returns `None` if the terrain is impassable.
pub fn terrain_step_cost(terrain: &HexTerrain) -> Option<i32> {
    if !terrain.passable {
        return None;
    }
    // Base cost = 10; scaled by 1/speed_mult so slow terrain costs more.
    let cost = (10.0 / terrain.speed_mult.max(0.01)).round() as i32;
    Some(cost)
}

/// Axon-growth speed multiplier derived from terrain type.
/// Returns 0.0 for impassable terrain.
pub fn axon_speed_mult(terrain: &HexTerrain) -> f32 {
    if !terrain.passable {
        return 0.0;
    }
    match terrain.terrain {
        TerrainType::Open      => 1.0,
        TerrainType::EcmSparse => 0.7,
        TerrainType::EcmDense  => 0.3,
        _                      => 0.0, // should not be passable, but be safe
    }
}

/// Look up the terrain at a world-space position.
pub fn terrain_at(
    pos: glam::Vec3,
    terrain: &HashMap<(i32, i32), HexTerrain>,
) -> Option<&HexTerrain> {
    let h = crate::map::hex::world_to_hex(pos.x, pos.z);
    terrain.get(&(h.col, h.row))
}

/// Sparse infinite hex grid used only for pathfinding / occupancy.
/// All world positions are mapped onto flat-top hex cells.
pub struct HexGrid {
    /// World units spanned by one hex cell (centre to centre on the q-axis).
    pub cell_size: f32,
    /// Current occupants: at most one entity per cell.
    pub occupied: HashMap<HexCoord, Entity>,
}

impl HexGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            occupied: HashMap::new(),
        }
    }

    /// Convert a world-space position to the nearest hex coordinate.
    pub fn world_to_hex(&self, pos: glam::Vec3) -> HexCoord {
        // Flat-top axial hex: q points along +x, r along +z
        let size = self.cell_size;
        let q_frac = (2.0 / 3.0 * pos.x) / size;
        let r_frac = (-1.0 / 3.0 * pos.x + (3.0_f32).sqrt() / 3.0 * pos.z) / size;
        round_hex(q_frac, r_frac)
    }

    /// Convert a hex coordinate to its world-space centre (y = 0).
    pub fn hex_to_world(&self, h: HexCoord) -> glam::Vec3 {
        let size = self.cell_size;
        let (q, r) = h;
        let x = size * (3.0 / 2.0) * q as f32;
        let z = size * ((3.0_f32).sqrt() / 2.0 * q as f32 + (3.0_f32).sqrt() * r as f32);
        glam::Vec3::new(x, 0.0, z)
    }

    /// The 6 axial neighbours of a hex cell.
    pub fn neighbours(h: HexCoord) -> [HexCoord; 6] {
        let (q, r) = h;
        [
            (q + 1, r),
            (q + 1, r - 1),
            (q, r - 1),
            (q - 1, r),
            (q - 1, r + 1),
            (q, r + 1),
        ]
    }

    /// A hex is passable if it is unoccupied, OR its occupant is `ignore`.
    pub fn is_passable(&self, h: HexCoord, ignore: Entity) -> bool {
        match self.occupied.get(&h) {
            None => true,
            Some(&e) => e == ignore,
        }
    }

    /// Manhattan distance in hex space.
    pub fn hex_distance(a: HexCoord, b: HexCoord) -> i32 {
        let (aq, ar) = a;
        let (bq, br) = b;
        let dq = bq - aq;
        let dr = br - ar;
        (dq.abs() + dr.abs() + (dq + dr).abs()) / 2
    }

    /// Run A* from `start` to `goal`.  Returns an empty vec if no path exists.
    /// The returned vec does NOT include the start cell; it ends with `goal`.
    ///
    /// `terrain` — if provided, impassable hex cells are blocked and edge costs
    /// are weighted by `1 / speed_mult` (open=10, ecm_sparse≈14, ecm_dense=25).
    /// Without terrain the cost is a uniform 10 per step.
    pub fn a_star(
        &self,
        mover: Entity,
        start: HexCoord,
        goal: HexCoord,
        terrain: &HashMap<(i32, i32), HexTerrain>,
    ) -> Vec<HexCoord> {
        // All costs are scaled ×10 so integer arithmetic stays admissible.
        const BASE_COST: i32 = 10;

        let mut open: BinaryHeap<(Reverse<i32>, i32, HexCoord)> = BinaryHeap::new();
        let mut g_score: HashMap<HexCoord, i32> = HashMap::new();
        let mut came_from: HashMap<HexCoord, HexCoord> = HashMap::new();

        g_score.insert(start, 0);
        open.push((Reverse(Self::hex_distance(start, goal) * BASE_COST), 0, start));

        while let Some((_, g, current)) = open.pop() {
            if current == goal {
                let mut path = Vec::new();
                let mut c = current;
                while c != start {
                    path.push(c);
                    c = came_from[&c];
                }
                path.reverse();
                return path;
            }

            if g_score.get(&current).copied().unwrap_or(i32::MAX) < g {
                continue;
            }

            for &nb in Self::neighbours(current).iter() {
                if !self.is_passable(nb, mover) && nb != goal {
                    continue;
                }

                // Terrain passability + weighted cost.
                let step_cost = {
                    let nb_world = self.hex_to_world(nb);
                    let map_hex = crate::map::hex::world_to_hex(nb_world.x, nb_world.z);
                    match terrain.get(&(map_hex.col, map_hex.row)) {
                        Some(t) if nb != goal => {
                            match terrain_step_cost(t) {
                                Some(c) => c,
                                None    => continue, // impassable terrain — skip
                            }
                        }
                        _ => BASE_COST, // outside map or at the goal: use base cost
                    }
                };

                let tentative_g = g + step_cost;
                if tentative_g < g_score.get(&nb).copied().unwrap_or(i32::MAX) {
                    g_score.insert(nb, tentative_g);
                    came_from.insert(nb, current);
                    let f = tentative_g + Self::hex_distance(nb, goal) * BASE_COST;
                    open.push((Reverse(f), tentative_g, nb));
                }
            }
        }

        Vec::new()
    }

    /// Rebuild occupancy from current positions in the ECS world.
    pub fn rebuild(&mut self, world: &hecs::World) {
        self.occupied.clear();
        // Neuroblasts occupy the grid
        for (entity, pos) in world.query::<&neuronify_core::Position>()
            .with::<&crate::components::Neuroblast>()
            .iter()
        {
            let h = self.world_to_hex(pos.position);
            self.occupied.insert(h, entity);
        }
        // Maturing neurons also occupy grid cells so they block paths
        for (entity, pos) in world.query::<&neuronify_core::Position>()
            .with::<&crate::components::MaturingNeuron>()
            .iter()
        {
            let h = self.world_to_hex(pos.position);
            self.occupied.insert(h, entity);
        }
    }
}

/// Hex-cube rounding — maps fractional axial coordinates to the nearest integer hex.
fn round_hex(q_frac: f32, r_frac: f32) -> HexCoord {
    let s_frac = -q_frac - r_frac;
    let mut q = q_frac.round() as i32;
    let mut r = r_frac.round() as i32;
    let s = s_frac.round() as i32;

    let dq = (q as f32 - q_frac).abs();
    let dr = (r as f32 - r_frac).abs();
    let ds = (s as f32 - s_frac).abs();

    if dq > dr && dq > ds {
        q = -r - s;
    } else if dr > ds {
        r = -q - s;
    }

    (q, r)
}
