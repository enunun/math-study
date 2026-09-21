# Figure and graph design

Requirements, reference material, and decisions for the figure feature (2D graphs and 3D figures with TikZ export). The technology is not chosen yet; this file records the requirements and the open questions so the selection can proceed in small steps.

## Requirements

- Figures look like blackboard drawings in a mathematics class, not like plots. The shape of axis arrowheads, ticks, and labels is part of the specification. Whether the axes are drawn at all is an option of each figure.
- 3D figures are supported.
- Hidden lines are drawn dotted or dashed, and the outline (silhouette) of a curved surface is drawn, as KeTCindy does.
- Figures export to TikZ. The TikZ output is a build product and is never edited by hand: to change a figure, change the data it was made from and export again.
- The data before the TikZ conversion (the scene) is kept in some form.
- A GUI application (an existing one, or one built here) may be connected later. It is out of scope now, but the design must not block it.
- Installing TeX Live in the container, to compile the TikZ output, is low priority.

## Reference figures

The KeTCindy sample gallery is the acceptance set. Each page lists figures with their source (`.txt` is the CindyScript that drew it).

- 2D graphs: <https://s-takato.github.io/ketcindysample/s02graphs/index.html>. Function graphs with plain axes, `O` at the origin, and axis labels; a second curve drawn dotted; a region between two curves hatched with oblique lines (`s0208intersecthatch`); implicit curves, envelopes, and differential-equation curves.
- Geometric figures: <https://s-takato.github.io/ketcindysample/s01geometricfigures/index.html>. Vectors with filled triangular arrowheads, dashed helper lines, and labels such as `\vec{a}+\vec{b}` (`s0105vector`); polygons, hatching, cycloids and other curves.
- Surfaces: <https://s-takato.github.io/ketcindysample/s09surfaceC/index.html>. Paraboloid, cone, saddle, sphere, a curve on a surface, plane cuts of a cone (`s0909conecut`), several surfaces together.

What the 3D samples show, and so what the engine must reproduce:

- The projection is parallel, as far as the images show, and the sources refer to two view angles, `TH` and `FI`.
- A surface is an opaque shell. The axes, the far half of a boundary circle, and the far part of a curve on the surface are hidden by it and drawn dotted. An axis is broken exactly where it meets the surface.
- Only the outline, the boundary edges, and the curves the author asks for are drawn. There is no wireframe by default.
- Curves on a surface and cuts by planes (`z=x-y`) are separate objects, drawn with their own colour; their hidden parts are dotted too.
- Curves in the samples are sometimes visibly faceted (`s0208intersecthatch`). The target is smooth curves.
- KeTCindy computes surfaces in C through an external compiler (`ExeccmdC`, GCC required). The equivalent here is the Rust core compiled to Wasm, with no external tool.

## Architecture under consideration

```text
scene (data) → geometry engine (projection, silhouette, visibility) → 2D vector IR → SVG / TikZ
```

- The scene is a declarative, serializable description with named objects. It contains no drawing instructions.
- The geometry engine is a pure function from a scene to the intermediate representation (IR). The IR holds paths, styles (solid, dotted, dashed), arrowheads, and labels with an anchor and a direction.
- SVG (build-time, static) and TikZ are two serializers of the same IR, so the screen and the export do not drift apart.
- Only vector output is possible: pixel renderers such as WebGL cannot produce TikZ.
- Existing tools do not cover the combination. KeTCindy depends on Cinderella 2 (CindyJS has open compatibility issues). Asymptote has strong 3D vector output, but no TikZ output was found. Not verified: TikZ and pgfplots do not compute hidden lines, and Three.js and JSXGraph are pixel or interactive tools.

## Decided

- The default arrowhead is TikZ's `Stealth`. The tip is a setting of each axis and vector, so it can be changed.
- Hidden lines are dotted by default. The style is a setting, so dashed can be chosen.
- The first target figure is `s0202graph1`: a sine curve and a shifted copy drawn dotted, on axes with `O`, `x`, and `y`, and a text label containing math. The sample draws its axes without arrowheads; here they get the default arrowhead.
- The order of work is 2D first (axes, arrowheads, ticks, labels, function graphs), then 3D on the same intermediate representation.

## Proposed, not confirmed

- Write the engine in Rust and run it as Wasm, like the calculator: the toolchain, the strict lint setup, and the parser are reused; floating-point results are deterministic across build, browser, and a later desktop or CLI build.
- Support parallel projections only (oblique and orthographic, from a view direction). Perspective is out of scope.
- Decide visibility by sampling curves and meshing surfaces, then refine the switch points by bisection. Extract outlines as the curve where the surface normal is perpendicular to the view direction. Add exact shapes for spheres, cylinders, and cones later.
- Fit smooth curves with Bézier segments in the export.
- Define each arrowhead (`Stealth`, and later `Latex` and `To`) with the dimensions of the TikZ `arrows.meta` tip, and draw the same geometry in SVG. The dimensions are taken from the pgf source, not from memory.
- Treat readability of the TikZ output as a non-goal: fidelity to the figure and a compact size matter. Until TeX Live is available, check the output against golden files instead of compiling it.
- Keep the scene as a JSON file, and write the engine version and the scene into a header comment of the TikZ file, so an export can be traced back and reproduced.
- Render figures to static SVG at build time. A rotating view can be added later with the same Wasm.

## Open

- How a figure is written in MDX (props, a small language, or TypeScript).
- The split of work between Rust and TypeScript, and where the SVG serializer lives (labels need MathJax, which runs in TypeScript).
- The figures required after `s0202graph1`.
