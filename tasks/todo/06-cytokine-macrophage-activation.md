# Task 6: Cytokine-Based Macrophage Activation

Macrophages are activated by chemical signals, not direct neural connections. This is biologically accurate: in real immunology, macrophages are activated by cytokines released by other cells, not by axonal connections.

## The activation chain

1. The player (or enemy) builds a **mast cell** — a stationary effector cell that sits near the macrophage patrol zone
2. The mast cell is connected to a driver neuron by an axon
3. When the driver neuron fires, the mast cell receives the spike and **releases cytokines** — visible chemical particles that diffuse outward from the mast cell into the surrounding area
4. Macrophages within the cytokine cloud become **activated** — they turn aggressive, pursue enemies, and engulf targets
5. When the driver neuron stops firing (depolarization block, axon severed, neuron killed), the mast cell stops releasing cytokines
6. The cytokine cloud dissipates over a few seconds
7. Macrophages that are no longer in any cytokine cloud return to **dormant** state

This is biologically accurate: mast cells are real immune cells that release histamine and cytokines when activated by neural signals. They are the bridge between the nervous system and the immune system. The cytokines they release (TNF-α, interleukins) activate macrophages in the local area.

## Mast cell entity

- Stationary (does not move)
- Connected to a driver neuron via an axon
- When it receives a spike, it emits a burst of cytokine particles
- Cytokine particles diffuse outward in a radius, fading over ~3-4 seconds
- Multiple spikes = sustained cytokine cloud
- If the axon is severed, no more spikes arrive, no more cytokines, macrophages go dormant

## Cytokine particles (visual)

- Small, warm-colored particles (orange/yellow) that drift outward from the mast cell
- They should be clearly visible — the player needs to see the activation signal
- The cloud radius defines the macrophage activation zone
- Particles fade over time — the cloud is sustained only by continuous mast cell firing

## Macrophage behavior

- **Dormant:** Drifts slowly, does nothing aggressive. Neutral appearance.
- **Active (in cytokine cloud):** Moves toward nearest enemy entity within patrol radius. On contact, engulfs target (rapid metabolic drain). Visually distinct — brighter, more aggressive appearance.
- Macrophages are attracted toward higher cytokine concentration — they drift toward the center of the cloud even when not chasing a target

## How the player disables enemy macrophages

The player does NOT attack the macrophage directly. Instead:
1. Trace the cytokine cloud back to the mast cell
2. Trace the axon from the mast cell back to its driver neuron
3. Silence the driver neuron (via convergent excitatory bombardment → depol block, or via inhibitory projection, or by severing the axon)
4. The mast cell stops receiving spikes → stops releasing cytokines
5. The cytokine cloud dissipates
6. The macrophage goes dormant

This preserves the core lesson: attack the neural circuit to disable the effector.

## Updating the scenario map

The existing SVG has `data-driver-id` on macrophages pointing directly to neurons. For this implementation, the `data-driver-id` should be interpreted as: "the neuron that drives the mast cell that activates this macrophage." The mast cell itself should be spawned at a position between the driver neuron and the macrophage, connected to the driver by an axon. The macrophage does NOT have an axon connection — it responds to the chemical environment.

## Testing

1. Verify mast cell emits cytokine particles when its driver fires
2. Verify macrophage activates when inside the cytokine cloud
3. Silence the driver neuron (set depol block) → verify cytokines stop → verify macrophage goes dormant within 3-4 seconds
4. Sever the axon between driver and mast cell → same dormancy result
5. Verify cytokine particles are visible and clearly show the activation zone
