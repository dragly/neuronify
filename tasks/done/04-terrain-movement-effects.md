# Task 4: Terrain Movement Effects

When any entity moves (neuroblast, growth cone, mobile unit, macrophage), the terrain at its position affects its speed.

## Terrain rules

| Terrain | Passable | Speed Multiplier | Axon Growth Mult | Notes |
|---------|----------|------------------|------------------|-------|
| `open` | yes | 1.0 | 1.0 | Normal |
| `ecm_sparse` | yes | 0.7 | 0.7 | Mild slowdown |
| `ecm_dense` | yes | 0.4 | 0.3 | Major slowdown |
| `csf` | no | — | — | Impassable |
| `glial_scar` | no | — | — | Impassable |
| `vessel` | no | — | — | Impassable + resource |

## Implementation

Before moving any entity to a new position:
1. Convert target position to hex coordinates using `worldToHex`
2. Look up terrain at that hex
3. If `passable === false`, block the movement (entity stays at current position)
4. If passable, multiply entity speed by the terrain's `speedMult`

For axon growth cones, also apply the growth multiplier column.

## Pathfinding

Implement hex-grid A* for neuroblast migration and mobile unit movement:
- Nodes are hex cells
- A cell is walkable if `passable === true`
- Edge cost = `1.0 / speedMult` (dense ECM costs more)
- Heuristic = hex distance to target
- Entities route around impassable terrain automatically

## Testing

Request a path from player base (hex 3,15) to enemy outpost (hex 18,2). Verify the path avoids all impassable hexes (CSF, scar, vessel). Verify the path prefers open terrain over dense ECM when both routes exist.
