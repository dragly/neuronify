//! Edit mode logic: vertex picking, height dragging, terrain painting.

use glam::Vec3;

use neuronify_game_lib::voronoi_map::{
    mesh_builder::MeshData,
    terrain_model::*,
    voronoi::Point2,
};

/// Pick the nearest editable vertex to the screen position.
/// Returns the world-position key if a vertex is close enough.
pub fn pick_vertex(
    application: &visula::Application,
    cached_mesh_data: &Option<MeshData>,
    screen_x: f32,
    screen_y: f32,
) -> Option<String> {
    let data = cached_mesh_data.as_ref()?;
    let aspect = application.config.width as f32 / application.config.height as f32;
    let view = application.camera_controller.view_matrix();
    let proj = application.camera_controller.projection_matrix(aspect);
    let vp = proj * view;
    let canvas_w = application.config.width as f32;
    let canvas_h = application.config.height as f32;

    let mut closest_dist = 18.0f32;
    let mut closest_wk: Option<String> = None;
    let mut seen = std::collections::HashSet::new();

    for vi in 0..data.vertex_world_keys.len() {
        let wk = &data.vertex_world_keys[vi];
        if wk == "wall" || seen.contains(wk) {
            continue;
        }
        let cls = match data.world_vert_cls.get(wk) {
            Some(c) if !c.is_empty() => c,
            _ => continue,
        };
        seen.insert(wk.clone());

        let v = &data.vertices[vi];
        let wp = glam::Vec4::new(v.position[0], v.position[1], v.position[2], 1.0);
        let clip = vp * wp;
        if clip.w <= 0.0 {
            continue;
        }
        let ndc_x = clip.x / clip.w;
        let ndc_y = clip.y / clip.w;
        let sx = (ndc_x * 0.5 + 0.5) * canvas_w;
        let sy = (1.0 - (ndc_y * 0.5 + 0.5)) * canvas_h;

        let dx = sx - screen_x;
        let dy = sy - screen_y;
        let d = (dx * dx + dy * dy).sqrt();
        if d < closest_dist {
            closest_dist = d;
            closest_wk = Some(wk.clone());
        }
    }

    closest_wk
}

/// Drag a picked vertex by a height delta, updating edge/interior profiles.
pub fn drag_vertex(
    model: &mut MapModel,
    world_key: &str,
    delta: f32,
    cached_mesh_data: &Option<MeshData>,
) {
    let data = match cached_mesh_data {
        Some(d) => d,
        None => return,
    };

    let cls_list = match data.world_vert_cls.get(world_key) {
        Some(c) => c,
        None => return,
    };

    for cls in cls_list {
        match cls {
            VertexClassification::Edge { key, param_idx } => {
                if let Some(profile) = model.edge_profiles.get_mut(key) {
                    if *param_idx < profile.len() {
                        profile[*param_idx] += delta;
                    }
                }
            }
            VertexClassification::Interior { key, i, j, k } => {
                if let Some(pts) = model.interior_profiles.get_mut(key) {
                    for p in pts.iter_mut() {
                        if p.i == *i && p.j == *j && p.k == *k {
                            p.z += delta;
                            break;
                        }
                    }
                }
            }
            VertexClassification::Corner { terrain } => {
                *model.terrain_height_offsets.entry(*terrain).or_insert(0.0) += delta;
            }
        }
    }
}

/// Paint the nearest cell to the given world position with the selected terrain.
pub fn paint_cell(
    model: &mut MapModel,
    world_pos: Vec3,
    terrain: EditorTerrain,
    map_w: f32,
    map_h: f32,
) {
    // World coordinates: x maps to voronoi x (with centering offset), z maps to voronoi y (negated).
    let cen_x: f32 = model.cell_centers.iter().map(|p| p.x).sum::<f32>()
        / model.cell_centers.len() as f32;
    let cen_y: f32 = model.cell_centers.iter().map(|p| p.y).sum::<f32>()
        / model.cell_centers.len() as f32;

    // Reverse the to_world transform: world_x = voronoi_x - cen_x, world_z = -(voronoi_y - cen_y)
    let vx = world_pos.x + cen_x;
    let vy = -world_pos.z + cen_y;

    let target = Point2::new(vx, vy);

    let mut best_idx = 0;
    let mut best_dist = f32::MAX;
    for (i, p) in model.cell_centers.iter().enumerate() {
        let d = p.dist_sq(target);
        if d < best_dist {
            best_dist = d;
            best_idx = i;
        }
    }

    if model.cell_terrains[best_idx] != terrain {
        model.cell_terrains[best_idx] = terrain;
        model.ensure_profiles();
    }
}
