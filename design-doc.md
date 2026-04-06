# Neuronify RTS — Game Design Document

## Elevator Pitch

Two neural networks grow in a shared petri dish, competing for finite resources. Everything spawns from a single radial glial stem cell and must physically migrate through hostile soup to reach its destination. Axons grow compartment by compartment. Neurons mature in place. Your network IS your army, your base, your economy. Nothing is instant. Everything is physical. Structure emerges from biology.

## Core Concept

The battlefield is neural tissue. There is one view, one layer, one continuous canvas. Everything — building, fighting, economy, defense — happens on the same surface.

**What you control** is defined by connectivity: you own every neuron reachable through a continuous chain of axons from your radial glial cell (origin). Sever a connection and downstream neurons become inert, drifting, and vulnerable to capture. Territory is your connected component in the network graph.

**Nothing appears instantly.** Every cell is born at the radial glial base and must physically migrate through the soup to its destination. Every axon grows from a growth cone that advances compartment by compartment. Every structure takes time to build and is vulnerable during construction. This creates genuine logistics — your axon network is simultaneously your signal pathway, your supply line, and your migration highway.

---

## The World

### The Petri Dish

The map is a finite 2D surface — a petri dish with hard boundaries. Circular or oval. Two players start at opposite sides.

The dish contains:
- **Resource nodes** (glucose wells / neurotrophic factor sources) scattered across the surface. Neurons near a resource node harvest metabolic energy. Resource nodes can deplete, forcing expansion.
- **Substrate gradients** (optional, for map variety): regions with different chemical properties. High-potassium zones make neurons more excitable. High-magnesium zones weaken synaptic transmission. Neurochemical terrain.
- **Microglia** — environmental NPC immune cells that patrol the dish. They activate at damage sites (where neurons die or axons break), converge, clear debris, but cause inflammatory collateral damage to everything nearby. Neither player controls them — they're a force of nature that punishes prolonged close-quarters fighting.
- **Toxins and free radicals** — drifting hazards. Toxins depolarize unprotected neurons toward overload. Free radicals strip myelin and stress axon compartments.

### Physics of the Soup

Unconnected neurons drift passively and slowly die. Connected neurons are held in place by axon tension. The soup is hostile — unprotected cells accumulate environmental damage. Membrane walls provide shielding.

---

## The Base: Radial Glial Cell

The radial glial cell is your origin — the ★ node. It is the only cell that can produce new cells, mirroring real neural development where radial glia are the progenitor stem cells of the entire nervous system.

The radial glial cell:
- Is larger than other neurons
- Auto-fires rhythmic calcium waves that propagate through the network
- Has a **production queue** — you select what to spawn, it builds one cell at a time
- Can self-replicate (at extreme cost) to establish a second base

### Production Tree

The radial glial cell produces precursor cells that migrate to their destination and mature:

**Neuroblasts** → migrate through soup, mature into neurons (excitatory, inhibitory, adaptive, burst)
**Astrocyte precursors** → migrate, mature into astrocytes (metabolic support, area denial)
**Oligodendrocyte precursors (OPCs)** → migrate, settle on axons, myelinate them
**Ependymal cells** → migrate, form membrane barriers (living walls)

In real neural development, radial glia produce neurons first, then astrocytes, then oligodendrocytes. This suggests a natural tech tier system:
- **Tier 1** (start): neuroblasts only
- **Tier 2** (resource threshold): astrocyte precursors and ependymal cells unlocked
- **Tier 3** (higher threshold): OPCs unlocked
- **Expansion**: self-replication (new radial glial cell) — extremely expensive

---

## Cell Migration

Cells born at the radial glial cell must physically travel to their destination. In real neuroscience, neuronal migration happens through multiple mechanisms:

- **Tangential migration**: cells crawl through the extracellular matrix under their own power, guided by chemical gradients. This is how inhibitory interneurons reach their targets in real development.
- **Chemoattractant guidance**: existing neurons emit attractive signals (netrins, BDNF) that help migrating cells navigate. More friendly neurons along the route = better guidance.
- **Chemorepulsion**: enemy neurons, toxins, and semaphorin-producing cells deflect or slow migrating cells.

### Migration Mechanic

1. Spawn a precursor cell at the radial glial cell (costs energy, takes production time)
2. Designate a target position (tap where you want it to go)
3. The precursor crawls through the soup toward the target under its own power — slowly, vulnerably
4. Migrating cells are small, fragile, and exposed. Toxins damage them. Enemy bombardment kills them. Inflammation zones are lethal.
5. Having friendly neurons along the route provides chemoattractant guidance — faster, more accurate migration
6. Migrating through empty soup is possible but slower and riskier (the cell may drift off course)
7. When the precursor arrives at its destination, it begins **maturation** (a few seconds, during which it can't fire or connect)
8. Once mature, the neuron begins extending dendrites automatically and can receive/send signals

This means expansion is an investment with real logistics. You can't spam neurons at the frontier — each one has to survive a journey from your base. Protecting migration routes matters. Cutting off routes strands reinforcements.

---

## Axon Growth

Mature neurons extend axons toward targets. Axons are not instant connections — they are physical chains of **compartments** that grow from a **growth cone**.

### Growth Cone Mechanic

1. Select a mature neuron
2. Designate a target (another neuron, or empty space for a future connection point)
3. A growth cone appears at the neuron and begins advancing toward the target
4. The growth cone lays down compartments behind it at regular intervals (COMP_SPACING)
5. Growth costs energy per compartment added
6. Growth speed is ~20-30 px/s (visible, not instant)
7. The growth cone is guided by chemoattractants (target neuron pulls it) and repelled by semaphorins
8. If the growth cone is hit by a toxin or enemy signal, it collapses — the partially-grown axon remains as a stub
9. When the growth cone reaches its target, the axon is complete and signal transmission begins

### Compartmental Axon Model

Each axon is a chain of compartments connected by spring forces:
- Compartments maintain rest spacing via springs
- If any segment stretches beyond a break threshold, the axon snaps at that point
- Myelination (via oligodendrocytes) speeds signal propagation, increases break threshold, and protects against environmental damage
- Signals propagate rapidly along compartments as visible wavefronts
- Axon compartments are physical entities — they can be pushed, stressed, and broken by environmental forces and enemy attacks

### Axon Properties
- **Weight**: Determines signal strength at the terminal synapse and metabolic throughput
- **Excitatory/Inhibitory**: Matches the source neuron type
- **Myelinated**: Faster propagation, higher break threshold, requires an oligodendrocyte sitting on the axon

---

## Neuron Types

All neuron types run the same integrate-and-fire differential equations. Combat behavior emerges from neural dynamics, not artificial stats.

### Core Neurons (Tier 1)

**Excitatory Neuron** (blue)
Accumulates input current, fires when voltage crosses threshold. Spikes depolarize downstream targets. Offensive: project excitatory axons onto enemy neurons to cause excitotoxicity. Defensive: relay signals through your network.

**Inhibitory Neuron** (red)
Same dynamics, but spikes hyperpolarize targets. Defensive: suppress enemy bombardment. Strategic: enables lateral inhibition (focus on strongest threat), reciprocal inhibition (decision-making), feedback inhibition (gain control).

### Mid-Game Neurons (Tier 2+)

**Adaptive Neuron**
Spike-frequency adaptation — strong initial response that fades with sustained activity. Border sentinels that respond to new threats but ignore background noise.

**Burst Neuron**
Produces rapid spike clusters. Concentrated temporal summation overwhelms enemy inhibition. Siege weapon with a vulnerability window after each burst.

**Semaphorin Neuron**
Does not fire spikes. Instead, continuously emits a local repulsive chemical field that pushes nearby enemy axon compartments away. If the push stretches an axon segment beyond its break threshold, the axon snaps. This is the biologically grounded "wire cutter" — it doesn't physically sever anything, it creates conditions where axons tear themselves apart. Place near enemy axon bundles to disrupt their connectivity.

### Late-Game Neurons (Tier 3)

**Neuromodulatory Neuron**
Area-of-effect signal that shifts the behavior of all neurons in range. Modes: "alert" (raise thresholds), "attack" (lower thresholds), "conservation" (reduce firing rates). Superweapon tier.

---

## Support Cells

### Astrocyte (Tier 2)

Star-shaped support cell. Stationary once placed. Dual role:
- **Economic**: harvests resources from nearby resource nodes with greater efficiency than bare neurons. The primary economy backbone.
- **Protective**: buffers local neurons against excitotoxicity by clearing excess glutamate. Also continuously absorbs health from enemy mobile units that enter its radius — passive area denial.
- **Metabolic relay**: improves resource flow through nearby axons.

### Oligodendrocyte (Tier 3)

Migrates to an axon and wraps it in myelin. One oligodendrocyte can myelinate multiple nearby axon segments. Effects:
- Signal propagation speed increases ~2.5x
- Axon break threshold increases (harder to sever)
- Protects against environmental damage and free radical degradation
- If the oligodendrocyte is killed, its myelin sheaths degrade over time

### Ependymal Cell (Tier 2)

Forms membrane barriers — living walls that shield interior neurons from toxins, free radicals, and enemy repulsive signals. Unlike inert walls, ependymal cells are alive: they cost metabolic energy to maintain, can be damaged and killed, and slowly regenerate health. They're your perimeter defense.

---

## Combat

Combat emerges from neural dynamics and cell biology. No artificial damage numbers for neural interactions — mobile combat units have explicit health for clarity.

### Neural Combat (Neuron vs. Neuron)

**Excitotoxicity** — The primary offensive mechanism. Project excitatory axons onto enemy neurons and bombard them with spikes. Sustained high-voltage causes **calcium damage** — a cumulative, irreversible damage counter that increases when voltage stays high. When calcium damage crosses a lethal threshold, the neuron dies. This is biologically accurate: glutamate excitotoxicity kills neurons through sustained calcium influx.
Counter: inhibitory neurons suppress the bombardment. Astrocytes clear excess glutamate.

**Inhibitory Silencing** — Project inhibitory axons onto enemy neurons to suppress them below threshold. A silenced neuron can't fire, can't relay signals, can't drive motor systems. Functionally disconnects everything downstream without physically cutting anything.
Counter: more excitatory input to overcome inhibition, or sever the inhibitory axon.

**Seizure Induction** — Create excitatory feedback loops in the enemy network. Triggers runaway synchronized firing — metabolically catastrophic and causes rapid calcium damage to all involved neurons.
Counter: sufficient inhibitory interneurons to dampen feedback loops.

**Repulsive Axon Disruption** — Semaphorin neurons emit repulsive fields that push enemy axon compartments away, stretching them until they break.
Counter: myelinated axons resist the pull. Membrane walls block the repulsive signal.

### Mobile Unit Combat

Mobile combat units are spawned from neurons equipped with a **NeuronSpawner** component. Each time the spawner neuron fires and its cooldown has elapsed, a unit appears. Firing rate controls production rate; silencing the neuron stops the supply.

**Factions** provide three thematic skins for the same role roster:

| Role | Biological | Tech | Tumor |
|------|-----------|------|-------|
| **Resource + area control** | Astrocyte | Nano-Harvester | Metabolic Tendril |
| **Fast raider** | T-Cell | Nano-Probe | Invadopod |
| **Connection cutter** | Microglial Cell | Disruptor Drone | Severing Claw |
| **Neuron destroyer** | Macrophage | Siege Synapse | Metastatic Bud |

**Connection Cutter**: Mobile. Targets axon compartments — drains structural integrity until the compartment breaks, severing the connection. Can switch to direct health damage against enemy mobile units when ordered.

**Neuron Destroyer**: Slow, durable. Locks onto a single neuron soma and rapidly drains its metabolic energy. One target at a time — powerful but methodical.

**Fast Raider**: Fast, fragile. Prioritizes high-value targets (radial glial cells, generators). Burst damage on contact, then cooldown vulnerability.

**Resource Extractor / Area Control**: Stationary. Harvests from resource nodes AND continuously absorbs health from enemy mobile units in radius. Economy backbone and passive guard tower in one.

---

## Economy

### Metabolic Model

Every cell consumes energy:
- **Resting cost**: Small constant drain to maintain membrane potential
- **Firing cost**: Significant energy per spike
- **Growth cost**: Axon growth costs per compartment. Cell production costs a lump sum at the radial glial cell.
- **Migration cost**: Migrating precursors consume energy during transit
- **Maintenance cost**: Ependymal walls, oligodendrocyte myelin, astrocytes all have ongoing costs

### Resource Flow

Resources flow along axon connections from harvesting neurons/astrocytes to consuming cells. Flow rate is proportional to axon weight. Astrocytes near resource nodes harvest with greater efficiency than bare neurons.

### Expansion Dilemma

- Nearby resources: safe, easy to reach, limited income
- Distant resources: require long axon growth through contested space, migrating cells must survive the journey, but rich returns
- Overextension: growing faster than income supports means frontier cells starve
- Second base: self-replicating the radial glial cell gives local production, but is extremely expensive and the precursor must survive a long, dangerous migration

---

## Environmental Systems

### Microglia (NPC)

Not player-controlled. Microglia are immune cells that originate from outside the brain (yolk sac macrophages). They patrol the dish randomly. When a neuron dies or an axon breaks, nearby microglia activate and converge on the damage site:
- Clear dead cell debris (cosmetic)
- Release pro-inflammatory cytokines that damage ALL nearby cells (both players)
- Create a temporary "inflammation zone" that's lethal to migrating precursors

This means killing enemy neurons triggers collateral inflammation. Tight clusters of enemy neurons are vulnerable to cascade kills via inflammation. But your own nearby cells get hit too. Rewards surgical strikes with long-range axons over close-quarters brawling.

### Toxins

Drift through the soup. Depolarize exposed neurons toward depolarization block and calcium death. Membrane walls (ependymal cells) block toxin damage.

### Free Radicals

Drift through the soup. Strip myelin from axons, stress axon compartments (push them apart). Over time, demyelinated axons slow down and become fragile. Membrane walls provide some protection.

---

## Movement

Neurons drift passively unless driven. To create directed movement:

**Motor cilia**: An attachment on a neuron that converts its firing rate into propulsion. Direction is set by the player.

**Moving a cluster**: Neurons bound by structural connections (cytoskeletal links) move as one body. Heavier clusters move slower.

**Constraints**: Clusters containing the radial glial cell cannot move — your base stays put. Moving a cluster stretches its axon connections to the main network. If they stretch too far, they break — severing the mobile unit from the network.

---

## Visual Design

Dark background with saturated accent colors.

- **Neurons**: All circles. Excitatory = blue, Inhibitory = red. Team indicator via thin colored ring (player = red ring, enemy = orange ring). Origin (radial glial cell) is larger with ★ symbol.
- **Axons**: Polylines of compartments. Blue for excitatory, red for inhibitory. Pulsing signal dots travel along the chain when spikes propagate. Myelinated axons have a visible sheath and are thicker.
- **Growth cones**: Bright dot at the advancing tip of a growing axon.
- **Migrating cells**: Small, dim dots crawling through the soup toward their destination.
- **Astrocytes**: Star-shaped, stationary, glowing softly near resource nodes.
- **Oligodendrocytes**: Small cells sitting on axons, visible as beads along the myelin sheath.
- **Ependymal walls**: Circular membrane cells forming barriers.
- **Microglia (NPC)**: Distinct color (yellow-green), patrol randomly, flash red when activated.
- **Toxins**: Purple drifting particles. Free radicals: yellow sparks.
- **Inflammation zones**: Temporary red-orange glow around damage sites where microglia have activated.
- **Calcium damage**: Neurons accumulating lethal damage glow increasingly bright/white before dying.

---

## Technical Foundation

### Simulation Engine
Forward Euler integration of integrate-and-fire equations. The game adds spatial coordinates, compartmental axon physics (spring forces, breakage), cell migration, and resource flow.

### Architecture (ECS)
- Neuron entities with voltage, refractory timer, calcium damage, auto-fire, motor components
- Axon entities with compartment chains, spring physics, signal propagation, myelination state
- Cell migration system (precursor position, velocity, target, maturation timer)
- Growth cone system (axon extension, compartment spawning)
- Environmental systems (toxin drift, microglia patrol/activation, inflammation)
- Resource flow along axon graph

### Platform Target
Web/desktop with GPU-accelerated simulation and rendering (WebGPU compute shaders for compartment physics and signal propagation).

---

## Development Phases

### Phase 1: Soup Sandbox
- Petri dish canvas with radial glial cell
- Neuroblast spawning, migration through soup, maturation
- Axon growth via growth cones (compartment by compartment)
- Basic excitatory and inhibitory neurons with integrate-and-fire simulation
- Signal propagation along axon compartments
- Resource nodes with metabolic harvesting
- Neuron death from starvation
- Single-player sandbox

### Phase 2: Combat Mechanics
- Excitotoxicity (calcium damage from sustained bombardment)
- Inhibitory silencing
- Axon breakage from tension
- Semaphorin neurons (repulsive axon disruption)
- Mobile combat units with NeuronSpawner
- Win condition: network starvation / decapitation
- Scenario system for testing and balancing

### Phase 3: Support Infrastructure
- Astrocytes (economic boost + area denial)
- Oligodendrocytes (myelination)
- Ependymal cells (membrane walls)
- Environmental hazards (toxins, free radicals)
- Microglia NPC system (inflammation on damage)

### Phase 4: Depth and Polish
- Adaptive, burst, and neuromodulatory neurons
- Faction visual themes (Biological / Tech / Tumor)
- Tech tier progression
- Second base (radial glial self-replication)
- Synaptic plasticity (Hebbian/STDP learning)
- Substrate gradients and map variety
- UI polish: zoom levels, minimap, production queue

---

## Key Design Principles

1. **Nothing is instant.** Every cell migrates. Every axon grows. Every structure takes time. Logistics matter.
2. **One layer, one view.** The circuit IS the battlefield. No mode switching.
3. **Biology is the mechanic.** Excitotoxicity, chemoattraction, inflammation, myelination — these aren't flavor text, they're the game rules.
4. **Connectivity is control.** You own what you're connected to. Severing connections is the most strategic act in the game.
5. **The simulation is honest.** The same differential equations drive everything. No hidden dice rolls.
6. **Structure is earned.** Nothing is a creature, a wall, or a supply line until you build it from migrating cells and growing axons.
