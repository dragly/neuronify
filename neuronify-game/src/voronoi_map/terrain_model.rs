//! Terrain types, edge/interior profiles, and the complete editable map model.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::voronoi::Point2;

/// Subdivision level: each triangle edge is divided into N segments.
pub const N: usize = 6;
pub const UNIT_H: f32 = 0.866025404; // sqrt(3)/2

// ── Terrain types ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EditorTerrain {
    Csf,
    GlialScar,
    Open,
    Vessel,
}

impl EditorTerrain {
    pub const ALL: [EditorTerrain; 4] = [
        EditorTerrain::Csf,
        EditorTerrain::GlialScar,
        EditorTerrain::Open,
        EditorTerrain::Vessel,
    ];

    pub fn name(self) -> &'static str {
        match self {
            EditorTerrain::Open => "open",
            EditorTerrain::Vessel => "vessel",
            EditorTerrain::GlialScar => "scar",
            EditorTerrain::Csf => "csf",
        }
    }

    pub fn height(self) -> f32 {
        match self {
            EditorTerrain::Open => 0.0,
            EditorTerrain::Vessel => 12.0,
            EditorTerrain::GlialScar => 20.0,
            EditorTerrain::Csf => -6.0,
        }
    }

    /// Color as [R, G, B] in 0..255.
    pub fn color_u8(self) -> [u8; 3] {
        match self {
            EditorTerrain::Open => [190, 178, 148],       // warm sandy tan
            EditorTerrain::Vessel => [185, 62, 72],       // deep arterial red
            EditorTerrain::GlialScar => [105, 88, 58],    // dark earthy brown
            EditorTerrain::Csf => [68, 128, 178],         // saturated steel blue
        }
    }

    /// Color as [R, G, B] in 0.0..1.0.
    pub fn color_f32(self) -> [f32; 3] {
        let [r, g, b] = self.color_u8();
        [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
    }
}

// ── Canonical keys ───────────────────────────────────────────────────────────

/// Canonical key for a pair of terrain types (sorted).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeKey(pub EditorTerrain, pub EditorTerrain);

impl EdgeKey {
    pub fn new(a: EditorTerrain, b: EditorTerrain) -> Self {
        if a <= b {
            EdgeKey(a, b)
        } else {
            EdgeKey(b, a)
        }
    }
}

/// Canonical key for a triple of terrain types (sorted).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TripleKey(pub EditorTerrain, pub EditorTerrain, pub EditorTerrain);

impl TripleKey {
    pub fn new(a: EditorTerrain, b: EditorTerrain, c: EditorTerrain) -> Self {
        let mut arr = [a, b, c];
        arr.sort();
        TripleKey(arr[0], arr[1], arr[2])
    }
}

// ── Interior point ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct InteriorPoint {
    pub i: usize,
    pub j: usize,
    pub k: usize,
    pub z: f32,
    /// Local-space X offset (along the base edge of the unit triangle).
    #[serde(default)]
    pub dlx: f32,
    /// Local-space Y offset (perpendicular to the base edge, toward apex).
    #[serde(default)]
    pub dly: f32,
}

// ── Vertex classification ────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub enum VertexClassification {
    /// A tile corner vertex. The height offset is shared across ALL tiles
    /// that have this terrain type in this slot of their canonical triple.
    /// `slot` is 0 (ta), 1 (tb), or 2 (tc) in the sorted triple.
    Corner {
        terrain: EditorTerrain,
    },
    Edge {
        key: EdgeKey,
        param_idx: usize,
    },
    Interior {
        key: TripleKey,
        i: usize,
        j: usize,
        k: usize,
    },
}

// ── Edge parameterisation helper ──────────────────────────────────────────────

/// Compute the canonical `param_idx` for an edge vertex.
///
/// `t_from` / `t_to` are the terrain types at the two endpoints (in the
/// triangle's sorted slot order), and `cell_from` / `cell_to` are their
/// cell-centre positions.  `raw` is the barycentric step count along the
/// `from→to` direction (0 at `from`, N at `to`).
///
/// The EdgeKey is always stored with the lesser terrain first.  When the two
/// terrains differ, `param_idx` increases toward the *greater* terrain.
/// When they are equal, we use a spatial tie-break (smaller `(x,y)` is the
/// 0-end) so that every triangle sharing the same physical Delaunay edge
/// agrees on the direction.
fn edge_param_idx(
    t_from: EditorTerrain,
    t_to: EditorTerrain,
    cell_from: super::voronoi::Point2,
    cell_to: super::voronoi::Point2,
    raw: usize,
) -> usize {
    if t_from == t_to {
        // Same terrain — coordinate tie-break.
        // "Forward" = from the cell with smaller (x, y) to the larger one.
        let from_is_canonical_start =
            cell_from.x < cell_to.x || (cell_from.x == cell_to.x && cell_from.y <= cell_to.y);
        if from_is_canonical_start { raw } else { N - raw }
    } else if t_from < t_to {
        // from is the lesser terrain → forward direction matches canonical
        raw
    } else {
        // from is the greater terrain → reverse
        N - raw
    }
}

// ── Map model ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapModel {
    pub cell_centers: Vec<Point2>,
    pub cell_terrains: Vec<EditorTerrain>,
    pub triangles: Vec<[usize; 3]>,
    #[serde(with = "map_as_vec")]
    pub edge_profiles: HashMap<EdgeKey, Vec<f32>>,
    #[serde(with = "map_as_vec")]
    pub interior_profiles: HashMap<TripleKey, Vec<InteriorPoint>>,
    /// Per-terrain-type height offset (added to every corner of that type).
    /// Shared across ALL tiles on the map — editing one corner of a terrain
    /// type moves all corners of that type, just like edge profiles.
    #[serde(default, with = "map_as_vec")]
    pub terrain_height_offsets: HashMap<EditorTerrain, f32>,
    /// Transition bias per terrain pair.
    #[serde(default, with = "map_as_vec")]
    pub transition_biases: HashMap<EdgeKey, f32>,
}

/// Serialize/deserialize a HashMap as a Vec of (key, value) pairs,
/// since JSON requires string keys.
mod map_as_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::HashMap;
    use std::hash::Hash;

    pub fn serialize<S, K, V>(map: &HashMap<K, V>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        K: Serialize,
        V: Serialize,
    {
        let vec: Vec<(&K, &V)> = map.iter().collect();
        vec.serialize(ser)
    }

    pub fn deserialize<'de, D, K, V>(de: D) -> Result<HashMap<K, V>, D::Error>
    where
        D: Deserializer<'de>,
        K: Deserialize<'de> + Eq + Hash,
        V: Deserialize<'de>,
    {
        let vec: Vec<(K, V)> = Vec::deserialize(de)?;
        Ok(vec.into_iter().collect())
    }
}

impl MapModel {
    /// Initialize edge profiles with a sharp step transition.
    ///
    /// The profile holds at h1 for the first ~40%, transitions steeply
    /// over the middle ~20%, then holds at h2 for the last ~40%.
    /// This gives an extruded cliff look rather than a smooth ramp.
    pub fn init_edge_profiles(&mut self) {
        for &t1 in &EditorTerrain::ALL {
            for &t2 in &EditorTerrain::ALL {
                let key = EdgeKey::new(t1, t2);
                self.edge_profiles.entry(key).or_insert_with(|| {
                    let h1 = key.0.height();
                    let h2 = key.1.height();
                    (0..=N)
                        .map(|i| {
                            let f = i as f32 / N as f32;
                            // Remap to a steep transition in the middle.
                            // Hold at h1 for f < 0.35, hold at h2 for f > 0.65,
                            // steep linear ramp between 0.35 and 0.65.
                            let s = ((f - 0.35) / 0.3).clamp(0.0, 1.0);
                            // Apply smoothstep to the remapped value for
                            // a slight ease at the cliff edges.
                            let s = s * s * (3.0 - 2.0 * s);
                            h1 * (1.0 - s) + h2 * s
                        })
                        .collect()
                });
            }
        }
    }

    /// Initialize all interior profiles with dominant-terrain heights.
    pub fn init_interiors(&mut self) {
        let terrains = &EditorTerrain::ALL;
        for (ai, &ta) in terrains.iter().enumerate() {
            for (bi, &tb) in terrains.iter().enumerate() {
                if bi < ai {
                    continue;
                }
                for &tc in &terrains[bi..] {
                    let key = TripleKey::new(ta, tb, tc);
                    self.interior_profiles.entry(key).or_insert_with(|| {
                        let heights = [key.0.height(), key.1.height(), key.2.height()];
                        let mut pts = Vec::new();
                        for i in 0..=N {
                            for j in 0..=(N - i) {
                                let k = N - i - j;
                                if i == 0 || j == 0 || k == 0 {
                                    continue;
                                }
                                let wa = i as f32 / N as f32;
                                let wb = j as f32 / N as f32;
                                let wc = k as f32 / N as f32;
                                let dom = if wa >= wb && wa >= wc {
                                    0
                                } else if wb >= wc {
                                    1
                                } else {
                                    2
                                };
                                pts.push(InteriorPoint {
                                    i,
                                    j,
                                    k,
                                    z: heights[dom],
                                    dlx: 0.0,
                                    dly: 0.0,
                                });
                            }
                        }
                        pts
                    });
                }
            }
        }
    }

    /// Get the transition bias for a terrain pair (default 0.5).
    pub fn bias(&self, a: EditorTerrain, b: EditorTerrain) -> f32 {
        *self.transition_biases.get(&EdgeKey::new(a, b)).unwrap_or(&0.5)
    }

    /// Determine the dominant terrain for a sub-face at barycentric weights
    /// (wa, wb, wc) for terrains (ta, tb, tc), taking transition biases
    /// into account.
    pub fn biased_dominant(
        &self,
        wa: f32,
        wb: f32,
        wc: f32,
        ta: EditorTerrain,
        tb: EditorTerrain,
        tc: EditorTerrain,
    ) -> usize {
        let bias_ab = self.bias(ta, tb);
        let bias_bc = self.bias(tb, tc);
        let bias_ac = self.bias(ta, tc);

        // Shift weights based on biases.
        // bias < 0.5 → the lesser terrain (first in EdgeKey) wins more area.
        // bias > 0.5 → the greater terrain wins more area.
        // We determine direction based on which terrain is "first" in the key.
        let mut aw = wa;
        let mut bw = wb;
        let mut cw = wc;

        // ta vs tb
        let shift_ab = (0.5 - bias_ab) * 0.6;
        if ta <= tb { aw += shift_ab; bw -= shift_ab; }
        else        { bw += shift_ab; aw -= shift_ab; }

        // tb vs tc
        let shift_bc = (0.5 - bias_bc) * 0.6;
        if tb <= tc { bw += shift_bc; cw -= shift_bc; }
        else        { cw += shift_bc; bw -= shift_bc; }

        // ta vs tc
        let shift_ac = (0.5 - bias_ac) * 0.6;
        if ta <= tc { aw += shift_ac; cw -= shift_ac; }
        else        { cw += shift_ac; aw -= shift_ac; }

        if aw >= bw && aw >= cw { 0 }
        else if bw >= cw { 1 }
        else { 2 }
    }

    /// Ensure all needed profiles exist for the current set of triangles.
    pub fn ensure_profiles(&mut self) {
        self.init_edge_profiles();
        self.init_interiors();
    }

    /// Get the height offset for a terrain type's corners.
    pub fn terrain_offset(&self, t: EditorTerrain) -> f32 {
        self.terrain_height_offsets.get(&t).copied().unwrap_or(0.0)
    }

    /// Get the local-space XY offset for an interior vertex, or (0, 0).
    pub fn interior_local_offset(
        &self,
        ta: EditorTerrain,
        tb: EditorTerrain,
        tc: EditorTerrain,
        i: usize,
        j: usize,
        k: usize,
    ) -> (f32, f32) {
        let key = TripleKey::new(ta, tb, tc);
        if let Some(pts) = self.interior_profiles.get(&key) {
            if let Some(p) = pts.iter().find(|p| p.i == i && p.j == j && p.k == k) {
                return (p.dlx, p.dly);
            }
        }
        (0.0, 0.0)
    }

    /// Classify a subdivision vertex within a canonical triple (ta <= tb <= tc).
    ///
    /// `cell_a`, `cell_b`, `cell_c` are the cell-centre positions for ta, tb, tc
    /// respectively.  They are needed to break ties when two adjacent terrains
    /// are the same type (e.g. Open–Open) so that the edge parameterisation is
    /// consistent across all triangles sharing that Delaunay edge.
    /// `cell_idx_a`, `cell_idx_b`, `cell_idx_c` are the indices into the
    /// `cell_centers` array for the three corners (ta, tb, tc).
    pub fn classify_vertex(
        ta: EditorTerrain,
        tb: EditorTerrain,
        tc: EditorTerrain,
        cell_a: super::voronoi::Point2,
        cell_b: super::voronoi::Point2,
        cell_c: super::voronoi::Point2,
        _cell_idx_a: usize,
        _cell_idx_b: usize,
        _cell_idx_c: usize,
        i: usize,
        j: usize,
        k: usize,
    ) -> VertexClassification {
        if i == N {
            return VertexClassification::Corner { terrain: ta };
        }
        if j == N {
            return VertexClassification::Corner { terrain: tb };
        }
        if k == N {
            return VertexClassification::Corner { terrain: tc };
        }
        if k == 0 {
            let ek = EdgeKey::new(ta, tb);
            let raw = j; // raw param along the ta→tb direction
            let idx = edge_param_idx(ta, tb, cell_a, cell_b, raw);
            return VertexClassification::Edge {
                key: ek,
                param_idx: idx,
            };
        }
        if i == 0 {
            let ek = EdgeKey::new(tb, tc);
            let raw = k; // raw param along the tb→tc direction
            let idx = edge_param_idx(tb, tc, cell_b, cell_c, raw);
            return VertexClassification::Edge {
                key: ek,
                param_idx: idx,
            };
        }
        if j == 0 {
            let ek = EdgeKey::new(ta, tc);
            let raw = k; // raw param along the ta→tc direction
            let idx = edge_param_idx(ta, tc, cell_a, cell_c, raw);
            return VertexClassification::Edge {
                key: ek,
                param_idx: idx,
            };
        }
        VertexClassification::Interior {
            key: TripleKey::new(ta, tb, tc),
            i,
            j,
            k,
        }
    }

    /// Get the height of a subdivision vertex from profiles.
    ///
    /// `cell_a`, `cell_b`, `cell_c` are the cell positions for ta, tb, tc.
    /// `cell_idx_a`, `cell_idx_b`, `cell_idx_c` are their indices into cell_centers.
    pub fn vertex_height(
        &self,
        ta: EditorTerrain,
        tb: EditorTerrain,
        tc: EditorTerrain,
        cell_a: super::voronoi::Point2,
        cell_b: super::voronoi::Point2,
        cell_c: super::voronoi::Point2,
        _cell_idx_a: usize,
        _cell_idx_b: usize,
        _cell_idx_c: usize,
        i: usize,
        j: usize,
        k: usize,
    ) -> f32 {
        if i == N {
            return ta.height() + self.terrain_offset(ta);
        }
        if j == N {
            return tb.height() + self.terrain_offset(tb);
        }
        if k == N {
            return tc.height() + self.terrain_offset(tc);
        }
        // Edge k==0: edge between ta and tb
        if k == 0 {
            let ek = EdgeKey::new(ta, tb);
            if let Some(prof) = self.edge_profiles.get(&ek) {
                let idx = edge_param_idx(ta, tb, cell_a, cell_b, j);
                return prof[idx.min(N)];
            }
        }
        // Edge i==0: edge between tb and tc
        if i == 0 {
            let ek = EdgeKey::new(tb, tc);
            if let Some(prof) = self.edge_profiles.get(&ek) {
                let idx = edge_param_idx(tb, tc, cell_b, cell_c, k);
                return prof[idx.min(N)];
            }
        }
        // Edge j==0: edge between ta and tc
        if j == 0 {
            let ek = EdgeKey::new(ta, tc);
            if let Some(prof) = self.edge_profiles.get(&ek) {
                let idx = edge_param_idx(ta, tc, cell_a, cell_c, k);
                return prof[idx.min(N)];
            }
        }
        // Interior
        let key = TripleKey::new(ta, tb, tc);
        if let Some(int_pts) = self.interior_profiles.get(&key) {
            if let Some(found) = int_pts.iter().find(|p| p.i == i && p.j == j && p.k == k) {
                return found.z;
            }
        }
        // Fallback: dominant terrain height
        let wa = i as f32 / N as f32;
        let wb = j as f32 / N as f32;
        let wc = k as f32 / N as f32;
        let dom = if wa >= wb && wa >= wc {
            0
        } else if wb >= wc {
            1
        } else {
            2
        };
        [ta.height(), tb.height(), tc.height()][dom]
    }
}
