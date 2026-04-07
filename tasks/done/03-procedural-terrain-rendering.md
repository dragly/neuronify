# Task 3: Procedural Terrain Rendering

Render the hex grid as a seamless, organic-looking map — not as visible hexagons. The final result should look like a hand-painted biological tissue map, not a board game.

## Approach

Generate a procedural mesh from the hex grid data. Each hex cell defines a terrain type, but the rendering should blend between adjacent terrain types to create organic transitions. This is the standard tile-transition technique used in games like Civilization, Age of Empires, and Battle for Wesnoth, adapted for biological tissue.

## Per-terrain-type rendering

**Open (extracellular matrix):** Pale, slightly textured surface. Use a subtle noise texture to suggest fibrous protein mesh. This is the default "ground" — it should feel like a microscope slide of tissue.

**ECM sparse:** Same as open but slightly darker/denser texture. The fibers are more visible. Subtle visual difference — the player should notice it but it shouldn't dominate.

**ECM dense:** Noticeably darker, thicker fibrous texture. Dense protein mesh that clearly reads as "rough terrain." Think of collagen fibers under a microscope.

**CSF pool:** Fluid surface. Subtle animated ripple or shimmer effect. Light blue-gray, slightly reflective. Clear visual contrast with solid terrain — this is open water with no substrate.

**Glial scar:** Dark, thick, irregular scar tissue. Raised-looking with hard edges. Should look like a wound that healed wrong — dense, tough, impenetrable. Use bump/height visual cues.

**Blood vessel:** Three-dimensional tube rendered to look like an actual blood vessel viewed from above. Red/pink with visible lumen (hollow center). Slight glossy/wet appearance. The vessel should look like it has volume — use shading to suggest a cylindrical cross-section. Where multiple vessel hexes connect, the tube should flow continuously (not hex-shaped segments). This is the most visually distinctive terrain type.

## Terrain transitions

Where two different terrain types share a hex edge, blend between them. Do NOT render hard hex-shaped boundaries. Techniques:

- For each hex edge shared between different terrain types, render a gradient or feathered transition across the edge
- Vessel hexes are the exception: vessels should have hard, defined edges (blood vessel walls are distinct boundaries in real tissue)
- CSF pool edges should also be relatively hard (fluid-substrate interface)
- ECM transitions (open ↔ sparse ↔ dense) should be very soft and gradual
- Glial scar edges should be rough and irregular (scar tissue has ragged boundaries)

## What NOT to do

- Do not render visible hex outlines in the final game view
- Do not use flat solid colors per hex
- Do not make it look like a board game grid
- The hex grid is the DATA structure. The rendering is a VISUAL interpretation of that data.

## Testing

Render the full scenario map. Verify: hex boundaries are not visible, terrain types are visually distinct, transitions look organic, blood vessels look tubular, CSF pools look like fluid.
