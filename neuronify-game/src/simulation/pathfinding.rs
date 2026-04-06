use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;

use hecs::Entity;

/// Axial hex coordinate.
pub type HexCoord = (i32, i32);

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
    pub fn a_star(&self, mover: Entity, start: HexCoord, goal: HexCoord) -> Vec<HexCoord> {
        // Cost type: u32 avoids float hashing issues.
        // Priority queue: (f, g, coord)  — min-heap on f via Reverse.
        let mut open: BinaryHeap<(Reverse<i32>, i32, HexCoord)> = BinaryHeap::new();
        let mut g_score: HashMap<HexCoord, i32> = HashMap::new();
        let mut came_from: HashMap<HexCoord, HexCoord> = HashMap::new();

        g_score.insert(start, 0);
        open.push((Reverse(Self::hex_distance(start, goal)), 0, start));

        while let Some((_, g, current)) = open.pop() {
            if current == goal {
                // Reconstruct path
                let mut path = Vec::new();
                let mut c = current;
                while c != start {
                    path.push(c);
                    c = came_from[&c];
                }
                path.reverse();
                return path;
            }

            // Skip if we already found a cheaper route to `current`.
            if g_score.get(&current).copied().unwrap_or(i32::MAX) < g {
                continue;
            }

            for &nb in Self::neighbours(current).iter() {
                if !self.is_passable(nb, mover) && nb != goal {
                    continue;
                }
                let tentative_g = g + 1;
                if tentative_g < g_score.get(&nb).copied().unwrap_or(i32::MAX) {
                    g_score.insert(nb, tentative_g);
                    came_from.insert(nb, current);
                    let f = tentative_g + Self::hex_distance(nb, goal);
                    open.push((Reverse(f), tentative_g, nb));
                }
            }
        }

        // No path found
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
