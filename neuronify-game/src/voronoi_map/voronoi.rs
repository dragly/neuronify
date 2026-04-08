//! Poisson disk sampling, Lloyd relaxation, and Bowyer-Watson Delaunay triangulation.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl Point2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn dist_sq(self, other: Self) -> f32 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
}

// ── Seeded RNG (same LCG as the JSX version) ────────────────────────────────

struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_f32(&mut self) -> f32 {
        self.state = (self.state.wrapping_mul(16807)) % 2147483647;
        self.state as f32 / 2147483647.0
    }
}

// ── Poisson disk sampling (Bridson's algorithm) ──────────────────────────────

pub fn poisson_disk(w: f32, h: f32, r: f32, seed: u64) -> Vec<Point2> {
    let mut rng = Rng::new(seed);
    let cs = r / std::f32::consts::SQRT_2;
    let cols = (w / cs).ceil() as usize;
    let rows = (h / cs).ceil() as usize;
    let mut grid: Vec<Option<Point2>> = vec![None; cols * rows];
    let mut pts = Vec::new();
    let mut active = Vec::new();

    let gidx = |p: Point2| -> usize {
        let gc = (p.x / cs) as usize;
        let gr = (p.y / cs) as usize;
        gc.min(cols - 1) + gr.min(rows - 1) * cols
    };

    let fits = |p: Point2, grid: &[Option<Point2>]| -> bool {
        if p.x < 0.0 || p.x >= w || p.y < 0.0 || p.y >= h {
            return false;
        }
        let gc = (p.x / cs) as i32;
        let gr = (p.y / cs) as i32;
        for dc in -2..=2 {
            for dr in -2..=2 {
                let nc = gc + dc;
                let nr = gr + dr;
                if nc >= 0 && nc < cols as i32 && nr >= 0 && nr < rows as i32 {
                    if let Some(g) = grid[nc as usize + nr as usize * cols] {
                        if g.dist_sq(p) < r * r {
                            return false;
                        }
                    }
                }
            }
        }
        true
    };

    let p0 = Point2::new(w / 2.0, h / 2.0);
    pts.push(p0);
    grid[gidx(p0)] = Some(p0);
    active.push(p0);

    while !active.is_empty() {
        let idx = (rng.next_f32() * active.len() as f32) as usize;
        let idx = idx.min(active.len() - 1);
        let p = active[idx];
        let mut found = false;
        for _ in 0..30 {
            let a = rng.next_f32() * 2.0 * std::f32::consts::PI;
            let d = r + rng.next_f32() * r;
            let c = Point2::new(p.x + d * a.cos(), p.y + d * a.sin());
            if fits(c, &grid) {
                pts.push(c);
                grid[gidx(c)] = Some(c);
                active.push(c);
                found = true;
                break;
            }
        }
        if !found {
            active.swap_remove(idx);
        }
    }
    pts
}

// ── Circumcircle ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
struct Circumcircle {
    x: f64,
    y: f64,
    r2: f64,
}

fn circumcircle(p0: Point2, p1: Point2, p2: Point2) -> Circumcircle {
    let (ax, ay) = (p0.x as f64, p0.y as f64);
    let (bx, by) = (p1.x as f64, p1.y as f64);
    let (cx, cy) = (p2.x as f64, p2.y as f64);
    let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
    if d.abs() < 1e-10 {
        return Circumcircle {
            x: 0.0,
            y: 0.0,
            r2: f64::INFINITY,
        };
    }
    let ux = ((ax * ax + ay * ay) * (by - cy)
        + (bx * bx + by * by) * (cy - ay)
        + (cx * cx + cy * cy) * (ay - by))
        / d;
    let uy = ((ax * ax + ay * ay) * (cx - bx)
        + (bx * bx + by * by) * (ax - cx)
        + (cx * cx + cy * cy) * (bx - ax))
        / d;
    Circumcircle {
        x: ux,
        y: uy,
        r2: (ax - ux).powi(2) + (ay - uy).powi(2),
    }
}

// ── Bowyer-Watson Delaunay triangulation ─────────────────────────────────────

/// Returns triangles as `[a, b, c]` indices into `pts`.
pub fn delaunay(pts: &[Point2], w: f32, h: f32) -> Vec<[usize; 3]> {
    let margin = w.max(h) * 10.0;
    let super_pts = [
        Point2::new(-margin, -margin),
        Point2::new(3.0 * margin, -margin),
        Point2::new(0.0, 3.0 * margin),
    ];

    let n = pts.len();
    let si0 = n;
    let si1 = n + 1;
    let si2 = n + 2;

    let all_pts: Vec<Point2> = pts.iter().copied().chain(super_pts.iter().copied()).collect();

    let mut triangles: Vec<[usize; 3]> = vec![[si0, si1, si2]];
    let mut cc: Vec<Circumcircle> = vec![circumcircle(all_pts[si0], all_pts[si1], all_pts[si2])];

    for pi in 0..n {
        let p = pts[pi];
        let px = p.x as f64;
        let py = p.y as f64;

        let mut bad = Vec::new();
        for ti in 0..triangles.len() {
            let c = cc[ti];
            if (px - c.x).powi(2) + (py - c.y).powi(2) < c.r2 {
                bad.push(ti);
            }
        }

        // Find boundary edges of the hole.
        let mut edges = Vec::new();
        for &ti in &bad {
            let t = triangles[ti];
            for e in 0..3 {
                let a = t[e];
                let b = t[(e + 1) % 3];
                let shared = bad.iter().any(|&tj| {
                    if tj == ti {
                        return false;
                    }
                    let u = triangles[tj];
                    u.contains(&a) && u.contains(&b)
                });
                if !shared {
                    edges.push([a, b]);
                }
            }
        }

        // Remove bad triangles.
        let bad_set: std::collections::HashSet<usize> = bad.into_iter().collect();
        let mut new_tris = Vec::new();
        let mut new_cc = Vec::new();
        for ti in 0..triangles.len() {
            if !bad_set.contains(&ti) {
                new_tris.push(triangles[ti]);
                new_cc.push(cc[ti]);
            }
        }

        // Create new triangles from boundary edges to new point.
        for [a, b] in &edges {
            let t = [*a, *b, pi];
            new_cc.push(circumcircle(all_pts[*a], all_pts[*b], all_pts[pi]));
            new_tris.push(t);
        }

        triangles = new_tris;
        cc = new_cc;
    }

    // Remove triangles referencing super-triangle vertices.
    triangles
        .into_iter()
        .filter(|t| t[0] < n && t[1] < n && t[2] < n)
        .collect()
}

// ── Lloyd relaxation ─────────────────────────────────────────────────────────

pub fn lloyd_relax(pts: &[Point2], w: f32, h: f32, iters: usize) -> Vec<Point2> {
    let mut pts = pts.to_vec();
    for _ in 0..iters {
        let tris = delaunay(&pts, w, h);
        let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); pts.len()];
        for t in &tris {
            for i in 0..3 {
                for j in (i + 1)..3 {
                    let a = t[i];
                    let b = t[j];
                    if a < pts.len() && b < pts.len() {
                        neighbors[a].push(b);
                        neighbors[b].push(a);
                    }
                }
            }
        }
        let new_pts: Vec<Point2> = pts
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let ns = &neighbors[i];
                if ns.is_empty() {
                    return *p;
                }
                let mut sx = 0.0f32;
                let mut sy = 0.0f32;
                for &ni in ns {
                    sx += pts[ni].x;
                    sy += pts[ni].y;
                }
                let mx = sx / ns.len() as f32;
                let my = sy / ns.len() as f32;
                Point2::new(
                    (p.x * 0.5 + mx * 0.5).clamp(2.0, w - 2.0),
                    (p.y * 0.5 + my * 0.5).clamp(2.0, h - 2.0),
                )
            })
            .collect();
        pts = new_pts;
    }
    pts
}

/// Compute Voronoi polygon vertices for each cell from Delaunay circumcenters.
/// Returns one polygon per cell (Vec of Point2 vertices in winding order),
/// or None for cells with fewer than 3 adjacent triangles.
pub fn voronoi_polygons(
    pts: &[Point2],
    triangles: &[[usize; 3]],
    w: f32,
    h: f32,
) -> Vec<Option<Vec<Point2>>> {
    let n = pts.len();
    let mut cell_tris: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut tri_cc: Vec<Point2> = Vec::with_capacity(triangles.len());

    for (ti, t) in triangles.iter().enumerate() {
        let cc = circumcircle(pts[t[0]], pts[t[1]], pts[t[2]]);
        tri_cc.push(Point2::new(cc.x as f32, cc.y as f32));
        for &vi in t {
            if vi < n {
                cell_tris[vi].push(ti);
            }
        }
    }

    pts.iter()
        .enumerate()
        .map(|(ci, cell)| {
            let tri_indices = &cell_tris[ci];
            if tri_indices.len() < 3 {
                return None;
            }
            let cx = cell.x;
            let cy = cell.y;
            let mut vertices: Vec<(f32, f32, f32)> = tri_indices
                .iter()
                .map(|&ti| {
                    let v = tri_cc[ti];
                    let angle = (v.y - cy).atan2(v.x - cx);
                    (v.x, v.y, angle)
                })
                .collect();
            vertices.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
            Some(
                vertices
                    .into_iter()
                    .map(|(x, y, _)| {
                        Point2::new(x.clamp(-5.0, w + 5.0), y.clamp(-5.0, h + 5.0))
                    })
                    .collect(),
            )
        })
        .collect()
}
