//! Generate the initial scenario map layout.

use std::collections::HashMap;

use super::terrain_model::*;
use super::voronoi::{self, Point2};

/// Map dimensions (in the Voronoi coordinate system).
pub const MAP_W: f32 = 700.0;
pub const MAP_H: f32 = 500.0;
pub const CELL_SPACING: f32 = 42.0;

/// Generate the default scenario map.
pub fn generate_scenario_map() -> MapModel {
    let mut pts = voronoi::poisson_disk(MAP_W, MAP_H, CELL_SPACING, 42);
    pts = voronoi::lloyd_relax(&pts, MAP_W, MAP_H, 3);

    let mut cell_terrains: Vec<EditorTerrain> = vec![EditorTerrain::Open; pts.len()];
    let triangles = voronoi::delaunay(&pts, MAP_W, MAP_H);

    let paint_circle = |terrains: &mut Vec<EditorTerrain>, pts: &[Point2], cx: f32, cy: f32, r: f32, t: EditorTerrain| {
        for (i, p) in pts.iter().enumerate() {
            if (p.x - cx).powi(2) + (p.y - cy).powi(2) < r * r {
                terrains[i] = t;
            }
        }
    };

    let paint_line = |terrains: &mut Vec<EditorTerrain>, pts: &[Point2], x1: f32, y1: f32, x2: f32, y2: f32, w: f32, t: EditorTerrain| {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let l = (dx * dx + dy * dy).sqrt();
        if l < 1.0 {
            return;
        }
        for (i, p) in pts.iter().enumerate() {
            let tt = ((p.x - x1) * dx + (p.y - y1) * dy) / (l * l);
            let tt = tt.clamp(0.0, 1.0);
            let px = x1 + tt * dx;
            let py = y1 + tt * dy;
            if (p.x - px).powi(2) + (p.y - py).powi(2) < w * w {
                terrains[i] = t;
            }
        }
    };

    // ── Non-open border (GlialScar around the map edges) ─────────────────
    let margin = 30.0;
    for (i, p) in pts.iter().enumerate() {
        if p.x < margin || p.x > MAP_W - margin || p.y < margin || p.y > MAP_H - margin {
            cell_terrains[i] = EditorTerrain::GlialScar;
        }
    }

    // ── Vessel network ───────────────────────────────────────────────────
    // Width must be >= CELL_SPACING/2 (~21) to reliably paint at least one cell.
    // Two main vessels with a clear gap in the middle for the open corridor.
    paint_line(&mut cell_terrains, &pts, 80.0, 190.0, 300.0, 200.0, 28.0, EditorTerrain::Vessel);
    // Branch up from upper vessel.
    paint_line(&mut cell_terrains, &pts, 200.0, 195.0, 180.0, 100.0, 24.0, EditorTerrain::Vessel);
    // Lower vessel: runs center-to-right.
    paint_line(&mut cell_terrains, &pts, 400.0, 280.0, 620.0, 300.0, 28.0, EditorTerrain::Vessel);
    // Branch down from lower vessel.
    paint_line(&mut cell_terrains, &pts, 520.0, 295.0, 540.0, 400.0, 24.0, EditorTerrain::Vessel);
    // Vessel segment top-right (near enemy base, for enemy glial harvesting).
    paint_line(&mut cell_terrains, &pts, 500.0, 100.0, 580.0, 80.0, 24.0, EditorTerrain::Vessel);
    // Vessel segment bottom-left (near player base, for player glial harvesting).
    paint_line(&mut cell_terrains, &pts, 100.0, 350.0, 150.0, 420.0, 24.0, EditorTerrain::Vessel);

    // ── CSF pools ────────────────────────────────────────────────────────
    paint_circle(&mut cell_terrains, &pts, 140.0, 120.0, 40.0, EditorTerrain::Csf);
    paint_circle(&mut cell_terrains, &pts, 560.0, 400.0, 40.0, EditorTerrain::Csf);
    paint_circle(&mut cell_terrains, &pts, 350.0, 430.0, 35.0, EditorTerrain::Csf);

    // ── Scar walls (partial barriers, not blocking the corridor) ─────────
    paint_line(&mut cell_terrains, &pts, 250.0, 310.0, 300.0, 360.0, 18.0, EditorTerrain::GlialScar);
    paint_line(&mut cell_terrains, &pts, 430.0, 140.0, 470.0, 170.0, 16.0, EditorTerrain::GlialScar);
    paint_circle(&mut cell_terrains, &pts, 280.0, 150.0, 18.0, EditorTerrain::GlialScar);
    paint_circle(&mut cell_terrains, &pts, 450.0, 370.0, 18.0, EditorTerrain::GlialScar);

    // ── Ensure player/enemy base areas are open ──────────────────────────
    // Player base: lower-left.
    paint_circle(&mut cell_terrains, &pts, 100.0, 400.0, 60.0, EditorTerrain::Open);
    // Enemy base: upper-right.
    paint_circle(&mut cell_terrains, &pts, 580.0, 100.0, 60.0, EditorTerrain::Open);

    // Bias vessel boundaries so vessel color extends down the slope to ground.
    // Bias > 0.5 means the second terrain in the EdgeKey wins more area.
    // EdgeKey is sorted, so Open < Vessel → bias > 0.5 means Vessel wins.
    let mut transition_biases = HashMap::new();
    transition_biases.insert(EdgeKey::new(EditorTerrain::Open, EditorTerrain::Vessel), 0.8);
    transition_biases.insert(EdgeKey::new(EditorTerrain::Csf, EditorTerrain::Vessel), 0.8);
    transition_biases.insert(EdgeKey::new(EditorTerrain::GlialScar, EditorTerrain::Vessel), 0.7);

    let mut model = MapModel {
        cell_centers: pts,
        cell_terrains,
        triangles,
        edge_profiles: HashMap::new(),
        interior_profiles: HashMap::new(),
        terrain_height_offsets: HashMap::new(),
        transition_biases,
    };

    model.ensure_profiles();
    model
}
