# Task 9: Scenario Testing Framework

Create a headless test that loads the scenario SVG, runs the simulation for a defined number of ticks, and verifies expected outcomes.

## Tests to implement

1. **Parse test:** Load SVG, verify entity counts and terrain grid dimensions match expected values
2. **Terrain test:** Verify no entity is placed on impassable terrain
3. **Pathfinding test:** Find path from (3,15) to (18,2), verify it avoids all impassable hexes
4. **Signal propagation test:** Force-fire the player origin neuron, run 200 ticks, verify downstream neurons fire
5. **Macrophage activation test:** Verify macrophage is active when driver fires, dormant when driver is in depol block
6. **Victory test:** Kill all three target neurons, verify victory condition triggers

Run these tests on every code change that touches the simulation or map loading systems.
