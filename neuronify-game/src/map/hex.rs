//! Hex grid coordinate utilities for the scenario map.
//!
//! The SVG terrain uses pointy-top hexagons in odd-row offset layout.
//! These functions convert between SVG pixel coordinates, world (ECS) coordinates,
//! and hex grid (col, row) coordinates.

// ── Map scale constants ───────────────────────────────────────────────────────

/// SVG viewBox centre X (half of 1116 width).
pub const SVG_CX: f32 = 558.0;
/// SVG viewBox centre Y (half of 812 height).
pub const SVG_CY: f32 = 406.0;
/// SVG-pixels-to-world-units scale factor.
pub const MAP_SCALE: f32 = 0.28;

// ── Hex geometry constants ────────────────────────────────────────────────────

/// Hex cell radius in SVG pixels.
const HEX_RADIUS: f32 = 24.0;
/// Horizontal distance between adjacent hex centres (pointy-top). ≈ 41.569
const HEX_W: f32 = 1.732_050_8 * HEX_RADIUS;
/// Vertical distance between adjacent hex centres (pointy-top, odd-r offset). = 36.0
const ROW_H: f32 = 1.5 * HEX_RADIUS;
/// SVG X coordinate of the centre of hex (col=0, row=0).
const MARGIN_X: f32 = 80.0;
/// SVG Y coordinate of the centre of hex (col=0, row=0).
const MARGIN_Y: f32 = 70.0;

// ── Coordinate type ───────────────────────────────────────────────────────────

/// Hex grid position in odd-row offset (col, row) coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HexCoord {
    pub col: i32,
    pub row: i32,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return the SVG pixel centre of hex cell `(col, row)` in odd-row offset layout.
///
/// Pointy-top hexagons: odd rows are shifted right by `HEX_W / 2`.
pub fn hex_center(col: i32, row: i32) -> (f32, f32) {
    let offset = if row & 1 != 0 { HEX_W * 0.5 } else { 0.0 };
    let x = MARGIN_X + col as f32 * HEX_W + offset;
    let y = MARGIN_Y + row as f32 * ROW_H;
    (x, y)
}

/// Convert a hex cell `(col, row)` to its world-space centre position (y = 0).
///
/// Composes `hex_center` (→ SVG pixels) with the SVG→world transform.
pub fn hex_to_world(col: i32, row: i32) -> glam::Vec3 {
    let (sx, sy) = hex_center(col, row);
    glam::Vec3::new(-(sx - SVG_CX) * MAP_SCALE, 0.0, -(sy - SVG_CY) * MAP_SCALE)
}

/// Convert a world-space position (ECS x/z) to the nearest hex cell.
///
/// Inverts the SVG→world transform, then snaps to the closest hex centre.
pub fn world_to_hex(world_x: f32, world_z: f32) -> HexCoord {
    // Inverse of: world_x = -(svg_x - SVG_CX) * MAP_SCALE
    let svg_x = SVG_CX - world_x / MAP_SCALE;
    let svg_y = SVG_CY - world_z / MAP_SCALE;

    // Snap to row first, then column (odd-row offset requires knowing parity).
    let row = ((svg_y - MARGIN_Y) / ROW_H).round() as i32;
    let offset = if row & 1 != 0 { HEX_W * 0.5 } else { 0.0 };
    let col = ((svg_x - MARGIN_X - offset) / HEX_W).round() as i32;

    HexCoord { col, row }
}

/// Return the 6 offset-coordinate neighbours of hex cell `(col, row)`.
///
/// Uses odd-row offset conventions: odd rows are shifted right by half a cell.
pub fn hex_neighbors(col: i32, row: i32) -> [HexCoord; 6] {
    if row & 1 == 0 {
        // Even row — neighbouring odd rows are shifted right, so lower/upper
        // diagonal neighbours have the same col.
        [
            HexCoord { col: col + 1, row },              // E
            HexCoord { col: col - 1, row },              // W
            HexCoord { col,          row: row - 1 },     // NE
            HexCoord { col: col - 1, row: row - 1 },     // NW
            HexCoord { col,          row: row + 1 },     // SE
            HexCoord { col: col - 1, row: row + 1 },     // SW
        ]
    } else {
        // Odd row — neighbouring even rows are shifted left, so lower/upper
        // diagonal neighbours have col + 1.
        [
            HexCoord { col: col + 1, row },              // E
            HexCoord { col: col - 1, row },              // W
            HexCoord { col: col + 1, row: row - 1 },     // NE
            HexCoord { col,          row: row - 1 },     // NW
            HexCoord { col: col + 1, row: row + 1 },     // SE
            HexCoord { col,          row: row + 1 },     // SW
        ]
    }
}

/// Hex distance between two cells using cube-coordinate metric.
pub fn hex_distance(a: HexCoord, b: HexCoord) -> i32 {
    let (aq, ar, as_) = offset_to_cube(a.col, a.row);
    let (bq, br, bs) = offset_to_cube(b.col, b.row);
    ((aq - bq).abs() + (ar - br).abs() + (as_ - bs).abs()) / 2
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Convert odd-r offset `(col, row)` to cube coordinates `(q, r, s)`.
fn offset_to_cube(col: i32, row: i32) -> (i32, i32, i32) {
    let q = col - (row - (row & 1)) / 2;
    let r = row;
    let s = -q - r;
    (q, r, s)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── hex_center ────────────────────────────────────────────────────────────

    #[test]
    fn test_hex_center_origin_neuron() {
        // p_origin: col=3, row=15, data-x≈225.5, data-y=610
        let (x, y) = hex_center(3, 15);
        assert!((x - 225.5).abs() < 0.5, "x={x} expected ~225.5");
        assert!((y - 610.0).abs() < 0.5, "y={y} expected 610.0");
    }

    #[test]
    fn test_hex_center_e1() {
        // e1: col=18, row=2, data-x≈828.2, data-y=142
        let (x, y) = hex_center(18, 2);
        assert!((x - 828.2).abs() < 0.5, "x={x} expected ~828.2");
        assert!((y - 142.0).abs() < 0.5, "y={y} expected 142.0");
    }

    #[test]
    fn test_hex_center_even_row_spacing() {
        // Even-row columns are separated by exactly HEX_W.
        let (x0, _) = hex_center(0, 0);
        let (x1, _) = hex_center(1, 0);
        assert!((x1 - x0 - HEX_W).abs() < 0.01);
    }

    #[test]
    fn test_hex_center_odd_row_shift() {
        // Odd-row col 0 is shifted right by HEX_W/2 relative to even-row col 0.
        let (x_even, _) = hex_center(0, 0);
        let (x_odd, _)  = hex_center(0, 1);
        assert!((x_odd - x_even - HEX_W * 0.5).abs() < 0.01);
    }

    // ── world_to_hex ──────────────────────────────────────────────────────────

    #[test]
    fn test_world_to_hex_origin_neuron() {
        // p_origin SVG (225.5, 610) → world using svg_to_world transform.
        let wx = -(225.5 - SVG_CX) * MAP_SCALE;
        let wz = -(610.0 - SVG_CY) * MAP_SCALE;
        let h = world_to_hex(wx, wz);
        assert_eq!(h.col, 3,  "col: got {}", h.col);
        assert_eq!(h.row, 15, "row: got {}", h.row);
    }

    #[test]
    fn test_world_to_hex_e1() {
        // e1 SVG (828.2, 142).
        let wx = -(828.2 - SVG_CX) * MAP_SCALE;
        let wz = -(142.0 - SVG_CY) * MAP_SCALE;
        let h = world_to_hex(wx, wz);
        assert_eq!(h.col, 18);
        assert_eq!(h.row, 2);
    }

    #[test]
    fn test_world_to_hex_roundtrip() {
        // hex_center → world → world_to_hex should recover the original cell.
        for row in 0..18i32 {
            for col in 0..22i32 {
                let (sx, sy) = hex_center(col, row);
                let wx = -(sx - SVG_CX) * MAP_SCALE;
                let wz = -(sy - SVG_CY) * MAP_SCALE;
                let h = world_to_hex(wx, wz);
                assert_eq!(
                    h,
                    HexCoord { col, row },
                    "roundtrip failed for ({col},{row}): got ({},{})",
                    h.col, h.row,
                );
            }
        }
    }

    // ── hex_neighbors ─────────────────────────────────────────────────────────

    #[test]
    fn test_hex_neighbors_count() {
        assert_eq!(hex_neighbors(5, 5).len(), 6);
    }

    #[test]
    fn test_hex_neighbors_unique() {
        let neighbors = hex_neighbors(10, 7);
        let mut seen = std::collections::HashSet::new();
        for nb in neighbors {
            assert!(seen.insert(nb), "duplicate neighbour {:?}", nb);
        }
    }

    #[test]
    fn test_hex_neighbors_symmetry() {
        // For any cell A and each of its neighbours N, A must appear in N's neighbours.
        let center = HexCoord { col: 5, row: 4 };
        for nb in hex_neighbors(center.col, center.row) {
            let back = hex_neighbors(nb.col, nb.row);
            assert!(
                back.contains(&center),
                "neighbour {nb:?} does not list {center:?} as a neighbour"
            );
        }
    }

    // ── hex_distance ──────────────────────────────────────────────────────────

    #[test]
    fn test_hex_distance_self() {
        let h = HexCoord { col: 5, row: 5 };
        assert_eq!(hex_distance(h, h), 0);
    }

    #[test]
    fn test_hex_distance_adjacent() {
        let a = HexCoord { col: 5, row: 5 };
        for nb in hex_neighbors(a.col, a.row) {
            assert_eq!(hex_distance(a, nb), 1, "neighbour {nb:?} should be distance 1");
        }
    }

    #[test]
    fn test_hex_distance_known() {
        // p_origin (3,15) to e1 (18,2): large diagonal crossing most of the map.
        let p_origin = HexCoord { col: 3,  row: 15 };
        let e1       = HexCoord { col: 18, row: 2  };
        let d = hex_distance(p_origin, e1);
        assert!(d > 10 && d < 30, "distance {d} out of expected range 10..30");
    }

    #[test]
    fn test_hex_distance_symmetric() {
        let a = HexCoord { col: 3,  row: 15 };
        let b = HexCoord { col: 18, row: 2  };
        assert_eq!(hex_distance(a, b), hex_distance(b, a));
    }
}
