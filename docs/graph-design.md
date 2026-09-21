# Figure and graph design

Requirements, reference material, and decisions for the figure feature (2D graphs and 3D figures with TikZ export). The core technology is chosen (see Decided); the open questions are listed at the end, so the rest can be settled in small steps.

## Requirements

- Figures look like blackboard drawings in a mathematics class, not like plots. The shape of axis arrowheads, ticks, and labels is part of the specification. Whether the axes are drawn at all is an option of each figure.
- 3D figures are supported.
- Hidden lines are drawn dotted or dashed, and the outline (silhouette) of a curved surface is drawn, as KeTCindy does.
- Figures export to TikZ. The TikZ output is a build product and is never edited by hand: to change a figure, change the data it was made from and export again.
- The data before the TikZ conversion (the scene) is kept in some form.
- A GUI application (an existing one, or one built here) may be connected later. It is out of scope now, but the design must not block it.
- The TikZ output is compiled with TeX Live in the container and compared with the SVG (`mise run tikz`); see Verified facts.

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
- Clipping (`clip.rs`): graphs and curves in a plane figure are clipped to the view rectangle after sampling (Liang–Barsky per segment). A line that leaves the view ends on the edge, and one that comes back starts on the edge as a new path, so SVG and TikZ need no clip region and the output holds only what is visible. Points on an edge stay; a line that only touches an edge or corner is dropped. The new end points lie on the chord of the sampled polyline, so they differ from the curve by at most the sampling tolerance (0.003 cm). Axes are not clipped: they use their own `range`, which may exceed the view. Space figures have no view rectangle, so nothing is clipped.
- Ticks (`ticks` of a plane axis, `Tick` in `scene.rs`, `compile_axis`, `tick_items`): an explicit list of `{at, label?}`. `at` is a number or an expression (parameters and constants, no variable) and must lie in the axis range (the view range when `range` is omitted), ends included; a bad expression is reported as `ticks`, with the index of the tick. A tick is a solid 0.6 pt line across the axis, 3 pt to each side, and an optional label `$…$` put at the line end. Its `anchor` (a TikZ anchor name, as for `label`) is `north` for the x axis and `east` for the y axis unless the tick sets it, so a label can be moved away from the curve: for `sin x`, `north east` under `π` and `north west` under `2π` keep the labels off the curve. Items follow the order axis line, ticks (line, then label), axis label. Space axes reject `ticks` for now. Ticks are explicit rather than a `step`, so values such as `pi/2` with a fraction label are possible and a GUI can add or remove one tick at a time; a `step` can be added later.
- Grid (`grid` object, `Grid` in `scene.rs`, `compile_grid`, `grid_items`): plane figures only. `x_step` gives vertical lines and `y_step` horizontal ones (at least one is required; a number or an expression with parameters and constants; positive and finite). Lines are at the integer multiples of the step counted from 0 that lie in the view, ends included (with a tolerance of 1e-9 in the quotient). `line` is `dotted` by default, and the width is 0.3 pt, thinner than axes and ticks. More than 200 lines in one direction is an error, so a mistyped step cannot produce thousands of `\draw` lines. The lines are drawn in scene order (vertical, then horizontal), so a grid placed before the axes lies under them. Grid lines have no colour yet: a light colour is the next thing to settle.
- Style (`Style` in `scene.rs`, `stroke_of` in `render.rs`): `style` of `graph`, `curve`, `axis`, `grid`, and `sphere` has `line` (`solid`, `dotted`, `dashed`), `hidden` (space figures), `color`, and `width`. Every field is optional, and an omitted field is not written back, so the defaults belong to the object type: the line is solid except for a grid (dotted), the width is 0.6 pt for axes and ticks, 0.8 pt for curves and outlines, and 0.3 pt for a grid. `color` is one of `gray`, `red`, `blue`, `green`, `orange`, `purple`; without it the line follows the text colour. SVG turns a name into `var(--figure-<name>)`, and `figure.css` defines each variable twice, for light and dark backgrounds, so a colour stays readable in both themes. TikZ gets an `xcolor` base name (`green` is `green!50!black`, `purple` is `violet`) after `line width`. `width` is a length such as `"1.2pt"` (0 < width ≤ 10 pt) and is stored in the IR in pt. The arrowhead size follows the axis width as in TikZ (`3pt + 4.5·lw`), and ticks use the colour and width of their axis. Hidden parts keep the colour and width of the visible ones.
- Points, vectors, segments (`point`, `vector`, `segment`; plane and space figures; `compile.rs`, `point_items`, `link_item`): a `point` has `at` (two numbers or expressions), optional `label` (a formula, `anchor` defaults to `south west`, so the name is at the upper right of the point), `dot` (a filled circle of radius 2 pt, IR item `dot`, TikZ `\fill`), and `style.color`. A `vector` (default arrow `stealth`) and a `segment` (no arrow) join two points named by `id`; an unknown id, or the id of something that is not a point, is `unknown_point`. A zero-length link draws nothing. Vectors and segments may come before the points they name, because their end coordinates are looked up after every point is evaluated.
- Dependent points and names in expressions (`compile.rs`): after a `point` is evaluated, `<id>_x` and `<id>_y` become names that the objects placed after it can use in any expression (another point, a graph, a label position, a tick, a grid step). The names follow the parameters in the value list (`Compiled.parameters`), and an object sees only a prefix of that list, so a point cannot use its own coordinates or a later point's (`unknown name`) and there are no cycles. A clash between such a name and a parameter or a variable is `name_conflict`. A `label` position (`at`) is a list of numbers or expressions, so a name can sit at a midpoint.
- Point expressions (`Position::Vector`, `evaluate_vector`, `Expr::point_kind`): the `at` of a `point` or a plane `label` may also be one string that treats point ids as position vectors from the origin: `"B + C - A"`, `"(A + B) / 2"`, `"A + t * (B - A)"`. Only sums and differences (point with point, number with number), a number times a point, and a point divided by a number are allowed; a product of two points, a point plus a number, a point inside a function, a power, or a divisor, and an expression with no point are rejected with an error that names the rule. It is evaluated once per component with the point ids bound to the x values and to the y values, which is exact because the allowed operations are linear. Parameters are numbers here, so a slider moves a point along a segment. The names are those of the points placed before, as for `<id>_x`; a point id must not be a function or constant name (`e`, `pi`, `sin`). In a space view the vectors have three components. Moving a point in a GUI changes its `at`, and every dependent point follows when the scene is rendered again.
- Region and hatching (`region` object, `region.rs`, `hatch.rs`; plane figures only): a `region` names one or two `graph` ids in `between` (one means between the graph and the x axis) and an x range `domain` (numbers or expressions, inside the domain of each named graph; the check runs in `compile.rs`, and a graph may come after the region). The boundary is a polygon: the first graph sampled forward over the range, then the second graph backward (or the x axis segment). `hatch_lines` fills it with parallel lines at `angle` (degrees from the x axis, default 45) and `gap` (default 3 mm) in cm space, so the angle is the drawn angle; a line sits where the coordinate across it is a multiple of `gap`, counted from the origin, and the inside is decided by the even-odd rule with half-open edge crossings, so crossing curves, concave shapes, and vertex hits all work, and the start or direction of the polygon does not change the lines. Each hatch line is clipped to the view and drawn as a `Path` (0.4 pt, solid, `style` for line, colour, width). A graph that is broken inside the range (a pole, a value over 300 cm) has no single polyline, so rendering fails with an error on the region. The outline of the region is not drawn: the graphs are separate objects. Filling: `fill: {color?, opacity?}` (opacity in (0, 1], default 0.25; the colour defaults to `style.color`, then the text colour) adds one IR item `fill` (a closed polygon in cm, clipped to the view by Sutherland–Hodgman `clip_polygon`) before the hatch lines, and `hatch: false` drops the hatch. SVG draws it as a closed path with `fill-opacity` and no stroke, and TikZ as `\fill[color, opacity=…] … -- cycle;`. A concave region whose clipped part falls into separate pieces is drawn as one polygon joined along the view edge.
- Arrowhead (`arrow.rs`): `Stealth`, ported from the pgf source (see Verified facts). Its outline is tested against an independent computation of the mitered outline.
- Intermediate representation (`figure.rs`, `render.rs`): a `Figure` of `Path` items (points in cm, y up, stroke width in pt, an optional arrow with its polygon and the point where the shaft stops) and `Label` items (position in cm, anchor, TeX text). Items follow the order of the scene objects. An axis becomes a path (axis width 0.6 pt) and, when it has a `label`, a label `$name$` at its end (`west` for x, `south` for y). Curves are 0.8 pt. `bounds` is the view plus a 0.6 cm margin.
- TikZ (`tikz.rs`): `export_tikz(json)` writes `\draw` and `\node` lines with numbers to four decimals, arrows as `-{Stealth}`, and the scene JSON as written, in a header comment; `extract_scene` recovers it. The output for the first figure is compared with `tests/golden/sine-and-shifted-sine.tikz` (regenerate with `UPDATE_GOLDEN=1 cargo test -p figure --test tikz` and read the diff). The Wasm build must produce the same text: `site/src/figure/wasm.test.ts` compares it with the same file.
- Wasm (`crates/figure-wasm/`): `parseScene` and `renderScene`, with typed `SceneOutcome`, `RenderOutcome`, and `Figure`. An error has `code`, `message`, `object`, `line`, `column`, and, for expressions, `field`, `index`, `start`, `end`.
- SVG (`site/src/figure/svg.ts`, `label.ts`): the IR becomes a `<figure>` with an `<svg role="img" aria-label>` (viewBox in cm, y flipped, stroke widths converted from pt, `dotted` and `dashed` with the TikZ dash lengths, arrowheads as filled and stroked polygons with mitered joins) and, over it, one absolutely positioned `<span class="figure-label">` per label. The label's TeX (text with `$…$`) becomes one formula, `\text{…}` for the text parts, and is placed as inline math, which the MathJax step renders. The box moves by the anchor (a west label has its left edge on the point) and has a 1/3 em padding, like a TikZ node. `aria-hidden` is set on labels because the SVG carries the description.
- MDX (`site/src/plugins/rehype-figures.ts`): `<Figure src="name" />` is replaced by the figure; `name` is lowercase letters, digits, and hyphens, and names `site/src/figures/<name>.json`. The plugin loads the Wasm with `initSync` when the config is read, so `mise run wasm` must have run. It runs before `rehypeMathjax`. A scene error fails the build with the figure name and the object `id`. The dev server does not watch the scene files: touch the MDX file to see a change.
- Pages: `dev/figure-first.mdx` shows the first figure, and `dev/figure-scene.mdx` edits a scene and shows the result, the errors, and the TikZ. Both are linked from the home page. `e2e/figure.spec.ts` measures the positions of the labels against the axes, the physical size (1 cm is 37.8 px), the layout at 390 px, and the colours in both themes.

Known limits, the first things to extend:

- No automatic ticks (`step`); the axes cross at 0.
- A region's boundary must be graphs `y = f(x)`, not curves or implicit curves. Surfaces and spheres cannot be filled: shading a mesh needs face ordering and would make a large TikZ file.
- The arrowhead is always `Stealth`; the filled triangle of the KeTCindy vector sample (`s0105vector`) needs another tip (`Latex` from the same pgf source).
- A point mark has a fixed radius (2 pt), and a point or vector outside the view is not clipped (the SVG area is the view plus a margin, so it is cut off).
- Curves are polylines in the TikZ output (about 100 points per period); Bézier fitting would make them smaller and smoother.
- The label font is the MathJax font, not the document font of the TikZ output.
- Label text has no colour, and the palette is fixed to six names.

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
- Surfaces given by expressions (`surface` object, `surface.rs`, `space.rs`): `vars` (two names), `expr` (x, y, z), `domain` (two ranges), `mesh` (default 48 by 48, 4 to 200), `boundary` (default false), and `style`. The surface becomes a grid of triangles. Visibility uses the parallel projection: a point is hidden when its screen position is inside the projected triangle of some mesh and that triangle is nearer the camera. The point is first moved toward the camera by twice the largest gap between the mesh and the true surface (measured at the cell centres), so a point on the surface counts as outside it. This is checked against a sphere written as a surface (the axis pieces switch where the analytic sphere switches, within 0.03 units) and against a height-field oracle (`z = x^2 + y^2`, ray marching). Outline: the vertex normals are the sum of the adjacent face normals, merged over vertices at the same position (seams and poles), and the outline is where the normal is perpendicular to the view direction, found by linear interpolation along the mesh edges and chained through shared edges; ends that meet within 1e-7 of the scale (a seam) are joined, and a closed outline is closed exactly. The outline's own visibility cannot use the plain test, because near the rim the front layer of the same mesh is deeper than the offset (its depth grows with the square root of the distance inside the rim); a rim point is hidden only when both points shifted along the surface normal, to either side, are hidden. Curves, axes, and rims of other surfaces are hidden by every mesh. `boundary: true` draws the non-degenerate edges of the domain; a surface whose edge is a seam or a point (a sphere) should leave it false, and a rim that is a circle is better written as a `curve`. Speed: the triangles are bucketed in a uniform grid over the screen (about one cell per triangle, at most 256 by 256), and a visibility query looks only at the triangles of the cell that holds the point. `Mesh::hides_exhaustive` scans all triangles and is the reference: `tests/surface.rs` checks that both give the same answer for scattered points on a sphere and a bowl at several view directions.
- Bézier surfaces (`bezier` field of a `surface`, `bezier.rs`): a control net `bezier[i][j]` (`i` along the first variable, `j` along the second; each point three numbers or expressions, parameters allowed) replaces `vars`, `expr`, and `domain`, and the ranges are both 0 to 1. The net needs 2 to 12 points in each direction and equal row lengths (`invalid` otherwise, also when both forms are given or neither). A point is evaluated by de Casteljau's method. Everything else (mesh, outline, hiding, cuts, intersections, Newton polishing) works on the same `SurfaceMap` closure as for expression surfaces, so a Bézier surface is used the same way (`tests/bezier.rs` compares it with the same shape written as expressions). Closed surfaces of arbitrary shape are not covered: a Bézier patch is a sheet, and a sphere is the only closed surface besides expressions.
- Plane cuts (`cut` object, `Mesh::cut`, `cut_items`): `surface` names a `surface` (an unknown id, or something else, is `unknown_surface`), and the plane is `normal · p = offset` with three expressions in `normal` (a nonzero finite vector) and an expression in `offset`, so a parameter can move the plane. The mesh gets a per-vertex value `normal · p - offset`, and the zero level curve is found by the same routine as the silhouette (`level_curves`), which is why the cut lies exactly on the mesh. The cut is hidden by every mesh, its own included, with the same offset toward the camera as any point on a surface. It is checked against a circle written as a `curve` on an analytic sphere (same dotted length, within 0.15 cm), against the circle in a cone, and by covering the expected ellipse to within 0.03 cm. A plane that misses the surface draws nothing and is not an error. Cutting a `sphere` object is not supported; write the sphere as a `surface`.
- Points, vectors, segments in space (`point_items`, `link_items` in `space.rs`; `PointPlot.at` and `LinkPlot` hold two or three numbers): a space `point` has three coordinates or a point expression evaluated in three components, and its names are `<id>_x`, `<id>_y`, `<id>_z`. A vector or segment is split by visibility like an axis (64 steps, then bisection): hidden parts use `style.hidden`, and the arrowhead is drawn only when the end is visible. A point mark that is hidden by a ball or a surface is not drawn, since a filled disc cannot be dotted; its label is always drawn. Labels take three numbers or a point expression in a space view too.
- Surface intersections (`intersection` object, `Mesh::intersection`, `triangle_intersection`): `surfaces` names two different `surface` ids (`unknown_surface` otherwise). Every triangle of the first mesh is tested against the triangles of the second that share a cell of a uniform 3D grid (cells per side about the cube root of the triangle count, at most 64). Two triangles meet in a segment on the line where their planes cross: each triangle is cut by the other's plane (vertices within 1e-12 of the plane count as on it), and the overlap of the two intervals along the line is the segment; coplanar or parallel pairs give nothing. The segment ends are merged through a spatial hash into nodes (tolerance 1e-7 of the scale), and the nodes are chained (ends first, then rings) and joined like the silhouette. The curve lies on both meshes, so hiding uses the same offset toward the camera as a cut. Checked against two spheres of radius 2 with centres 2 apart (a circle of radius sqrt(3) in the plane x = 1: every point within 0.03 cm of the projected circle, and the dotted length within 0.3 cm of the same circle hidden by analytic balls), and rendered for a sphere and a cylinder (Viviani-type curve). Tangent contact and coplanar overlap are not handled. Cost: the pair test is limited by the grid, but the first mesh is still scanned fully.
- Smoothing of cuts and intersections (`refine.rs`, `Curve`, `Node`; used by `Mesh::cut` and `Mesh::intersection`, which now also take the exact surface functions): a curve found on the meshes has its vertices on mesh edges, so it leaves the true surface by the mesh gap (about 0.004 for a sphere of radius 2 at 48 divisions) and it is jagged where two surfaces are nearly tangent (the node of a Viviani curve, where a sphere and a cylinder touch). The curve is described by parameters `x` (`(u, v)` for a cut, `(u, v, s, t)` for an intersection) and a residual that is zero on the curve (`normal · S(u, v) - offset`, or `S_A(u, v) - S_B(s, t)`). Each point carries the parameters of the two triangles it came from (barycentric mix of the vertex parameters), and three steps follow, in this order: (1) polish: Gauss–Newton with a numeric Jacobian moves `x` by the minimum step that zeroes the residual (`Jᵀ(JJᵀ + λI)⁻¹F`, the damping `λ` keeps it stable at a tangent point), the result is used only if it converged (residual under 1e-10 of the scale) and moved less than an eighth of the scale, so a point never jumps to another branch; (2) thin: Douglas–Peucker with half the chord tolerance (1e-3 of the scale) drops the vertices that a straight line replaces, so the vertex count no longer follows the mesh size; (3) refine: for each pair of neighbours the midpoint of the parameters is polished the same way, and if it is farther from the chord than the tolerance (and not farther than half the chord from the chord midpoint, again a guard against branch jumps) it is inserted and the two halves are refined recursively, up to depth 5. The vertices are therefore on both exact surfaces (about 1e-10), the chord midpoints are within 1e-3 of the scale of the surface, and the vertex density follows the curvature: for the Viviani curve the count is about the same with meshes of 24 and 96 divisions. Chains are polished and refined before they are joined across a seam of the parameter domain, since the parameters jump there (`t = 0` and `t = 2π`). The silhouette is not polished (its residual needs second derivatives of the surface); its error stays at the mesh gap. Near the tangent point itself the polish can fail and the original vertex stays, which leaves a small kink.
- Not done: hidden-line removal between curves and non-spherical surfaces, ticks, perspective.

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
    { "id": "origin_label", "type": "label", "at": [0, 0], "anchor": "south east", "tex": "$O$" },
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
- `axis`: one object per axis, so a single axis or a half axis (`range: [0, 5]`) is possible. `direction` is `x` or `y` (`z` in 3D). `arrow` is `stealth` by default and can be `none` or another tip. `range` defaults to the view. `ticks` is a list of `{at, label}` (see Implemented).
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

## Known algorithms (looked up, not adopted)

Read on 2026-09-22. Vector hidden-line removal is classically done with Appel's quantitative invisibility: each projected curve carries a count of the front faces in front of it, and the count changes only where the curve crosses the projected silhouette or an edge, so visibility is not tested point by point. Robust variants exist for triangulated freeform surfaces and use silhouette curves to split the surface, and analytic versions exist for quadrics (planes, cylinders, cones, tori, spheres). Acceleration structures such as a bounding volume hierarchy or an octree with a Z pyramid reject hidden geometry quickly. The engine does not use quantitative invisibility now: it tests points against the mesh and refines each switch by bisection, which needs no edge-crossing bookkeeping and handles curves that lie on a surface, and a screen-space grid is enough for the mesh sizes used here. Quantitative invisibility remains the choice if a switch point must be exact (it would come out at a silhouette crossing) or if scenes grow far larger.

## Verified facts

### TikZ output compiled and compared with the SVG

Checked on 2026-09-22 with TeX Live 2026 (pgf 3.1.12, LuaHBTeX 1.24), `standalone` with `luatexja-preset`, and the figures on `figure-first` and `figure-space`.

- Every figure compiles without an error. The picture size equals the engine's bounds (the view plus margin), up to rounding.
- Lines, arrowheads, dots, fills, dashes, colours, and opacity agree with the SVG: with the labels hidden on both sides, no ink in either image lies more than 2 px (0.5 mm) from ink in the other, at 3x resolution (0.0 % of the ink for every figure).
- Labels agree when their size agrees. A TikZ node is 10 pt (`\normalsize`), so the label size in `figure.css` is 10 pt; with the page's 16 px (12 pt) every label was 30 % larger and the mismatch was 3 to 13 %, against 0 to 8 % now. The MathJax font and Computer Modern look alike at this size.
- Known difference: the vertical position of a label anchored `north` or `south` differs by up to about 4 px. The HTML label box has a fixed line height (10 pt), while a TikZ node box is as tall as the glyphs of its text (a label with a descender such as `y` is taller than `x`), so the anchored edge sits at a different height. Fitting it would need the height and depth of each label from MathJax.
- The figures of `dev/figure-algorithms.mdx` are not part of `mise run tikz`. Checked by hand: the paths match, and every figure compiles (a label with `\boldsymbol` needs `amsmath`), but `projection-screen` and `projection-staircase` exceed the label limit (about 20%) although the PDF and the SVG look the same: the labels differ by about 2 px vertically. Giving the nodes a fixed `text height` and `text depth` (7.5 pt and 2.5 pt) brought `sphere-with-axes` from 8% to 0% but did not change these two, so the cause of the remaining offset is not found yet.
- The check wraps the product TikZ in a `standalone` document and adds only a bounding box (`\useasboundingbox`, the view plus margin) and, for the split kinds, options that hide one half. The product itself is not edited.

### Stealth arrowhead

Read on 2026-09-21 from the master branch of the pgf repository: the declaration of `Stealth` in `tex/generic/pgf/libraries/pgflibraryarrows.meta.code.tex`, and the dimension setup in `tex/generic/pgf/basiclayer/pgfcorearrows.code.tex`.

- With `lw` the line width of the path, the defaults are length `L = 3pt + 4.5·lw`, width `W = 0.75·L`, and inset `I = 0.325·L`. The third number in `length = +3pt 4.5 .8` only applies to double lines. At the default line width of 0.4pt this gives L = 4.8pt, W = 3.6pt, and I = 1.56pt.
- The tip is a closed path of four points (tip, upper back corner, inset point, lower back corner), filled and stroked with mitered joins. The stroke width is `lw' = min(lw, (L − I) / 4)`. The points are pulled inward by the miter lengths: at the tip `0.5·lw'·sqrt(4(L/W)² + 1)`, at the inset point `0.5·lw'·sqrt(4(I/W)² + 1)`, and at the back corners by an angle formula in the source.
- The visual tip is at `x = L` from the back of the arrow. The line ends at `I + (inset miter) − 0.25·lw'` from the back, so it does not show through the tip.
- pgf computes the corner miters from the nominal shape (length, width, inset), not from the mitered path, so the real outer corner protrudes past the L × W box by about a tenth of the line width (measured: 0.028 pt at 0.4 pt). The port keeps this; its test allows a tenth of the line width.
- Port the code from the source, not from this summary. The port is compared with real TikZ output by `mise run tikz` (see the next section): the paths, arrowheads included, agree with LuaLaTeX and pgf 3.1.12 within 2 px at 3x resolution.

### Dash patterns

From `tex/generic/pgf/frontendlayer/tikz/tikz.code.tex` (master, read on 2026-09-21): `dotted` is `dash pattern=on \pgflinewidth off 2pt`, and `dashed` is `dash pattern=on 3pt off 3pt`. The SVG output uses the same lengths.

## Open

- The figures required after `s0202graph1`, and the order of the extensions listed under Implemented.
- How a figure is captioned in an article.
