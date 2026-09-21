<!-- rtk-instructions v2 -->
# RTK (Rust Token Killer) - Token-Optimized Commands

## Golden Rule

**Always prefix commands with `rtk`**. If RTK has a dedicated filter, it uses it. If not, it passes through unchanged. This means RTK is always safe to use.

**Important**: Even in command chains with `&&`, use `rtk`:
```bash
# ❌ Wrong
git add . && git commit -m "msg" && git push

# ✅ Correct
rtk git add . && rtk git commit -m "msg" && rtk git push
```

## RTK Commands by Workflow

### Build & Compile (80-90% savings)
```bash
rtk cargo build         # Cargo build output
rtk cargo check         # Cargo check output
rtk cargo clippy        # Clippy warnings grouped by file (80%)
rtk tsc                 # TypeScript errors grouped by file/code (83%)
rtk lint                # ESLint/Biome violations grouped (84%)
rtk prettier --check    # Files needing format only (70%)
rtk next build          # Next.js build with route metrics (87%)
```

### Test (60-99% savings)
```bash
rtk cargo test          # Cargo test failures only (90%)
rtk go test             # Go test failures only (90%)
rtk jest                # Jest failures only (99.5%)
rtk vitest              # Vitest failures only (99.5%)
rtk playwright test     # Playwright failures only (94%)
rtk pytest              # Python test failures only (90%)
rtk rake test           # Ruby test failures only (90%)
rtk rspec               # RSpec test failures only (60%)
rtk test <cmd>          # Generic test wrapper - failures only
```

### Git (59-80% savings)
```bash
rtk git status          # Compact status
rtk git log             # Compact log (works with all git flags)
rtk git diff            # Compact diff (80%)
rtk git show            # Compact show (80%)
rtk git add             # Ultra-compact confirmations (59%)
rtk git commit          # Ultra-compact confirmations (59%)
rtk git push            # Ultra-compact confirmations
rtk git pull            # Ultra-compact confirmations
rtk git branch          # Compact branch list
rtk git fetch           # Compact fetch
rtk git stash           # Compact stash
rtk git worktree        # Compact worktree
```

Note: Git passthrough works for ALL subcommands, even those not explicitly listed.

### GitHub (26-87% savings)
```bash
rtk gh pr view <num>    # Compact PR view (87%)
rtk gh pr checks        # Compact PR checks (79%)
rtk gh run list         # Compact workflow runs (82%)
rtk gh issue list       # Compact issue list (80%)
rtk gh api              # Compact API responses (26%)
```

### JavaScript/TypeScript Tooling (70-90% savings)
```bash
rtk pnpm list           # Compact dependency tree (70%)
rtk pnpm outdated       # Compact outdated packages (80%)
rtk pnpm install        # Compact install output (90%)
rtk npm run <script>    # Compact npm script output
rtk npx <cmd>           # Compact npx command output
rtk prisma              # Prisma without ASCII art (88%)
rtk uv run <cmd>        # Compact uv project command output
```

### Files & Search (60-75% savings)
```bash
rtk ls <path>           # Tree format, compact (65%)
rtk read <file>         # Code reading with filtering (60%)
rtk grep <pattern>      # Search grouped by file (75%). Format flags (-c, -l, -L, -o, -Z) run raw.
rtk find <pattern>      # Find grouped by directory (70%)
```

### Analysis & Debug (70-90% savings)
```bash
rtk err <cmd>           # Filter errors only from any command
rtk log <file>          # Deduplicated logs with counts
rtk json <file>         # JSON structure without values
rtk deps                # Dependency overview
rtk env                 # Environment variables compact
rtk summary <cmd>       # Smart summary of command output
rtk diff                # Ultra-compact diffs
```

### Infrastructure (85% savings)
```bash
rtk docker ps           # Compact container list
rtk docker images       # Compact image list
rtk docker logs <c>     # Deduplicated logs
rtk kubectl get         # Compact resource list
rtk kubectl logs        # Deduplicated pod logs
```

### Network (65-70% savings)
```bash
rtk curl <url>          # Compact HTTP responses (70%)
rtk wget <url>          # Compact download output (65%)
```

### Meta Commands
```bash
rtk gain                # View token savings statistics
rtk gain --history      # View command history with savings
rtk discover            # Analyze Claude Code sessions for missed RTK usage
rtk proxy <cmd>         # Run command without filtering (for debugging)
rtk init                # Add RTK instructions to CLAUDE.md
rtk init --global       # Add RTK to ~/.claude/CLAUDE.md
```

## Token Savings Overview

| Category | Commands | Typical Savings |
|----------|----------|-----------------|
| Tests | vitest, playwright, cargo test | 90-99% |
| Build | next, tsc, lint, prettier | 70-87% |
| Git | status, log, diff, add, commit | 59-80% |
| GitHub | gh pr, gh run, gh issue | 26-87% |
| Package Managers | pnpm, npm, npx | 70-90% |
| Files | ls, read, grep, find | 60-75% |
| Infrastructure | docker, kubectl | 85% |
| Network | curl, wget | 65-70% |

Overall average: **60-90% token reduction** on common development operations.
<!-- /rtk-instructions -->

# math-study

A study site for mathematics (Astro 7, Starlight, MDX). `README.md` (Japanese) gives the overview. `docs/tech-decisions.md` records the technical decisions and the facts verified along the way.

## Working conventions

- This is a solo project: work on `main` only. Do not use branches or pull requests. Commit finished work and push it to `main`, and split unrelated changes into separate commits.
- Pushing publishes the site through GitHub Pages. After a push, confirm that the GitHub Actions `build` and `deploy` jobs succeed.
- After a change, run `mise run check`. For changes that affect rendering or behavior, also run `mise run e2e` and check the appearance with `mise run screenshot`. The procedure is in the `verify-site` skill.
- Record what you learn in the same change, so the next session and the human maintainer can find it. Put durable knowledge where it belongs: `CLAUDE.md` for conventions and pitfalls, `.claude/skills/**` for procedures, `docs/tech-decisions.md` for decisions and verified facts, and `README.md` plus `site/src/content/docs/dev/internals.mdx` (Japanese, for humans) for how the system works. Do not leave findings only in the conversation or the scratchpad. If a change alters who does what, what is custom-built, or how a feature is used, update `internals.mdx` and the "Code map" below in the same commit. Skip this only when the information is already recorded.
- Read the `write-content` skill before writing site content or documentation.
- The figure feature (`crates/figure/`, design in `docs/graph-design.md`) is developed test-first: write the failing test, see it fail, then write the code that makes it pass.
- Run `finalize-artifacts` before reporting a deliverable as done (see "Artifact Cleanup" below).
- Prefix commands with `rtk`. The hook is in `.claude/settings.json`; do not install it in the user-level settings.
- `git commit` runs lefthook hooks. If they fail, fix what they report. Never use `--no-verify`.
- Before `git add -A`, read `git status --short`: check the number of entries and the paths. A pnpm store (`.pnpm-store/`, 26,000 files) once slipped into a commit and a push because its location changed after a container rebuild. It is git-ignored now. Rewriting history and force-pushing need the user's confirmation.

## Language

- Write everything meant for Claude in English: `CLAUDE.md`, `.claude/skills/**`, and `docs/**`. Only `README.md` and the site content (`site/src/content/**`) are in Japanese. Code comments and commit messages are also in Japanese.
- Japanese text follows these rules. textlint enforces them on the site content and `README.md`, in CI and in the pre-commit hook.
  - Use the plain form (である調), not ですます.
  - Use the full-width comma "，" (U+FF0C) and full-width period "．" (U+FF0E). Never use "、" or "。".
  - Put no space between Japanese and Latin text, digits, or inline code ("MDXの記法", not "MDX の記法").
  - Use a full-width colon "：" after Japanese text, with no space after it.
  - Avoid AI-sounding writing: bold-prefix bullet lists, hype, and emphasis in ordinary sentences.

## Pitfalls

- In agent environments (`CLAUDECODE=1`), `astro preview` starts a background server and the command itself exits. E2E tests use `e2e/serve.ts` instead. If you start a preview by hand, manage it with `astro preview status` and `astro preview stop`.
- Type-aware linting (oxlint) needs the type definitions that `astro sync` generates. `mise run lint` runs `sync` first.
- In the container, `mise.toml` is read as the global config, so `mise lock` needs `--global`.
- Keep `typescript` on the 6.x line: `astro check` does not support TypeScript 7.
- pnpm 12 allows dependency build scripts only when listed. Add them to `allowBuilds` in `pnpm-workspace.yaml`.
- Write the textlint config in `.textlintrc.yml`. In MDX, `textlint-disable` comments do not work, so rewrite the text to avoid a false positive.
- oxfmt does not format `.astro` files. Put logic in `.ts` files and keep `.astro` files thin.
- The Playwright browsers live in `/opt/ms-playwright`. If the version of `@playwright/test` in `package.json` drifts from the installed browsers, run `mise run browsers`.
- The test page (`site/src/content/docs/dev/notation.mdx`) is hidden from the sidebar and search. Update it whenever the notation changes.
- Math (MathJax 4) is rendered at build time by `site/src/plugins/rehype-mathjax.ts`. MathJax itself runs in a native Worker thread (`site/src/math/worker.ts`), not through Vite: the plugin module is loaded by the short-lived Vite runner that reads `astro.config.ts`, and a lazy `import()` after that runner closes fails with "Vite module runner has been closed". Terminating the Worker (`astro:build:done`, `astro:server:done`) is also what lets the process exit; without it, MathJax's speech threads keep it alive.
- `mise run e2e` also runs axe and a link/console-error crawl over every page in `site/dist` (`e2e/accessibility.spec.ts`, `e2e/site.spec.ts`), and CI runs `mise run e2e` in the `build` job before `deploy`. Any axe violation fails. A scrollable region must be keyboard-focusable: display math and tables get `tabindex="0"` from plugins, and code blocks wrap. Visual regression of the site is deliberately not automated, and only Chromium is tested. The exception is the TikZ output: `mise run tikz` compares it with the SVG figures, and `mise run tikz:export` only writes it out as `.tikz`, `.tex`, PDF, and PNG (both local only, see the `verify-site` skill). Details are in the `verify-site` skill.
- Do not inline the MathJax CSS with a `<style>` element: the MDX output HTML-escapes its text (`&#x22;`), and browsers do not decode entities in `<style>`. The CSS is one shared file, `mathjax.css`. It is written to `dist/` at `astro:build:done` and served by a dev-server middleware; pages with math get a `<link>` to it. The fonts are copied to `site/public/mathjax-fonts/` (git-ignored) at config setup. The CSS holds one size rule per glyph that some page used (`mjx-c.mjx-c1D465 { … }`), and `mjx-c` is clipped to its padding box (`clip-path`). A browser that keeps an older `mathjax.css` next to newer HTML (GitHub Pages sends `max-age=600`) therefore draws the new glyphs as clipped slivers. The link carries `?v=<time of the config load>` (`stylesheetUrl` in `astro.config.ts`) and the dev middleware sends `no-store`; keep both, and keep the query out of any URL comparison (`e2e/math.spec.ts` checks it).
- The browser MathJax (the calculator) builds its font URL from `import.meta.env.BASE_URL`, which has no trailing slash (`/math-study`), so join with `/` (`fontUrl()` in `site/src/calculator/mathjax.ts`). Math that the browser typesets (`renderMath` in `site/src/calculator/mathjax.ts`) gets the `not-content` class too, for the same margin reason as the build-time math (`e2e/calculator.spec.ts` checks it). Fonts are requested only after the first formula is typeset, so a test must wait for `document.fonts.ready` before it checks for failed requests (`e2e/calculator.spec.ts`).
- Undefined macros and TeX syntax errors fail the build, with the file line and the TeX source in the message. MathJax's `TexError` does not extend `Error`, so read its `message` property instead of using `instanceof Error`.
- Starlight adds `margin-top: 1rem` to every element that follows a sibling inside `.sl-markdown-content` (`markdown.css`), unless the element is inside a `.not-content` ancestor. MathJax's CHTML output is made of adjacent custom elements (`mjx-num` and `mjx-dbox` in a fraction, `mjx-mo` and `mjx-script` in a superscript), so without the exclusion, fractions, limits, and proof-tree labels are pulled apart. `rehype-mathjax.ts` adds the `not-content` class to each `mjx-container`, and `e2e/math.spec.ts` fails if any element inside one gets a 16px top margin. Do the same for any other component that renders adjacent custom elements.
- Add math macros as a module under `site/src/math/macros/modules/`, one file per area, default-exporting a `MacroModule` (add to the file of the same area, or create a new one). `macros/index.ts` collects the files with `import.meta.glob`, so there is no registry to edit, and `merge.ts` fails the build when two modules define the same name. Update the list on the test page. The number-set macros follow the LaTeX `numbersets` package (`\NaturalNumbers`, `\Integers`, `\RationalNumbers`, `\RealNumbers`, `\ComplexNumbers`, and `\NumberSet[style]{X}` with the styles `bb`, `bfup`, `bfit`). MathJax has no `\csname`, and macro expansion inserts a space between a control sequence and a following letter, so the style is selected through an environment name (`\begin{numbersetstyle#1}`), which is why `macros.ts` also exports `environments`.
- Definitions and theorems (`<Definition>`, `<Lemma>`, `<Proposition>`, `<Theorem>`, `<Corollary>` from `@/components/statement`) are numbered at build time by `site/src/plugins/rehype-statements.ts`. The label is kind + page identifier + sequence number (`定理abs-3`), counted per page across kinds. The page identifier is the frontmatter `pageId` or the file name (`index.mdx` uses the directory name). It must be unique across the site and use only letters, digits, hyphens, and underscores, starting with a letter or digit. Use `pageId` as a short name when the file name would make labels long (`pageId: cont` gives `定理cont-7`); labels and `<Ref page>` both use it. `<Ref to="id" page="pageId" />` becomes a link and `<Proof of="id">` sets the proof title; `Ref` needs no import because the plugin replaces it. Other pages are found by scanning `site/src/content/docs/**/*.mdx` (`plugins/statements/catalog.ts`), so pages referenced from other pages need lowercase ASCII file names. Bad identifiers, duplicates, and dangling references fail the build. Display formulas with `\label{id}` are numbered by the same plugin as `(pageId-n)` and referenced with `<Ref>` (`式(pageId-n)`); `\ref` and `\eqref` in math are rejected. Do not read the position of a display formula from `<code>`: only the wrapping `<pre>` has one. To add a statement kind, extend `STATEMENT_KINDS` in `statements/collect.ts`, and add a wrapper component.
- oxfmt ignores `site/src/content/**/*.mdx`: it re-wraps paragraphs that contain inline JSX such as `<Ref />`, which splits inline math and inserts line breaks between Japanese words. Format MDX by hand, and keep each paragraph on one line.
- MDX is linted by remark-lint (`.remarkrc.mjs`, `mise run lint:mdx`), not markdownlint. remark parses MDX like the build does, so unclosed tags and broken `{…}` expressions fail at commit time, and components need no registration. Keep its remark plugins (`remark-mdx`, `remark-math`) in step with `astro.config.ts`: without `remark-math`, `{…}` inside `$$…$$` is read as a JSX expression. remark-cli prints a re-serialized copy of a single input file to stdout and rewrites some text (`[x]` becomes `\[x]`), so always pass `--no-stdout` and never use `--output`. markdownlint still checks `README.md`, `docs/`, and any plain `.md` in the content.
- Source text has no spaces between Japanese and Latin letters, digits, inline code, or math (textlint enforces it); typesetting adds a 1/8em gap, like TeX's `\xkanjiskip`, without changing the DOM text. `site/src/styles/typesetting.css` sets `text-autospace: normal` for text and inline code (no gap next to punctuation). Inline math is an atomic box that `text-autospace` ignores (measured: gap 0), so `site/src/plugins/rehype-autospace.ts`, which runs after `rehypeMathjax` (see `pipeline.ts`), adds `autospace-before` and `autospace-after` to an inline `mjx-container` when its neighbor is Han, Hiragana, Katakana, or `ー`, and the CSS gives those classes a 0.125em margin. Code blocks and the inside of math are `no-autospace`. `e2e/typesetting.spec.ts` measures the gaps.
- A `<Figure>` with an `id` is numbered like a statement, but with its own counter: `rehype-statements.ts` gives it the label `図<pageId>-<n>` and passes `label` and `anchor` to the element, and `rehype-figures.ts` turns them into the `id` of the `<figure>` and a `<figcaption>` with the number and the `caption` (which may contain `$…$`). `<Ref to="id" />` links to it. Figure references work only within the same page (the catalog of other pages holds statements and equations only), and `<Proof of>` rejects a figure id. A figure without `id` is unchanged. In prose, refer to a figure with `<Ref>`, never with a vague phrase such as "図の".
- Math macros are named by meaning in the numbersets style (`\RealNumbers`). Do not add short aliases such as `\R`.
- `\colored{red}{x}` colors part of a formula with a figure color (`gray`, `red`, `blue`, `green`, `orange`, `purple`): the `html` extension of MathJax (`worker.ts`) adds the class `math-color-red`, and `figure.css` maps it to the palette variable, which is defined on `:root` so that math and figures share it and both follow the dark theme. It works in build-time math only; the browser MathJax (the calculator and the projection island) does not load the extension.
- Rust and WebAssembly: the polynomial calculator core is in `crates/`, built by `mise run wasm` into the git-ignored `site/src/wasm/`, which `dev`, `build`, `sync`, and `test` depend on. Read the `rust-wasm` skill before touching it. The Rust toolchain and the `wasm-bindgen` CLI come from `mise.toml` (the CLI through the `http` backend, to avoid the GitHub API rate limit) and their versions are pinned in `mise.lock`; the CLI version must equal the `wasm-bindgen` crate version. Native `cargo test` needs `gcc`, which the Dockerfile installs (a running container needs `apt-get install gcc libc6-dev` until it is rebuilt).
- TeX Live (the official `install-tl`, scheme infraonly plus LaTeX, LuaLaTeX, pgf, standalone, LuaTeX-ja, and the Harano Aji fonts) is installed by `.devcontainer/install-texlive.sh`, which the Dockerfile runs; `poppler-utils` gives `pdftoppm`. It lives in `/opt/texlive/<year>` and links its programs into `/usr/local/bin`. A running container that predates the Dockerfile change needs `apt-get install perl poppler-utils` and then `sh .devcontainer/install-texlive.sh` as root (several minutes, about 250 MB). It installs the latest release, so the pgf version follows the year (2026: pgf 3.1.12, LuaHBTeX 1.24); recompile with `mise run tikz` after a rebuild. Figure labels are TeX that MathJax reads, so the comparison document loads `amsmath` (a label with `\boldsymbol` does not compile without it). Figure labels are 10 pt (`figure.css`), the TikZ default `\normalsize`, so that the SVG and the TikZ output look alike.
- Browser MathJax (the calculator) must switch enrichment, speech, and braille off through `menuOptions.settings`, because the menu settings override the `enable*` options; otherwise it starts a speech worker and fails. `e2e/serve.ts` serves `.wasm` as `application/wasm`; GitHub Pages already does.
- `rehype-figures.ts` loads `site/src/wasm/figure_bg.wasm` with `initSync` when `astro.config.ts` is read, so `mise run wasm` must have run before `astro build`, `astro check`, or the dev server (the mise tasks do it). Plugins imported by the config use relative imports, not the `@/` alias. `<Figure src>` names a file in `site/src/figures/` (lowercase letters, digits, hyphens); the dev server does not watch the scene files. The SVG uses `currentColor`, so figures follow the theme. The TikZ golden file (`crates/figure/tests/golden/`) is compared by both `cargo test` and `site/src/figure/wasm.test.ts`; regenerate it with `UPDATE_GOLDEN=1` and read the diff.
- Playwright's `boundingBox()` of an SVG path adds the stroke width times the miter limit (4), so a 1.6 cm circle measures 4 px too wide. Measure figure geometry with `getBoundingClientRect()` in `evaluate` (`geometry()` in `e2e/figure-space.spec.ts`).
- Speech strings are English only. Known quirks: `\norm` is read as "metric", and the `\Axiom…\fCenter…` form gets no speech.
- `mise run test` runs the Vitest unit tests (`site/src/**/*.test.ts`). `mise run check` includes them.

## Code map

Build-time pipeline: MDX → remark (`remark-math`) → rehype (`rehypeStatements`, then `rehypeMathjax`) → Astro components with Starlight → static HTML. `site/astro.config.ts` wires it together. The human-facing explanation is `site/src/content/docs/dev/internals.mdx`.

- `site/src/math/macros/`: TeX macros, one module per area in `modules/` (`number-sets.ts` also defines the `environments` that pick number-set styles), collected by `index.ts` and merged by `merge.ts`. `renderer.ts` is the main-thread client (Worker, request queue, moves the speech string to `aria-label`). `worker.ts` runs MathJax and holds its configuration.
- `site/src/integrations/mathjax.ts`: copies fonts to `public/mathjax-fonts/`, writes `mathjax.css` (and serves it in dev), and stops the Worker.
- `site/src/plugins/rehype-mathjax.ts`: replaces each formula with MathJax output, adds `not-content`, keeps the `id` set by the numbering step.
- `site/src/plugins/rehype-statements.ts`: the numbering and reference step. It uses `plugins/statements/`: `tree.ts` (node types, `DocumentError`), `collect.ts` (statement kinds, page identifiers, numbering), `equations.ts` (`\label` → `\tag`), `math-sites.ts` (finds formulas in hast and mdast), `scan.ts` (reads a whole MDX file for the catalog), `catalog.ts` (index of other pages, page URLs). `inspect.ts`, `test-support.ts`, and `test-processor.ts` are unit-test helpers.
- `site/src/components/fold/` (`Proof`, `Remark`, `Detail`, `auto-open.ts`) and `site/src/components/statement/` (`Definition`, `Lemma`, `Proposition`, `Theorem`, `Corollary`): the components. `Ref` has no component; the plugin replaces it.
- `site/src/content/docs/dev/continuity.mdx`: the reference article that uses every notation. The `dev/` pages are hidden from the sidebar and search but must stay reachable: link every new `dev/` page from the home page (`site/src/content/docs/index.mdx`). When real subject articles are added, give each subject a group in the `sidebar` of `site/astro.config.ts`; since Starlight 0.39 a group is `{ label, items: [{ autogenerate: { directory } }] }`, not `{ label, autogenerate }`. `site/src/content.config.ts`: the `pageId` frontmatter schema. `site/src/content/docs/dev/`: the hidden pages (`notation.mdx` notation reference, `abs.mdx` cross-reference target, `internals.mdx` explanation).
- `crates/polynomial/` (Rust: lexer, recursive-descent parser, evaluator, polynomial, TeX and text output, `calculate`), `crates/polynomial-wasm/` (the `wasm-bindgen` wrapper with the typed `Outcome`), `site/src/calculator/` (Wasm and browser MathJax loaders), `site/src/components/calculator/` (the React island), `site/src/content/docs/dev/calculator.mdx` (its page).
- `crates/figure/` (Rust: scene types, `parse_scene`, version policy, validation, `expr/` expression evaluator, `compile.rs`, `sample.rs`, `clip.rs`, `hatch.rs` and `region.rs` for regions, `arrow.rs`, the IR `figure.rs`, `render.rs`, `space.rs`, `surface.rs`, `bezier.rs` (Bézier surfaces), and `refine.rs` (polishing of cuts and intersections) for space figures and surfaces, `tikz.rs`), `crates/figure-wasm/` (the wasm wrapper with the typed `SceneOutcome`), `site/src/figure/` (Wasm loader, `svg.ts` and `label.ts` for the SVG and labels, the samples of the check page), `site/src/plugins/rehype-figures.ts` (`<Figure src>` → figure, before `rehypeMathjax`), `site/src/styles/figure.css`, `site/src/content/docs/dev/figure-first.mdx` (the first figure), `site/src/content/docs/dev/figure-space.mdx` (the space figures), `site/src/components/figure/` (the React island), `site/src/projection/` (`camera.ts`: the projection basis in TypeScript, checked against the engine in `camera.test.ts`; `scene.ts`: the scene of the interactive figure) with `site/src/components/projection/` (the sliders island that draws it in the browser through `site/src/figure/hast-react.tsx`, which turns the SVG tree into React elements), `site/src/content/docs/dev/figure-scene.mdx` (the check page), `site/src/content/docs/dev/figure-algorithms.mdx` and `figure-fill.mdx` (the mathematical principles of the algorithms, for readers who do not care about the implementation: projection, hiding, lines from surfaces, approximation, and the hatching of plane regions; update them when an algorithm changes), and `site/src/figures/` (the scene JSON files, one per figure, named after their content). The scene format and the plan are in `docs/graph-design.md`. The first figure and a minimal space figure (a sphere with axes) are drawn end to end; the extension list is in `docs/graph-design.md`.
- `site/src/plugins/pipeline.ts`: the rehype plugins in execution order, with the ordering constraints (numbering before math rendering, autospace after it). Add a new rehype plugin there, not in `astro.config.ts`.
- `site/src/plugins/rehype-autospace.ts` and `site/src/styles/typesetting.css`: the gap between Japanese and Latin text, digits, code, and inline math.
- `site/src/plugins/rehype-focusable-tables.ts`: gives tables `tabindex="0"` so a horizontally scrolling table is keyboard-accessible.
- `e2e/`: Playwright specs and helpers (`routes.ts` lists every page in `site/dist` for the all-pages checks). `.remarkrc.mjs`, `.textlintrc.yml`, `.oxlintrc.jsonc`, `lefthook.yml`, `mise.toml`: lint and task configuration.

## Skills

- `verify-site`: how to verify a change (build, lint, E2E, screenshots).
- `write-content`: how to write content and use the notation (writing rules, folding components, handling lint findings).
- `rust-wasm`: how the Rust and Wasm calculator works, and how to add or change it.
- `finalize-artifacts`: how to finish a deliverable.

# Artifact Cleanup

## Golden Rule

**Whenever you produce an artifact, always run the `finalize-artifacts` skill (`.claude/skills/finalize-artifacts/`) to clean it up before reporting the work as done.**

An artifact is any deliverable you create or substantially rewrite: documents, READMEs, code and code comments, config files, scripts, commit messages, PR descriptions, and so on.

- Invoke the skill via the Skill tool (`finalize-artifacts`) after the artifact is written and before the final reply.
- The skill edits the artifact files in place. Do not append a changelog of the cleanup to the artifact; in the final reply, mention what changed in a sentence or two at most unless the user asks for a full report.
- Skip it only for replies that produce no artifact (answering questions, explaining code, running read-only commands).
