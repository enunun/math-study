# Figure and graph design

Requirements, reference material, and decisions for the figure feature (2D graphs and 3D figures with TikZ export). The core technology is chosen (see Decided); the open questions are listed at the end, so the rest can be settled in small steps.

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
- The engine core is written in Rust and runs as Wasm from the start, so the scene and the intermediate representation are fixed once and 3D does not force a rewrite.
- Rust turns a scene into the intermediate representation (IR) and into the TikZ text. TypeScript builds the SVG from the IR and draws the label formulas with MathJax.
- The scene is a JSON file, one per figure. A GUI that edits graphs (typing an expression, dragging a point) is planned, and the scene must stay easy for it to read and write. This gives the rules below.

- The figure feature is developed test-first: a failing test is written before the code that makes it pass. The first figure's JSON file is a fixture of the tests.

## Implemented

Step 1, the scene, in `crates/figure/` (`parse_scene` is the entry point; the tests are in `crates/figure/tests/`):

- The types of the scene (`scene.rs`): `Scene`, `View`, and the objects `Axis`, `Label`, `Parameter`, `Graph`, `Curve`. They read and write JSON, so a scene that was written out can be read again. Omitted fields take defaults: `arrow` is `stealth`, `anchor` is `center`, and the line is solid. A length is written as a string such as `"2.5mm"` (`cm`, `mm`, `pt`).
- The version (`version.rs`): the engine version is the workspace version. The engine reads a scene whose major version equals its own (its minor version too while the major version is 0) and that is not newer than itself. The version is checked before any other field, so a newer scene with new fields reports the version, not an unknown field.
- The checks (`validate.rs`): a non-empty `description`, increasing finite ranges (`view`, `range`, numeric `domain`), identifiers for `id` and `var`, unique `id`s, non-empty expressions and `tex`, two expressions per curve. Unknown fields and unknown object types are rejected.
- An error carries the `id` of the object it came from (`Error::object`). JSON syntax errors carry the line and the column.
- Expressions are still plain strings. Their syntax, the names they use, and the order of an expression domain are checked in step 2.

## Scene rules

- The scene holds inputs only: points, parameters, expressions, and styles. Nothing computed is stored.
- Every object has a stable `id`, and objects refer to each other by `id`. Dragging a point or a slider changes one value in the JSON, and the engine runs again.
- The engine is a pure function of the scene. An error carries the `id` of the object and, for an expression, the position in it, so a GUI can point at the cause.
- The scene has a `version`, the semantic version of the application that wrote it, and unknown fields are rejected.

## Scene format

The fields below are those of the first figure, `site/src/figures/sine-and-shifted-sine.json` (the reference figure `s0202graph1`). Other object types are added together with the figures that need them. Figure files are named after their content, in lowercase with hyphens.

```json
{
  "version": "0.1.0",
  "description": "y=sin x のグラフと，x軸の方向に平行移動した点線のグラフ",
  "view": { "x": [-7, 7], "y": [-1.6, 1.8], "unit": { "x": "1cm", "y": "2cm" } },
  "objects": [
    { "id": "x_axis", "type": "axis", "direction": "x", "arrow": "stealth", "label": "x" },
    { "id": "y_axis", "type": "axis", "direction": "y", "arrow": "stealth", "label": "y" },
    { "id": "origin_label", "type": "label", "at": [0, 0], "anchor": "south east", "tex": "O" },
    { "id": "shift", "type": "parameter", "value": -1.2 },
    { "id": "sine", "type": "graph", "var": "x", "expr": "sin(x)", "domain": [-7, 7] },
    {
      "id": "shifted_sine",
      "type": "graph",
      "var": "x",
      "expr": "sin(x - shift)",
      "domain": [-7, 7],
      "style": { "line": "dotted" }
    },
    {
      "id": "title",
      "type": "label",
      "at": [1, 2],
      "anchor": "east",
      "tex": "Graph of $y=\\sin x$"
    }
  ]
}
```

- `description` becomes the alternative text of the SVG (the site requires zero axe violations, so a figure needs an accessible name). `view` gives the visible range in mathematical coordinates and the physical length of one unit on each axis, so the two scales can differ.
- An `id` starts with a letter and is made of letters, digits, and `_`, because expressions refer to a `parameter` by its `id`.
- `axis`: one object per axis, so a single axis or a half axis (`range: [0, 5]`) is possible. `direction` is `x` or `y` (`z` in 3D). `arrow` is `stealth` by default and can be `none` or another tip. `range` defaults to the view.
- `label`: a formula placed at a point with an anchor. The origin `O` is a label, not part of an axis.
- `parameter`: a named number used in expressions. A GUI shows it as a slider.
- `graph`: `y = f(x)`. `curve`: a parametric curve, whose `expr` is an array of two expressions (plane) or three (space), for example `["cos(t)", "sin(t)"]` with `"var": "t"`. Domain ends are numbers or expressions such as `"2*pi"`.
- Later: implicit curves, polar curves, and surfaces (two variables, `expr` of three).
- Sampling must adapt to curvature (cycloid cusps) and cut the line at breaks (`tan x`).

## Proposed, not confirmed

- Support parallel projections only (oblique and orthographic, from a view direction). Perspective is out of scope.
- Decide visibility by sampling curves and meshing surfaces, then refine the switch points by bisection. Extract outlines as the curve where the surface normal is perpendicular to the view direction. Add exact shapes for spheres, cylinders, and cones later.
- Fit smooth curves with Bézier segments in the export.
- Define each arrowhead (`Stealth`, and later `Latex` and `To`) with the geometry of the TikZ `arrows.meta` tip (see Verified facts), and draw the same shape in SVG.
- Expose the application version from Wasm, so the site, the TikZ header comment, and a GUI read one source. A breaking change to the format bumps the version and comes with a migration.
- Treat readability of the TikZ output as a non-goal: fidelity to the figure and a compact size matter. Until TeX Live is available, check the output against golden files instead of compiling it.
- Write the engine version and the scene into a header comment of the TikZ file, so an export can be traced back and reproduced.
- Render figures to static SVG at build time. A rotating view can be added later with the same Wasm.

## Verified facts

### Stealth arrowhead

Read on 2026-09-21 from the master branch of the pgf repository: the declaration of `Stealth` in `tex/generic/pgf/libraries/pgflibraryarrows.meta.code.tex`, and the dimension setup in `tex/generic/pgf/basiclayer/pgfcorearrows.code.tex`.

- With `lw` the line width of the path, the defaults are length `L = 3pt + 4.5·lw`, width `W = 0.75·L`, and inset `I = 0.325·L`. The third number in `length = +3pt 4.5 .8` only applies to double lines. At the default line width of 0.4pt this gives L = 4.8pt, W = 3.6pt, and I = 1.56pt.
- The tip is a closed path of four points (tip, upper back corner, inset point, lower back corner), filled and stroked with mitered joins. The stroke width is `lw' = min(lw, (L − I) / 4)`. The points are pulled inward by the miter lengths: at the tip `0.5·lw'·sqrt(4(L/W)² + 1)`, at the inset point `0.5·lw'·sqrt(4(I/W)² + 1)`, and at the back corners by an angle formula in the source.
- The visual tip is at `x = L` from the back of the arrow. The line ends at `I + (inset miter) − 0.25·lw'` from the back, so it does not show through the tip.
- Port the code from the source, not from this summary. Without TeX Live in the container, the port cannot be compared with real TikZ output yet.

## Open

- How a figure is referenced from MDX. The candidate is `<Figure src="sine-and-shifted-sine" />`.
- The fields of the intermediate representation (IR).
- The figures required after `s0202graph1`.
