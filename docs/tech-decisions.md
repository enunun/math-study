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

## Open items

- The plotting library (a custom SVG, Mafs, or JSXGraph).
- The range of expressions the polynomial calculator handles.
- Automated accessibility checks (axe).
- Running E2E tests in CI.
- Automatic theorem numbering (below).

## Theorem numbering design

- A label is built from the kind, the page identifier, and the sequence number within the page. Definitions, lemmas, and theorems share one counter.
- A page identifier uses only ASCII letters, digits, and a few symbols such as hyphens. It comes from a frontmatter field or the URL slug.
- References are written by identifier and resolved to a number and a link at build time. Numbering is done by a build-time plugin.
- A page identifier is unique across the site, so labels are unique without deciding a chapter structure.
