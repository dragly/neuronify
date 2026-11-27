These scripts draw the Neuronify icon and export every icon file the app ships.
After a change, rebuild them all from the repository root with:

    uv run tools/icon/build.py

This needs uv and Inkscape.
It overwrites `icon.svg`, `icon.png`, `favicon.ico` and `macos.iconset/` in `neuronify-core/assets/`
and `neuron-icon.js` in `neuronify/`,
or writes them all to the folder given as the first argument instead.
The shipped files were rendered with Inkscape 1.2,
so another version may change a few pixels even when the drawing is the same.

The drawing is in `neuron.py`.
It holds the traced points of the soma, dendrites and axon,
the stroke width, the size of the nucleus
and the radius at each kind of corner
(the inside of a fork, its outer sides, where a branch meets the soma,
the hole in the soma, the two sides of the axon bend, the line ends and the nucleus).
The outline is built sharp first and every corner then gets an exact arc,
so moving a point keeps the rounding consistent.
The `SMALL_` values describe the bolder drawing used for the macOS 16 and 32 px files.

The colours, the shadows, the rounded tile and the macOS grid are at the top of `build.py`.

`neuron-icon.js` holds what the loading screen needs to draw the neuron:
the colours, the shadow and the outline as rings of points.
Each point is stored in quarter units as the step from the point before,
and the arcs are cut into straight pieces of at most 9 degrees.

The 16 and 32 px favicon sizes are pixel art in `pixel-16.png` and `pixel-32.png`.
They are edited by hand (Aseprite works well) and copied into `favicon.ico` as they are.
