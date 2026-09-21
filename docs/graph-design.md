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
- The order of work is 2D first (axes, arrowheads, ticks, labels, function graphs), then 3D on the same intermediate representation. One minimal 3D figure (see Minimal 3D figure) was built before the 2D extensions, to test that the intermediate representation carries 3D.
- The engine core is written in Rust and runs as Wasm from the start, so the scene and the intermediate representation are fixed once and 3D does not force a rewrite.
- Rust turns a scene into the intermediate representation (IR) and into the TikZ text. TypeScript builds the SVG from the IR and draws the label formulas with MathJax.
- The scene is a JSON file, one per figure. A GUI that edits graphs (typing an expression, dragging a point) is planned, and the scene must stay easy for it to read and write. This gives the rules below.
- The figure feature is developed test-first: a failing test is written before the code that makes it pass. The first figure's JSON file is a fixture of the tests.

## Implemented

The first figure (`s0202graph1`) is drawn end to end: scene JSON, Rust engine compiled to Wasm, SVG at build time, and TikZ text. The Rust side is `crates/figure/` (entry points `parse_scene`, `render`, `tikz::export_tikz`); every module has tests in `crates/figure/tests/`, written first.

- Scene (`scene.rs`, `parse.rs`, `validate.rs`, `version.rs`): the types read and write JSON, so a scene that was written out can be read again. Omitted fields take defaults: `arrow` is `stealth`, `anchor` is `center`, and the line is solid. A length is written as a string such as `"2.5mm"` (`cm`, `mm`, `pt`). The engine reads a scene whose major version equals its own (its minor version too while the major version is 0) and that is not newer than itself; the version is checked before any other field. Checks: a non-empty `description`, increasing finite ranges, identifiers for `id` and `var`, unique `id`s, non-empty `tex`, two expressions per plane curve and three per space curve; unknown fields and object types are rejected.
- Expressions (`expr/`): a lexer, a recursive-descent parser, and a floating-point evaluator, separate from the polynomial calculator (identifiers here have several letters, and coefficients are floats). Grammar: `+ - * /`, right-associative `^`, unary minus below `^` (`-x^2` is `-(x^2)`), parentheses. Implicit multiplication (`2x`) is an error. Functions: `sin cos tan asin acos atan sinh cosh tanh exp log ln sqrt abs` (`log` and `ln` are natural logarithms). Constants: `pi`, `e`. Errors carry a span counted in characters. Limits: 2000 characters, depth 128.
- Compile pass (`compile.rs`): reads every expression with the names `[var, parameters…]`, evaluates the domain ends (which may use parameters and constants, not the variable), and rejects a domain that is not finite and increasing, function or constant names as ids or variables (`reserved_name`), and a variable named like a parameter (`name_conflict`). `parse_scene` runs it, so a scene that parses can be rendered.
- Sampling (`sample.rs`): 16 fixed intervals, then adaptive halving (depth ≤ 12) until the points at 1/4, 1/2, and 3/4 are within 0.003 cm of the chord. Points that are not finite or exceed 300 cm cut the line (TikZ cannot handle larger); the cut is located by bisection. Accuracy is tested as the distance from the true curve to the polyline, not the vertical error, which overstates steep curves.
- Arrowhead (`arrow.rs`): `Stealth`, ported from the pgf source (see Verified facts). Its outline is tested against an independent computation of the mitered outline.
- Intermediate representation (`figure.rs`, `render.rs`): a `Figure` of `Path` items (points in cm, y up, stroke width in pt, an optional arrow with its polygon and the point where the shaft stops) and `Label` items (position in cm, anchor, TeX text). Items follow the order of the scene objects. An axis becomes a path (axis width 0.6 pt) and, when it has a `label`, a label `$name$` at its end (`west` for x, `south` for y). Curves are 0.8 pt. `bounds` is the view plus a 0.6 cm margin.
- TikZ (`tikz.rs`): `export_tikz(json)` writes `\draw` and `\node` lines with numbers to four decimals, arrows as `-{Stealth}`, and the scene JSON as written, in a header comment; `extract_scene` recovers it. The output for the first figure is compared with `tests/golden/sine-and-shifted-sine.tikz` (regenerate with `UPDATE_GOLDEN=1 cargo test -p figure --test tikz` and read the diff). The Wasm build must produce the same text: `site/src/figure/wasm.test.ts` compares it with the same file.
- Wasm (`crates/figure-wasm/`): `parseScene` and `renderScene`, with typed `SceneOutcome`, `RenderOutcome`, and `Figure`. An error has `code`, `message`, `object`, `line`, `column`, and, for expressions, `field`, `index`, `start`, `end`.
- SVG (`site/src/figure/svg.ts`, `label.ts`): the IR becomes a `<figure>` with an `<svg role="img" aria-label>` (viewBox in cm, y flipped, stroke widths converted from pt, `dotted` and `dashed` with the TikZ dash lengths, arrowheads as filled and stroked polygons with mitered joins) and, over it, one absolutely positioned `<span class="figure-label">` per label. The label's TeX (text with `$…$`) becomes one formula, `\text{…}` for the text parts, and is placed as inline math, which the MathJax step renders. The box moves by the anchor (a west label has its left edge on the point) and has a 1/3 em padding, like a TikZ node. `aria-hidden` is set on labels because the SVG carries the description.
- MDX (`site/src/plugins/rehype-figures.ts`): `<Figure src="name" />` is replaced by the figure; `name` is lowercase letters, digits, and hyphens, and names `site/src/figures/<name>.json`. The plugin loads the Wasm with `initSync` when the config is read, so `mise run wasm` must have run. It runs before `rehypeMathjax`. A scene error fails the build with the figure name and the object `id`. The dev server does not watch the scene files: touch the MDX file to see a change.
- Pages: `dev/figure-first.mdx` shows the first figure, and `dev/figure-scene.mdx` edits a scene and shows the result, the errors, and the TikZ. Both are linked from the home page. `e2e/figure.spec.ts` measures the positions of the labels against the axes, the physical size (1 cm is 37.8 px), the layout at 390 px, and the colours in both themes.

Known limits, the first things to extend:

- No clipping to the view: a graph that leaves the view is drawn outside it (SVG shows the bounding box, TikZ does not clip). Poles such as `tan` are cut only by the 300 cm limit.
- No ticks, tick labels, grid, or dependent points; the axis crosses at 0.
- Curves are polylines in the TikZ output (about 100 points per period); Bézier fitting would make them smaller and smoother.
- The label font is the MathJax font, not the document font of the TikZ output.
- Colours and line widths are not yet settable in the scene.

## Minimal 3D figure

The first 3D figure is `s0905sphere`, a sphere of radius 2 with the three axes through it. The sample shows the parts of the 3D requirements at once: the silhouette circle, axes that are dotted where the sphere hides them, and axes that break where they meet the surface. The sphere is an exact shape (`type: "sphere"`), so visibility is a closed-form ray test. General parametric surfaces (a mesh, and outlines from the normal) come later, with the sphere as the reference to check them against.

Measured from the sample image (`s0905spherefig.jpg`): the projection is parallel (a radius-2 sphere is a circle of the same radius as 2 units on the axes), and the view matches an azimuth of about 64° and an elevation of about 22°, in the convention below. The `x` axis leaves to the lower left, `y` to the lower right, `z` up. Labels sit beyond the positive ends. Each axis is one line through the origin, from -5 to 5.

Decisions for the minimal figure:

- The engine version stays 0.1.0. The new fields are additive, and no scene has been published.
- `view` is either a plane view (`x`, `y`, `unit` with two lengths, as before) or a space view: `azimuth` and `elevation` in degrees and `unit`, one length for all three axes. The camera is in the direction `(cos e cos a, cos e sin a, sin e)` from the origin, looking at it. The screen's right is `(-sin a, cos a, 0)` and its up is `(-sin e cos a, -sin e sin a, cos e)`, so `x` comes toward the viewer at azimuth 0 and elevation 0, with `y` to the right and `z` up. A space figure has no visible range: the bounds are those of the drawn items.
- An object type belongs to one kind of view. `graph` is a plane object, `sphere` is a space object, and `axis`, `curve`, `label`, and `parameter` are both. A `label` has two numbers in `at` in a plane view and three in a space view; a label is never hidden by a surface. A space `axis` needs `range`; `direction` may be `z` only in a space view. A space `curve` has three expressions.
- `style` gets `hidden`: `dotted` (default), `dashed`, or `none` (not drawn), the style of the parts an opaque surface hides. An `axis` gets `style` for it.
- A point is hidden when the ray from it toward the camera meets a sphere at a positive distance; a point inside a sphere is hidden. The switch points between visible and hidden pieces are refined by bisection.
- The IR does not change: it is still 2D paths and labels, so SVG and TikZ need no change.

Implemented (`space.rs`, tests in `tests/space_scene.rs` and `tests/space.rs`, pages `dev/figure-space.mdx`, fixtures `sphere-with-axes.json` and `sphere-with-circles.json`):

- The visibility test is an independent oracle in the tests: march along the line of sight and look for a point inside the sphere. Points of pieces are checked against it, and the switch points against the closed-form values (the axis leaves the surface at `x=2`, and grazes it at `|t|·hypot(d_y, d_z)=r`).
- Rounding error puts a point on the sphere itself (a curve on the surface) on either side of it, so it would be flagged hidden or visible at random. The hiding radius is 1e-12 smaller, which puts surface points outside; the price is a band of about 3e-6 around the outline where the switch is not exact.
- A closed curve is split at the start of its parameter range, so a visible half that spans it becomes two solid paths. Merging them is a later refinement.
- The sphere outline is a polyline of a circle with the same 0.003 cm tolerance as curves. TikZ `circle` or Bézier arcs would be smaller.
- Only spheres hide. The outline of one sphere is not hidden by another, and a hiding sphere does not hide another sphere's outline.
- Axes are cut at 64 sample points before bisection, so a hidden interval shorter than one step is missed.
- A space axis with an arrow draws the head only when its last piece is visible, and a label is always drawn at the positive end. `anchor` for the label is chosen from the eight directions of the projected axis.
- The origin `O` is a `label` at `[0, 0, 0]` in every space figure, as in plane figures. Its anchor is the free gap between the projected axes (`north east` at azimuth 60 and elevation 20, the lower left of the origin).
- Not done: points at 3D positions, surfaces given by expressions, plane cuts, hidden-line removal between curves and non-spherical surfaces, ticks, perspective.

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
- `label`: text placed at a point. `tex` is TeX text in which `$…$` is a formula (`Graph of $y=\sin x$`). `anchor` is the TikZ `anchor`: the part of the label box that is put on the point, so `west` makes the label extend to the right of the point, and `north west` puts the label below and to the right. The origin `O` is a label, not part of an axis. An axis `label` is a formula: `"x"` is drawn as `$x$`.
- `parameter`: a named number used in expressions. A GUI shows it as a slider.
- `graph`: `y = f(x)`. `curve`: a parametric curve, whose `expr` is an array of two expressions (plane) or three (space), for example `["cos(t)", "sin(t)"]` with `"var": "t"`. Domain ends are numbers or expressions such as `"2*pi"`.
- Later: implicit curves, polar curves, and surfaces (two variables, `expr` of three).
- Sampling must adapt to curvature (cycloid cusps) and cut the line at breaks (`tan x`).

## Proposed, not confirmed

- Support parallel projections only (oblique and orthographic, from a view direction). Perspective is out of scope.
- Decide visibility by sampling curves and meshing surfaces, then refine the switch points by bisection. Extract outlines as the curve where the surface normal is perpendicular to the view direction. Add exact shapes for spheres, cylinders, and cones later.
- Add the arrowheads `Latex` and `To` from the same pgf source, and fit smooth curves with Bézier segments in the export.
- A breaking change to the scene format bumps the application version and comes with a migration.
- A rotating view can be added later with the same Wasm in the browser.

## Verified facts

### Stealth arrowhead

Read on 2026-09-21 from the master branch of the pgf repository: the declaration of `Stealth` in `tex/generic/pgf/libraries/pgflibraryarrows.meta.code.tex`, and the dimension setup in `tex/generic/pgf/basiclayer/pgfcorearrows.code.tex`.

- With `lw` the line width of the path, the defaults are length `L = 3pt + 4.5·lw`, width `W = 0.75·L`, and inset `I = 0.325·L`. The third number in `length = +3pt 4.5 .8` only applies to double lines. At the default line width of 0.4pt this gives L = 4.8pt, W = 3.6pt, and I = 1.56pt.
- The tip is a closed path of four points (tip, upper back corner, inset point, lower back corner), filled and stroked with mitered joins. The stroke width is `lw' = min(lw, (L − I) / 4)`. The points are pulled inward by the miter lengths: at the tip `0.5·lw'·sqrt(4(L/W)² + 1)`, at the inset point `0.5·lw'·sqrt(4(I/W)² + 1)`, and at the back corners by an angle formula in the source.
- The visual tip is at `x = L` from the back of the arrow. The line ends at `I + (inset miter) − 0.25·lw'` from the back, so it does not show through the tip.
- pgf computes the corner miters from the nominal shape (length, width, inset), not from the mitered path, so the real outer corner protrudes past the L × W box by about a tenth of the line width (measured: 0.028 pt at 0.4 pt). The port keeps this; its test allows a tenth of the line width.
- Port the code from the source, not from this summary. Without TeX Live in the container, the port cannot be compared with real TikZ output yet.

### Dash patterns

From `tex/generic/pgf/frontendlayer/tikz/tikz.code.tex` (master, read on 2026-09-21): `dotted` is `dash pattern=on \pgflinewidth off 2pt`, and `dashed` is `dash pattern=on 3pt off 3pt`. The SVG output uses the same lengths.

## Open

- The figures required after `s0202graph1`, and the order of the extensions listed under Implemented.
- Whether the scene gets colours and line widths, and how a figure is captioned in an article.
