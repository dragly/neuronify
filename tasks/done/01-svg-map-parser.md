# Task 1: SVG Map Parser

Write a map loader that parses the scenario SVG file and returns a structured game state.

## What to parse

**Scenario metadata:** Find the `<g data-scenario="...">` element. Extract `data-name`, `data-objective`, `data-victory`, `data-briefing`.

**Hex terrain:** Find all `<polygon data-terrain="...">` elements. Each has:
- `data-col`, `data-row` — hex grid coordinates
- `data-terrain` — one of: `open`, `ecm_sparse`, `ecm_dense`, `csf`, `glial_scar`, `vessel`
- `data-passable` — `"true"` or `"false"`
- `data-speed-mult` — movement speed multiplier (only on passable terrain)
- `data-resource` — `"glucose"` if this hex is a resource source (only on `vessel` hexes)

Store terrain as a lookup by `(col, row)`.

**Neurons:** Find all `<g data-entity="neuron">` elements. Extract: `data-id`, `data-team`, `data-type`, `data-x`, `data-y`, `data-col`, `data-row`, `data-auto-fire`, `data-fire-hz`, `data-is-origin`.

**Axons:** Find all `<g data-entity="axon">` elements. Extract: `data-id`, `data-from`, `data-to`, `data-excitatory`. Use `data-from` and `data-to` to look up neuron positions, then build the compartment chain between them using the existing `buildAxon` function.

**Macrophages:** Find all `<g data-entity="macrophage">` elements. Extract: `data-id`, `data-team`, `data-x`, `data-y`, `data-driver-id`, `data-patrol-radius`, `data-state`.

**Toxin spawners:** Find all `<g data-entity="toxin_spawner">` elements. Extract: `data-id`, `data-x`, `data-y`, `data-rate`.

## Testing

Load the SVG, print terrain grid dimensions, count of each terrain type, and list all entities. Expected: 22×18 grid, ~194 open hexes, 16 neurons, 18 axons, 2 macrophages, 1 toxin spawner.
