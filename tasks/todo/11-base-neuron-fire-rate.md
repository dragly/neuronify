# Task 11 — Base Neuron Fire Rate

## Problem

The origin/base neuron fires too slowly. It should fire 5× more often than its current rate.

## Acceptance Criteria

- The `RegularSpikeGenerator` frequency on the origin neuron in `setup_scenario_from_svg` (and any other place the base neuron is spawned, e.g. `setup_game`) is multiplied by 5.
- Existing simulation tests still pass.
