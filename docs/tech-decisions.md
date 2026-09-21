# Technical decisions

The technical decisions for the mathematics study site, and the facts verified along the way. Reasons and what was actually tested are recorded so the decisions can be revisited later.

## Requirements

- A static site that delivers content one way. It is published on GitHub Pages.
- A few pages have a graph plotter and a polynomial calculator.
- Proof details and gap-filling supplements can be folded.
- Math, custom macros, and proof figures (derivation trees) can be written.
- As a parsing exercise, the core of the calculator and the plotter is written in Rust and runs as Wasm.
- Maintainability and local developer experience matter most. There is a single developer.

## Decisions

| Area                        | Decision                                                                            | Main reason                                                                                                     |
| --------------------------- | ----------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| Site framework              | Astro 7, Starlight, MDX                                                             | Sidebar, search, table of contents, and dark mode come built in. The output is static HTML that Pages can serve |
| Markdown processing         | Select `unified()` explicitly                                                       | Astro 7's default, Sätteri, cannot run remark/rehype plugins, so `remark-math` and similar would not work       |
| Math                        | MathJax 4, rendered at build time in Node, CHTML output                             | Proof trees can be written with `bussproofs`, and custom macros work. KaTeX has no proof-tree support           |
| Speech                      | Put English speech strings in `aria-label`                                          | They can be embedded in static HTML without runtime JS. Japanese is not supported (see below)                   |
| Folding                     | `<details>` for proofs and long supplements, an inline button for short supplements | Works without JS. The inline supplement reads naturally as a continuation of the sentence                       |
| Calculator and plotter core | A Rust workspace, with one separate Wasm package per feature                        | Learning parsing is the goal. Wasm is built by calling the `wasm-bindgen` CLI directly                          |
| Browser-side components     | React                                                                               | Calculator results are rendered by loading MathJax in the browser                                               |
| Repository layout           | Monorepo, working on `main` only                                                    | Solo development; early API changes fit in one commit                                                           |
| Tool management             | mise pins Node, pnpm, gh, rtk, and lefthook                                         | Versions live in one place (`mise.toml` and `mise.lock`), and CI reads the same config                          |
| Formatting and lint         | oxfmt and oxlint                                                                    | Fast, and can be configured strictly                                                                            |
| Text checks                 | textlint and markdownlint                                                           | Style, punctuation, and AI-sounding writing are checked mechanically                                            |
| Commit-time checks          | lefthook                                                                            | Staged files are checked before the commit                                                                      |
| Browser verification        | Playwright, with tests in `e2e/`                                                    | Browser behavior is kept in a reproducible form                                                                 |
| Publishing                  | GitHub Actions to GitHub Pages                                                      | Actions are pinned by commit SHA                                                                                |

## Verified facts

### Astro 7 and Starlight

- Astro 7's default Markdown processor is Sätteri. remark and rehype plugins run only when `@astrojs/markdown-remark` is installed and `unified()` is selected. Starlight depends on the same API.
- `astro check` uses the TypeScript programmatic API. TypeScript 7 does not provide it, so TypeScript stays on 6.x until Astro supports it. Dependabot is configured not to propose `typescript` 7 or later.
- In agent environments (`CLAUDECODE=1`), `astro preview` starts a background server and the command itself exits. Playwright's `webServer` expects a long-running command, so `e2e/serve.ts` serves the site instead.

### Math

- MathJax 4 rendered both macros and `bussproofs` trees in Node. Write trees with `\AxiomC` and `\BinaryInfC`. The `\Axiom…\fCenter…` form is rendered, but speech generation fails with an error.
- Speech strings exist only for a few locales, such as English. There is no Japanese locale. Japanese text is read one character at a time, and a proof tree's premises are not read out.
- In build-time output, the speech string is stored in the `data-semantic-speech-none` attribute. Turning it into `aria-label` needs post-processing.
- An undefined macro does not fail by default; it is rendered as plain text. Removing the `noundefined` package makes it an error box, and throwing from `formatError` catches syntax errors such as an unclosed brace.
- Generating speech costs about 23 ms per formula (about 4 ms without it).
- HTML size per formula is about 15 KB (2 KB gzipped) for CHTML output and about 23 KB (5.6 KB gzipped) for SVG output. The browser bundle `tex-chtml.js` is about 974 KB (275 KB gzipped).

### GitHub Pages

- Response headers cannot be configured. `cache-control` is `max-age=600` and only gzip compression is used, so JS and fonts loaded by the browser should stay small.
- A published site can be up to 1 GB, with a soft bandwidth limit of 100 GB per month.
- The limit of 10 builds per hour does not apply when publishing with Actions.

### Tool behavior

- Type-aware linting needs `site/.astro/types.d.ts`, which `astro sync` generates. CI has no generated files, so the `sync` task runs first.
- In TypeScript 6, the default of `types` in tsconfig is empty. List `@types/node` and similar packages explicitly.
- In the container, `mise.toml` is read as the global config. Update the lockfile with `mise lock --global`.
- textlint does not read `.textlintrc.jsonc`; write the config in YAML. In MDX, `textlint-disable` comments do not work.
- oxfmt does not format `.astro` files. Its `sortImports` moves a leading `// @ts-check` along with the import, so config files are `.ts`.
- Mafs has had no release since October 2024 and no commit since March 2025. The plotting library has not been chosen yet.

## Math pipeline (implemented)

Flow: `remark-math` finds `$…$` and `$$…$$`, the rehype plugin (`site/src/plugins/rehype-mathjax.ts`) sends each formula to MathJax 4, and the result replaces the formula in the tree. MathJax runs in a native Worker thread (`site/src/math/worker.ts`) that the main thread talks to by messages (`site/src/math/renderer.ts`). The Astro integration (`site/src/integrations/mathjax.ts`) copies the fonts, provides the CSS, and stops the Worker. Macros live in `site/src/math/macros.ts`.

What was verified while building it:

- The plugin module is loaded by the short-lived Vite runner that reads `astro.config.ts`. That runner is closed afterwards, so a lazy `import()` in the plugin fails with "Vite module runner has been closed". A Worker thread outside Vite avoids this, and `terminate()` also stops MathJax's own speech threads. Without a shutdown the process never exits.
- Removing `noundefined` and throwing from `formatError` together makes both an undefined macro ("Undefined control sequence") and an unclosed brace ("Missing close brace") fail. `TexError` does not extend `Error`, so the message is read from its `message` property. The error carries the file line through `file.fail`.
- MathJax does not expect concurrent renders, and Astro processes pages concurrently. Requests are queued and sent one at a time. A test compares concurrent and sequential results.
- The stylesheet grows with the characters used (about 10.5 KB for the base, 16 to 18 KB for typical content). `outputJax.clearCache()` resets it to the base. A per-page inline `<style>` does not work, because the MDX output escapes its text (`&#x22;`) and browsers do not decode entities in `<style>`. So the CSS is one shared file: it is written to `dist/` after all pages are rendered, and the dev server serves the current accumulated CSS. Only pages with math get a `<link>`.
- Starlight's `markdown.css` (in `@layer starlight.content`) gives `margin-top: 1rem` to every element that follows a sibling in the content, except inside `.not-content`. CHTML places `mjx-num`, `mjx-dbox`, `mjx-script`, and similar elements next to each other, so fractions, integral limits, and proof-tree labels came out with 16px gaps. Adding `not-content` to each `mjx-container` fixes it, and `e2e/math.spec.ts` checks the computed margins inside math.
- The number-set macros follow the LaTeX `numbersets` package (github.com/enunun/numbersets, v0.2.0): `\NaturalNumbers`, `\Integers`, `\RationalNumbers`, `\RealNumbers`, `\ComplexNumbers`, `\NumberSet[style]{X}`, and the styles `bb`, `bfup`, `bfit`. MathJax 4 has no `\csname`. Macro expansion substitutes arguments as text but inserts a space when a control sequence is followed by a letter (`ParseUtil.addArgs`), so `\numbersetstyle#1` cannot build a name. Inside `\begin{…#1}` it can, and the environments come from the `configmacros` `environments` option. The begin code is prepended to the remaining input, so it must be `\mathbb` (which then takes the braced content) and not `\mathbb{`. An unknown style fails the build with "Unknown environment".
- The dev server may pass middleware URLs with the base stripped, so the CSS middleware accepts both forms.
- `chtml.displayOverflow: 'scroll'` makes a long display formula scroll inside its own container, without a page-wide horizontal scroll (checked at a 390 px viewport).
- Stripping the `data-semantic*`, `data-speech*`, `data-braille*`, and `data-latex` attributes, which only the browser-side explorer uses, cuts each formula's HTML by roughly a third.
- The fonts are 105 woff2 files (1.8 MB). The base CSS references 35 of them, and the browser loads only the ranges it needs. They are copied to `site/public/mathjax-fonts/` at config setup and are git-ignored.

Known limits:

- Speech strings are English only. `\norm` is read as "metric", and the `\Axiom…\fCenter…` form gets no speech.
- Generating speech costs about 23 ms per formula, and there is no cache yet.

## Open items

- The plotting library (a custom SVG, Mafs, or JSXGraph).
- The range of expressions the polynomial calculator handles.
- Automated accessibility checks (axe).
- Running E2E tests in CI.
- Equation numbers (`\tag`, `\label`) for math.
- A cache for speech generation, if build time becomes a problem.
- Rendering math in the browser for the calculator, with the same macros.

## Theorem numbering (implemented)

Flow: `site/src/plugins/rehype-statements.ts` runs on each MDX page before the MathJax plugin. It reads the JSX nodes that MDX passes through to the rehype tree, numbers the statement elements, and rewrites `<Ref>` and `<Proof of>`.

- A label is built from the kind, the page identifier, and the sequence number within the page (`定理abs-3`). Definitions, lemmas, propositions, theorems, and corollaries share one counter. There is no space between the Japanese kind and the Latin identifier, following the writing rules for Japanese text.
- A page identifier uses letters, digits, hyphens, and underscores. It comes from the frontmatter `pageId` (also validated by the content schema in `content.config.ts`) or the file name, and is unique across the site. Labels are therefore unique without a chapter structure.
- References are written by identifier. An identifier is optional on a statement; without one, the anchor is the label and the statement cannot be referenced. With one, the anchor stays stable when statements are inserted or reordered.
- A reference to another page needs that page's numbering while a different page is being rendered. Astro renders pages concurrently and a plugin sees one page at a time, so `plugins/statements/catalog.ts` scans `content/docs/**/*.mdx` with remark-parse, remark-mdx, and remark-frontmatter, and caches the result by file modification time, so the dev server picks up edits. Pages that fail to parse are skipped there; their own build reports the error. Pages without statements are not registered, so their identifiers need not be unique.
- The page URL is derived from the file path. Starlight lowercases and slugifies file names, so a cross-page reference requires file names made of lowercase letters, digits, hyphens, and underscores, and fails the build otherwise.
- Numbering follows document order in the tree. The scanner and the plugin share `findStatementNodes` and `numberStatements`, so both agree. Code blocks are not counted, because they are not JSX nodes.
- `Ref` is replaced by a plain `<a>` in the tree, so it needs no component and no import. The statement components receive `label` and `anchor` as props from the plugin.
- oxfmt formats MDX badly when a paragraph contains inline JSX: it splits the paragraph around `<Ref />` and breaks inline math at 100 columns. MDX is excluded from oxfmt (`.oxfmtrc.json`) and formatted by hand.
