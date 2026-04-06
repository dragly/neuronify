# Task 7: Victory Condition Checker

The `data-victory` attribute on the scenario metadata defines the win condition.

## Format

The victory string has the format `condition_type:param1,param2,...`

For this scenario: `all_neurons_dead:e1,e2,e3`

## Implementation

Parse the victory string:
- Split on `:` to get condition type and parameters
- `all_neurons_dead`: check if ALL listed neuron IDs have `alive === false` (calcium death, starvation, or any other death cause)
- Check every second (not every tick — performance optimization)
- When met, trigger victory screen with scenario name and completion stats

## Future condition types (do not implement yet, just ensure the parser is extensible)

- `neuron_captured:id` — a neuron changed team
- `survive_time:seconds` — player network survives N seconds
- `control_vessels:count` — player has astrocytes adjacent to N vessel hexes

## Testing

Set neurons e1, e2, e3 to `alive = false` in sequence. Verify victory triggers only when ALL three are dead, not before.
