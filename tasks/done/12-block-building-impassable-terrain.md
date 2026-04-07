# Task 12 — Block Axon Building Through Impassable Terrain

## Problem

When the player uses the Axon tool to draw a connection, growth cones that reach impassable terrain (vessels, CSF, glial scars) get stuck with speed multiplier 0.0 and never advance. The player receives no feedback and cannot recover.

## Solution

1. During connection drawing (waypoint painting), check whether each new waypoint would land on impassable terrain. If it would, reject the waypoint — the connection stops at the border of the last passable hex.
2. On growth cone advancement, if the next step would enter impassable terrain, despawn the growth cone instead of letting it sit stuck at speed 0.

## Acceptance Criteria

- Painting an axon connection into impassable terrain stops the line at the passable/impassable border.
- Growth cones that encounter impassable terrain on their path are despawned cleanly.
- Existing tests still pass.
