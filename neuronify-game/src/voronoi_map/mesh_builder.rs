//! Build 3D mesh from MapModel.
//!
//! Port of the JSX `buildMapWithIdentity()` function. Each Delaunay triangle
//! is subdivided into N² sub-triangles, with heights from edge/interior profiles.

use std::collections::HashMap;

use visula::primitives::mesh_primitive::MeshVertexAttributes;

use super::terrain_model::*;
use super::voronoi::Point2;

/// Result of mesh building, including vertex classification data for editing.
pub struct MeshData {
    pub vertices: Vec<MeshVertexAttributes>,
    pub indices: Vec<u32>,
    /// For each vertex, a world-position key string (or "wall" for wall verts).
    pub vertex_world_keys: Vec<String>,
    /// Map from world-position key to list of vertex classifications.
    pub world_vert_cls: HashMap<String, Vec<VertexClassification>>,
    /// For each world key, the tile's affine c1 vector (pb - pa) in 2D map space.
    /// Used for converting world deltas to local tile coords.
    pub world_key_c1: HashMap<String, [f32; 2]>,
    /// For each world key, the tile's affine c2 vector.
    pub world_key_c2: HashMap<String, [f32; 2]>,
}


fn darken(color: [u8; 3], factor: f32) -> [u8; 3] {
    [
        (color[0] as f32 * factor) as u8,
        (color[1] as f32 * factor) as u8,
        (color[2] as f32 * factor) as u8,
    ]
}

/// All 6 permutations of (0,1,2).
const PERMS: [[usize; 3]; 6] = [
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [1, 2, 0],
    [2, 0, 1],
    [2, 1, 0],
];

/// Build the complete terrain mesh from a MapModel.
pub fn build_map_mesh(model: &MapModel, _map_w: f32, _map_h: f32, cell_spacing: f32) -> MeshData {
    let pts: Vec<Point2> = model.cell_centers.clone();
    let max_edge = cell_spacing * 3.0;

    // Filter triangles with edges that are too long (border artifacts).
    let filtered: Vec<[usize; 3]> = model
        .triangles
        .iter()
        .filter(|t| {
            let d01 = pts[t[0]].dist_sq(pts[t[1]]).sqrt();
            let d12 = pts[t[1]].dist_sq(pts[t[2]]).sqrt();
            let d02 = pts[t[0]].dist_sq(pts[t[2]]).sqrt();
            d01 < max_edge && d12 < max_edge && d02 < max_edge
        })
        .copied()
        .collect();

    let mut all_pos: Vec<[f32; 3]> = Vec::new();
    let mut all_col: Vec<[u8; 3]> = Vec::new();
    let mut all_nrm: Vec<[f32; 3]> = Vec::new();
    let mut vertex_world_keys: Vec<String> = Vec::new();
    let mut world_vert_cls: HashMap<String, Vec<VertexClassification>> = HashMap::new();
    let mut world_key_c1: HashMap<String, [f32; 2]> = HashMap::new();
    let mut world_key_c2: HashMap<String, [f32; 2]> = HashMap::new();

    let cen_x: f32 = pts.iter().map(|p| p.x).sum::<f32>() / pts.len() as f32;
    let cen_y: f32 = pts.iter().map(|p| p.y).sum::<f32>() / pts.len() as f32;

    for tri in &filtered {
        let [i0, i1, i2] = *tri;
        let at0 = model.cell_terrains[i0];
        let at1 = model.cell_terrains[i1];
        let at2 = model.cell_terrains[i2];

        let mut sorted = [at0, at1, at2];
        sorted.sort();
        let [ta, tb, tc] = sorted;

        let sorted_colors = [ta.color_u8(), tb.color_u8(), tc.color_u8()];
        let dark_colors = [
            darken(sorted_colors[0], 0.65),
            darken(sorted_colors[1], 0.65),
            darken(sorted_colors[2], 0.65),
        ];

        let actual_t = [at0, at1, at2];
        let actual_p = [pts[i0], pts[i1], pts[i2]];

        // Find permutation mapping actual vertices to sorted order with CCW winding.
        let perm = find_permutation(&actual_t, &actual_p, ta, tb, tc);
        let perm = match perm {
            Some(p) => p,
            None => continue,
        };

        let pa = actual_p[perm[0]];
        let pb = actual_p[perm[1]];
        let pc = actual_p[perm[2]];

        let c1x = pb.x - pa.x;
        let c1y = pb.y - pa.y;
        let c2x = (pc.x - pa.x - 0.5 * c1x) / UNIT_H;
        let c2y = (pc.y - pa.y - 0.5 * c1y) / UNIT_H;

        let to_world = |lx: f32, ly: f32, lz: f32| -> [f32; 3] {
            let ux = lx;
            let uy = -lz;
            [
                pa.x + c1x * ux + c2x * uy - cen_x,
                ly,
                -(pa.y + c1y * ux + c2y * uy - cen_y),
            ]
        };

        // The three cell centers and indices in sorted (canonical) order.
        let cell_a = actual_p[perm[0]]; // ta
        let cell_b = actual_p[perm[1]]; // tb
        let cell_c = actual_p[perm[2]]; // tc
        let actual_idx = [i0, i1, i2];
        let idx_a = actual_idx[perm[0]];
        let idx_b = actual_idx[perm[1]];
        let idx_c = actual_idx[perm[2]];

        let vert_height = |i: usize, j: usize, k: usize| -> f32 {
            model.vertex_height(ta, tb, tc, cell_a, cell_b, cell_c, idx_a, idx_b, idx_c, i, j, k)
        };

        let classify_vert = |i: usize, j: usize, k: usize| -> VertexClassification {
            MapModel::classify_vertex(ta, tb, tc, cell_a, cell_b, cell_c, idx_a, idx_b, idx_c, i, j, k)
        };

        struct RegVert {
            wp: [f32; 3],
            wk: String,
        }

        let mut register_vert = |i: usize, j: usize, k: usize| -> RegVert {
            let mut lx = j as f32 / N as f32 + (k as f32 / N as f32) * 0.5;
            let mut ly = (k as f32 / N as f32) * UNIT_H;
            // Apply local XY offset for interior vertices.
            if i > 0 && j > 0 && k > 0 && i < N && j < N && k < N {
                let (dlx, dly) = model.interior_local_offset(ta, tb, tc, i, j, k);
                lx += dlx;
                ly += dly;
            }
            let z = vert_height(i, j, k);
            let wp = to_world(lx, z, -ly);

            // Build the world-position key for vertex deduplication.
            //
            // Edge and corner vertices are shared between adjacent triangles.
            // Each triangle computes positions through its own affine transform,
            // which can produce tiny floating-point differences for the same
            // logical vertex.  To guarantee identical keys, we compute a
            // *canonical* 2D position for each vertex class:
            //
            //  - Corner (i==N, j==N, or k==N): use the cell centre directly.
            //  - Edge (k==0, i==0, or j==0): lerp between the two endpoint
            //    cell centres using the EdgeKey's canonical parameter.
            //  - Interior: use the full barycentric interpolation of this
            //    triangle's three cell centres (interior vertices are NOT
            //    shared across triangles, so precision doesn't matter).
            let cls = classify_vert(i, j, k);

            let (canon_x, canon_z) = match &cls {
                VertexClassification::Corner { .. } => {
                    // Use the cell centre directly for canonical position.
                    if i == N { (cell_a.x, cell_a.y) }
                    else if j == N { (cell_b.x, cell_b.y) }
                    else { (cell_c.x, cell_c.y) }
                }
                VertexClassification::Edge { key: _, param_idx } => {
                    // Use the actual cell pair for the specific barycentric
                    // edge this vertex lies on (k==0 → a–b, i==0 → b–c,
                    // j==0 → a–c).  This avoids the ambiguity when two edges
                    // of the same triangle share the same EdgeKey.
                    //
                    // param_idx goes from 0 at the canonical start to N at
                    // the canonical end.  The canonical direction is determined
                    // by edge_param_idx() which may reverse the raw parameter.
                    // To get the correct lerp position, we reconstruct the
                    // raw parameter and lerp along the barycentric edge.
                    let t = *param_idx as f32 / N as f32;
                    let (c_start, c_end) = if k == 0 {
                        // k==0 edge: ta(cell_a) to tb(cell_b)
                        canonical_edge_cells(ta, tb, cell_a, cell_b)
                    } else if i == 0 {
                        // i==0 edge: tb(cell_b) to tc(cell_c)
                        canonical_edge_cells(tb, tc, cell_b, cell_c)
                    } else {
                        // j==0 edge: ta(cell_a) to tc(cell_c)
                        canonical_edge_cells(ta, tc, cell_a, cell_c)
                    };
                    (
                        c_start.x * (1.0 - t) + c_end.x * t,
                        c_start.y * (1.0 - t) + c_end.y * t,
                    )
                }
                VertexClassification::Interior { .. } => {
                    let wi = i as f32 / N as f32;
                    let wj = j as f32 / N as f32;
                    let wk_b = k as f32 / N as f32;
                    (
                        cell_a.x * wi + cell_b.x * wj + cell_c.x * wk_b,
                        cell_a.y * wi + cell_b.y * wj + cell_c.y * wk_b,
                    )
                }
            };
            let wk = format!("{:.2},{:.2}", canon_x - cen_x, -(canon_z - cen_y));

            {
                let entry = world_vert_cls.entry(wk.clone()).or_default();
                if !entry.iter().any(|c| *c == cls) {
                    entry.push(cls);
                }
            }
            // Store the tile's affine transform for this vertex so the
            // editor can convert world deltas to local coords.
            world_key_c1.entry(wk.clone()).or_insert([c1x, c1y]);
            world_key_c2.entry(wk.clone()).or_insert([c2x, c2y]);
            RegVert { wp, wk }
        };

        // Sub-face data for wall generation.
        struct SubFace {
            vl: [([f32; 2], f32); 3], // (local_xy, height)
            dom: usize,
        }
        let mut sub_faces: Vec<SubFace> = Vec::new();

        let vl = |ii: usize, jj: usize, kk: usize| -> ([f32; 2], f32) {
            let mut lx = jj as f32 / N as f32 + (kk as f32 / N as f32) * 0.5;
            let mut ly = (kk as f32 / N as f32) * UNIT_H;
            if ii > 0 && jj > 0 && kk > 0 && ii < N && jj < N && kk < N {
                let (dlx, dly) = model.interior_local_offset(ta, tb, tc, ii, jj, kk);
                lx += dlx;
                ly += dly;
            }
            ([lx, ly], vert_height(ii, jj, kk))
        };

        // Generate sub-triangles.
        for i in 0..N {
            for j in 0..(N - i) {
                let k = N - i - j;

                // Upward-pointing triangle.
                let r0 = register_vert(i, j, k);
                let r1 = register_vert(i, j + 1, k - 1);
                let r2 = register_vert(i + 1, j, k - 1);

                let cwa = (3 * i + 1) as f32 / (3 * N) as f32;
                let cwb = (3 * j + 1) as f32 / (3 * N) as f32;
                let cwc = 1.0 - cwa - cwb;
                let fdom = model.biased_dominant(cwa, cwb, cwc, ta, tb, tc);

                push_tri(
                    &r0.wp, &r1.wp, &r2.wp,
                    sorted_colors[fdom],
                    &r0.wk, &r1.wk, &r2.wk,
                    &mut all_pos, &mut all_col, &mut all_nrm, &mut vertex_world_keys,
                );

                sub_faces.push(SubFace {
                    vl: [vl(i, j, k), vl(i, j + 1, k - 1), vl(i + 1, j, k - 1)],
                    dom: fdom,
                });

                // Downward-pointing triangle.
                if j + 1 <= N - i - 1 {
                    let r3 = register_vert(i + 1, j, k - 1);
                    let r4 = register_vert(i, j + 1, k - 1);
                    let r5 = register_vert(i + 1, j + 1, k - 2);

                    let dwa = (3 * (i + 1)) as f32 / (3 * N) as f32 - 1.0 / (3 * N) as f32;
                    let dwb = (3 * j + 2) as f32 / (3 * N) as f32;
                    let dwc = 1.0 - dwa - dwb;
                    let fdom2 = model.biased_dominant(dwa, dwb, dwc, ta, tb, tc);

                    push_tri(
                        &r3.wp, &r4.wp, &r5.wp,
                        sorted_colors[fdom2],
                        &r3.wk, &r4.wk, &r5.wk,
                        &mut all_pos, &mut all_col, &mut all_nrm, &mut vertex_world_keys,
                    );

                    sub_faces.push(SubFace {
                        vl: [
                            vl(i + 1, j, k - 1),
                            vl(i, j + 1, k - 1),
                            vl(i + 1, j + 1, k - 2),
                        ],
                        dom: fdom2,
                    });
                }
            }
        }

        // Generate walls between adjacent sub-faces with height discontinuities.
        for si in 0..sub_faces.len() {
            for sj in (si + 1)..sub_faces.len() {
                let fa = &sub_faces[si];
                let fb = &sub_faces[sj];

                // Find shared edge vertices.
                let mut shared = Vec::new();
                for ai in 0..3 {
                    for bi in 0..3 {
                        if (fa.vl[ai].0[0] - fb.vl[bi].0[0]).abs() < 0.001
                            && (fa.vl[ai].0[1] - fb.vl[bi].0[1]).abs() < 0.001
                        {
                            shared.push((fa.vl[ai].0, fa.vl[ai].1, fb.vl[bi].1));
                            break;
                        }
                    }
                }

                if shared.len() < 2 {
                    continue;
                }

                let (s0_lxy, s0_za, s0_zb) = shared[0];
                let (s1_lxy, s1_za, s1_zb) = shared[1];

                if (s0_za - s0_zb).abs() < 0.5 && (s1_za - s1_zb).abs() < 0.5 {
                    continue;
                }

                let hi_dom = {
                    let sum_a: f32 =
                        fa.vl[0].1 + fa.vl[1].1 + fa.vl[2].1;
                    let sum_b: f32 =
                        fb.vl[0].1 + fb.vl[1].1 + fb.vl[2].1;
                    if sum_a > sum_b {
                        fa.dom
                    } else {
                        fb.dom
                    }
                };
                let wc = dark_colors[hi_dom];

                let pw = |lxy: [f32; 2], z: f32| -> [f32; 3] {
                    to_world(lxy[0], z, -lxy[1])
                };

                push_wall_tri(
                    &pw(s0_lxy, s0_za),
                    &pw(s1_lxy, s1_za),
                    &pw(s0_lxy, s0_zb),
                    wc,
                    &mut all_pos,
                    &mut all_col,
                    &mut all_nrm,
                    &mut vertex_world_keys,
                );
                push_wall_tri(
                    &pw(s0_lxy, s0_zb),
                    &pw(s1_lxy, s1_za),
                    &pw(s1_lxy, s1_zb),
                    wc,
                    &mut all_pos,
                    &mut all_col,
                    &mut all_nrm,
                    &mut vertex_world_keys,
                );
            }
        }
    }

    // Build final vertex attributes.
    let vertices: Vec<MeshVertexAttributes> = (0..all_pos.len())
        .map(|i| MeshVertexAttributes {
            position: all_pos[i],
            normal: all_nrm[i],
            uv: [0.0, 0.0],
            color: [all_col[i][0], all_col[i][1], all_col[i][2], 255],
        })
        .collect();

    let indices: Vec<u32> = (0..vertices.len() as u32).collect();

    MeshData {
        vertices,
        indices,
        vertex_world_keys,
        world_vert_cls,
        world_key_c1,
        world_key_c2,
    }
}

/// Orient two cell centres into the canonical edge direction.
///
/// Returns `(c_start, c_end)` where `param_idx=0` is at `c_start` and
/// `param_idx=N` is at `c_end`.  Must agree with `edge_param_idx()` in
/// `terrain_model.rs`.
fn canonical_edge_cells(
    t_from: EditorTerrain,
    t_to: EditorTerrain,
    cell_from: Point2,
    cell_to: Point2,
) -> (Point2, Point2) {
    if t_from == t_to {
        // Same terrain — spatial tie-break: smaller (x, y) is the start.
        let from_is_start =
            cell_from.x < cell_to.x || (cell_from.x == cell_to.x && cell_from.y <= cell_to.y);
        if from_is_start {
            (cell_from, cell_to)
        } else {
            (cell_to, cell_from)
        }
    } else if t_from < t_to {
        (cell_from, cell_to)
    } else {
        (cell_to, cell_from)
    }
}

fn find_permutation(
    actual_t: &[EditorTerrain; 3],
    actual_p: &[Point2; 3],
    ta: EditorTerrain,
    tb: EditorTerrain,
    tc: EditorTerrain,
) -> Option<[usize; 3]> {
    // Try to find a CCW permutation first.
    for p in &PERMS {
        if actual_t[p[0]] == ta && actual_t[p[1]] == tb && actual_t[p[2]] == tc {
            let pa = actual_p[p[0]];
            let pb = actual_p[p[1]];
            let pc = actual_p[p[2]];
            let cross =
                (pb.x - pa.x) * (pc.y - pa.y) - (pb.y - pa.y) * (pc.x - pa.x);
            if cross > 0.0 {
                return Some(*p);
            }
        }
    }
    // Fallback: any matching permutation.
    for p in &PERMS {
        if actual_t[p[0]] == ta && actual_t[p[1]] == tb && actual_t[p[2]] == tc {
            return Some(*p);
        }
    }
    None
}

fn push_tri(
    p0: &[f32; 3],
    p1: &[f32; 3],
    p2: &[f32; 3],
    col: [u8; 3],
    wk0: &str,
    wk1: &str,
    wk2: &str,
    all_pos: &mut Vec<[f32; 3]>,
    all_col: &mut Vec<[u8; 3]>,
    all_nrm: &mut Vec<[f32; 3]>,
    vertex_world_keys: &mut Vec<String>,
) {
    let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    let mut nx = e1[1] * e2[2] - e1[2] * e2[1];
    let mut ny = e1[2] * e2[0] - e1[0] * e2[2];
    let mut nz = e1[0] * e2[1] - e1[1] * e2[0];
    let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-10);
    nx /= len;
    ny /= len;
    nz /= len;

    if ny < 0.0 {
        // Flip winding.
        all_pos.extend_from_slice(&[*p0, *p2, *p1]);
        nx = -nx;
        ny = -ny;
        nz = -nz;
        vertex_world_keys.push(wk0.to_string());
        vertex_world_keys.push(wk2.to_string());
        vertex_world_keys.push(wk1.to_string());
    } else {
        all_pos.extend_from_slice(&[*p0, *p1, *p2]);
        vertex_world_keys.push(wk0.to_string());
        vertex_world_keys.push(wk1.to_string());
        vertex_world_keys.push(wk2.to_string());
    }

    for _ in 0..3 {
        all_col.push(col);
        all_nrm.push([nx, ny, nz]);
    }
}

fn push_wall_tri(
    p0: &[f32; 3],
    p1: &[f32; 3],
    p2: &[f32; 3],
    col: [u8; 3],
    all_pos: &mut Vec<[f32; 3]>,
    all_col: &mut Vec<[u8; 3]>,
    all_nrm: &mut Vec<[f32; 3]>,
    vertex_world_keys: &mut Vec<String>,
) {
    let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    let mut nx = e1[1] * e2[2] - e1[2] * e2[1];
    let mut ny = e1[2] * e2[0] - e1[0] * e2[2];
    let mut nz = e1[0] * e2[1] - e1[1] * e2[0];
    let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-10);
    nx /= len;
    ny /= len;
    nz /= len;

    if ny < 0.0 {
        all_pos.extend_from_slice(&[*p0, *p2, *p1]);
        nx = -nx;
        ny = -ny;
        nz = -nz;
    } else {
        all_pos.extend_from_slice(&[*p0, *p1, *p2]);
    }

    for _ in 0..3 {
        vertex_world_keys.push("wall".to_string());
        all_col.push(col);
        all_nrm.push([nx, ny, nz]);
    }
}

/// Build a tile catalog mesh: each unique terrain triple rendered as a
/// standalone equilateral triangle, laid out in a grid.
/// Returns full `MeshData` with vertex classifications for editing.
pub fn build_tile_catalog(model: &MapModel) -> MeshData {
    use std::collections::BTreeSet;

    let mut triples = BTreeSet::new();
    let pts = &model.cell_centers;
    let cell_spacing = 42.0f32;
    let max_edge = cell_spacing * 3.0;
    for tri in &model.triangles {
        let [i0, i1, i2] = *tri;
        let d01 = pts[i0].dist_sq(pts[i1]).sqrt();
        let d12 = pts[i1].dist_sq(pts[i2]).sqrt();
        let d02 = pts[i0].dist_sq(pts[i2]).sqrt();
        if d01 >= max_edge || d12 >= max_edge || d02 >= max_edge {
            continue;
        }
        let mut sorted = [
            model.cell_terrains[i0],
            model.cell_terrains[i1],
            model.cell_terrains[i2],
        ];
        sorted.sort();
        triples.insert((sorted[0], sorted[1], sorted[2]));
    }

    let triples: Vec<(EditorTerrain, EditorTerrain, EditorTerrain)> =
        triples.into_iter().collect();
    if triples.is_empty() {
        return MeshData {
            vertices: Vec::new(),
            indices: Vec::new(),
            vertex_world_keys: Vec::new(),
            world_vert_cls: HashMap::new(),
            world_key_c1: HashMap::new(),
            world_key_c2: HashMap::new(),
        };
    }

    let cols = (triples.len() as f32).sqrt().ceil() as usize;
    let tile_size = 40.0f32;
    let spacing = tile_size * 1.4;
    let total_w = cols as f32 * spacing;
    let total_h = ((triples.len() + cols - 1) / cols) as f32 * spacing;
    let off_x = -total_w * 0.5 + tile_size * 0.5;
    let off_z = -total_h * 0.5 + tile_size * 0.3;

    let mut all_pos: Vec<[f32; 3]> = Vec::new();
    let mut all_col: Vec<[u8; 3]> = Vec::new();
    let mut all_nrm: Vec<[f32; 3]> = Vec::new();
    let mut vertex_world_keys: Vec<String> = Vec::new();
    let mut world_vert_cls: HashMap<String, Vec<VertexClassification>> = HashMap::new();
    let mut world_key_c1: HashMap<String, [f32; 2]> = HashMap::new();
    let mut world_key_c2: HashMap<String, [f32; 2]> = HashMap::new();

    let base_a = super::voronoi::Point2::new(0.0, 0.0);
    let base_b = super::voronoi::Point2::new(tile_size, 0.0);
    let base_c = super::voronoi::Point2::new(tile_size * 0.5, tile_size * UNIT_H);

    for (ti, &(ta, tb, tc)) in triples.iter().enumerate() {
        let col = ti % cols;
        let row = ti / cols;
        let ox = col as f32 * spacing;
        let oz = row as f32 * spacing;

        let cell_a = super::voronoi::Point2::new(base_a.x + ox, base_a.y + oz);
        let cell_b = super::voronoi::Point2::new(base_b.x + ox, base_b.y + oz);
        let cell_c = super::voronoi::Point2::new(base_c.x + ox, base_c.y + oz);

        let sorted_colors = [ta.color_u8(), tb.color_u8(), tc.color_u8()];

        let c1x = cell_b.x - cell_a.x;
        let c1y = cell_b.y - cell_a.y;
        let c2x = (cell_c.x - cell_a.x - 0.5 * c1x) / UNIT_H;
        let c2y = (cell_c.y - cell_a.y - 0.5 * c1y) / UNIT_H;

        let to_world = |lx: f32, ly: f32, lz: f32| -> [f32; 3] {
            let ux = lx;
            let uy = -lz;
            [
                cell_a.x + c1x * ux + c2x * uy + off_x,
                ly,
                -(cell_a.y + c1y * ux + c2y * uy + off_z),
            ]
        };

        let vert_height = |i: usize, j: usize, k: usize| -> f32 {
            model.vertex_height(ta, tb, tc, cell_a, cell_b, cell_c, usize::MAX, usize::MAX, usize::MAX, i, j, k)
        };

        let classify_vert = |i: usize, j: usize, k: usize| -> VertexClassification {
            MapModel::classify_vertex(ta, tb, tc, cell_a, cell_b, cell_c, usize::MAX, usize::MAX, usize::MAX, i, j, k)
        };

        // Register a vertex: compute position, classify, and track for editing.
        let mut register_vert = |i: usize, j: usize, k: usize| -> ([f32; 3], String) {
            let lx = j as f32 / N as f32 + (k as f32 / N as f32) * 0.5;
            let ly = (k as f32 / N as f32) * UNIT_H;
            let z = vert_height(i, j, k);
            let wp = to_world(lx, z, -ly);

            // Use classification-based key (tile-local position doesn't matter
            // for sharing — what matters is the classification).
            let cls = classify_vert(i, j, k);
            let wk = format!("cat_{:.2},{:.2}", wp[0], wp[2]);

            let entry = world_vert_cls.entry(wk.clone()).or_default();
            if !entry.iter().any(|c| *c == cls) {
                entry.push(cls);
            }
            world_key_c1.entry(wk.clone()).or_insert([c1x, c1y]);
            world_key_c2.entry(wk.clone()).or_insert([c2x, c2y]);
            (wp, wk)
        };

        for i in 0..N {
            for j in 0..(N - i) {
                let k = N - i - j;

                // Upward triangle.
                let (p0, wk0) = register_vert(i, j, k);
                let (p1, wk1) = register_vert(i, j + 1, k - 1);
                let (p2, wk2) = register_vert(i + 1, j, k - 1);

                let cwa = (3 * i + 1) as f32 / (3 * N) as f32;
                let cwb = (3 * j + 1) as f32 / (3 * N) as f32;
                let cwc = 1.0 - cwa - cwb;
                let fdom = model.biased_dominant(cwa, cwb, cwc, ta, tb, tc);

                push_tri(
                    &p0, &p1, &p2,
                    sorted_colors[fdom],
                    &wk0, &wk1, &wk2,
                    &mut all_pos, &mut all_col, &mut all_nrm, &mut vertex_world_keys,
                );

                // Downward triangle.
                if j + 1 <= N - i - 1 {
                    let (p3, wk3) = register_vert(i + 1, j, k - 1);
                    let (p4, wk4) = register_vert(i, j + 1, k - 1);
                    let (p5, wk5) = register_vert(i + 1, j + 1, k - 2);

                    let dwa = (3 * (i + 1)) as f32 / (3 * N) as f32 - 1.0 / (3 * N) as f32;
                    let dwb = (3 * j + 2) as f32 / (3 * N) as f32;
                    let dwc = 1.0 - dwa - dwb;
                    let fdom2 = model.biased_dominant(dwa, dwb, dwc, ta, tb, tc);

                    push_tri(
                        &p3, &p4, &p5,
                        sorted_colors[fdom2],
                        &wk3, &wk4, &wk5,
                        &mut all_pos, &mut all_col, &mut all_nrm, &mut vertex_world_keys,
                    );
                }
            }
        }
    }

    let vertices: Vec<MeshVertexAttributes> = (0..all_pos.len())
        .map(|i| MeshVertexAttributes {
            position: all_pos[i],
            normal: all_nrm[i],
            uv: [0.0, 0.0],
            color: [all_col[i][0], all_col[i][1], all_col[i][2], 255],
        })
        .collect();
    let indices: Vec<u32> = (0..vertices.len() as u32).collect();

    MeshData {
        vertices,
        indices,
        vertex_world_keys,
        world_vert_cls,
        world_key_c1,
        world_key_c2,
    }
}

/// Build wireframe edge data for Delaunay triangle boundaries.
/// Returns pairs of world-space points for each triangle edge.
pub fn build_wireframe_edges(model: &MapModel, _map_w: f32, _map_h: f32, cell_spacing: f32) -> Vec<([f32; 3], [f32; 3])> {
    let pts = &model.cell_centers;
    let max_edge = cell_spacing * 3.0;
    let cen_x: f32 = pts.iter().map(|p| p.x).sum::<f32>() / pts.len() as f32;
    let cen_y: f32 = pts.iter().map(|p| p.y).sum::<f32>() / pts.len() as f32;

    let mut edges = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for tri in &model.triangles {
        let [i0, i1, i2] = *tri;
        let d01 = pts[i0].dist_sq(pts[i1]).sqrt();
        let d12 = pts[i1].dist_sq(pts[i2]).sqrt();
        let d02 = pts[i0].dist_sq(pts[i2]).sqrt();
        if d01 >= max_edge || d12 >= max_edge || d02 >= max_edge {
            continue;
        }

        for (a, b) in [(i0, i1), (i1, i2), (i0, i2)] {
            let key = if a < b { (a, b) } else { (b, a) };
            if seen.insert(key) {
                let pa = pts[a];
                let pb = pts[b];
                let y = 0.5; // slightly above ground
                edges.push((
                    [pa.x - cen_x, y, -(pa.y - cen_y)],
                    [pb.x - cen_x, y, -(pb.y - cen_y)],
                ));
            }
        }
    }
    edges
}
