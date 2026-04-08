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

    // Paint terrain using the same layout as the JSX reference.
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

    // Vessel network (blood vessels running through the middle).
    paint_line(&mut cell_terrains, &pts, 50.0, 250.0, 650.0, 220.0, 28.0, EditorTerrain::Vessel);
    paint_line(&mut cell_terrains, &pts, 200.0, 240.0, 500.0, 230.0, 18.0, EditorTerrain::Vessel);
    paint_line(&mut cell_terrains, &pts, 400.0, 230.0, 480.0, 100.0, 20.0, EditorTerrain::Vessel);
    paint_line(&mut cell_terrains, &pts, 200.0, 245.0, 150.0, 370.0, 18.0, EditorTerrain::Vessel);
    paint_line(&mut cell_terrains, &pts, 150.0, 370.0, 500.0, 360.0, 18.0, EditorTerrain::Vessel);
    paint_line(&mut cell_terrains, &pts, 320.0, 230.0, 320.0, 160.0, 14.0, EditorTerrain::Vessel);
    paint_line(&mut cell_terrains, &pts, 500.0, 350.0, 550.0, 430.0, 14.0, EditorTerrain::Vessel);

    // CSF pools (fluid areas).
    paint_circle(&mut cell_terrains, &pts, 130.0, 130.0, 55.0, EditorTerrain::Csf);
    paint_circle(&mut cell_terrains, &pts, 170.0, 110.0, 40.0, EditorTerrain::Csf);
    paint_circle(&mut cell_terrains, &pts, 550.0, 400.0, 50.0, EditorTerrain::Csf);
    paint_circle(&mut cell_terrains, &pts, 580.0, 380.0, 40.0, EditorTerrain::Csf);
    paint_circle(&mut cell_terrains, &pts, 350.0, 430.0, 45.0, EditorTerrain::Csf);
    paint_circle(&mut cell_terrains, &pts, 100.0, 350.0, 35.0, EditorTerrain::Csf);

    // Scar walls.
    paint_line(&mut cell_terrains, &pts, 250.0, 300.0, 330.0, 370.0, 22.0, EditorTerrain::GlialScar);
    paint_line(&mut cell_terrains, &pts, 430.0, 160.0, 500.0, 140.0, 20.0, EditorTerrain::GlialScar);
    paint_line(&mut cell_terrains, &pts, 80.0, 420.0, 170.0, 450.0, 18.0, EditorTerrain::GlialScar);
    paint_circle(&mut cell_terrains, &pts, 400.0, 380.0, 25.0, EditorTerrain::GlialScar);
    paint_circle(&mut cell_terrains, &pts, 280.0, 150.0, 20.0, EditorTerrain::GlialScar);
    paint_circle(&mut cell_terrains, &pts, 600.0, 200.0, 22.0, EditorTerrain::GlialScar);

    let n_cells = pts.len();
    let mut model = MapModel {
        cell_centers: pts,
        cell_terrains,
        triangles,
        edge_profiles: HashMap::new(),
        interior_profiles: HashMap::new(),
        cell_height_offsets: vec![0.0; n_cells],
    };

    model.ensure_profiles();
    model
}
