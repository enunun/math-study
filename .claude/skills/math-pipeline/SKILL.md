---
name: math-pipeline
description: How math (MathJax 4) is rendered, at build time and in the browser, and the pitfalls specific to that pipeline (the Worker, the shared CSS file and its cache-busting, the Starlight/not-content interaction, autospacing between Japanese and Latin text, slashed inline fractions). Use before touching site/src/plugins/rehype-mathjax.ts, site/src/plugins/rehype-autospace.ts, site/src/typesetting/, site/src/styles/typesetting.css, site/src/math/, or site/src/integrations/mathjax.ts.
---

# math-pipeline

Build-time math: MDX → remark (`remark-math`) → rehype (`rehypeStatements`, then `rehypeMathjax`) → static HTML. `site/astro.config.ts` wires it together; `site/src/plugins/pipeline.ts` lists the rehype plugins in execution order (numbering before math rendering, autospace after it) — add a new rehype plugin there, not in `astro.config.ts`. For writing rules and notation, see the `write-content` skill; this skill is about the pipeline's own mechanics.

## The Worker

MathJax itself runs in a native Worker thread (`site/src/math/worker.ts`), not through Vite: `rehype-mathjax.ts` is loaded by the short-lived Vite runner that reads `astro.config.ts`, and a lazy `import()` after that runner closes fails with "Vite module runner has been closed". Terminating the Worker (`astro:build:done`, `astro:server:done`, handled in `site/src/integrations/mathjax.ts`) is also what lets the process exit; without it, MathJax's speech threads keep it alive. `renderer.ts` is the main-thread client (the Worker, its request queue, and moving the speech string to `aria-label`).

An undefined macro or a TeX syntax error fails the build, with the file line and the TeX source in the message. MathJax's `TexError` does not extend `Error`, so read its `message` property instead of using `instanceof Error`.

## The shared CSS file

Do not inline the MathJax CSS with a `<style>` element: the MDX output HTML-escapes its text (`&#x22;`), and browsers do not decode entities in `<style>`. The CSS is one shared file, `mathjax.css`, written to `dist/` at `astro:build:done` and served by a dev-server middleware (`site/src/integrations/mathjax.ts`); pages with math get a `<link>` to it. Fonts are copied to `site/public/mathjax-fonts/` (git-ignored) at config setup. The CSS holds one size rule per glyph that some page used (`mjx-c.mjx-c1D465 { … }`), and `mjx-c` is clipped to its padding box (`clip-path`). A browser that keeps an older `mathjax.css` next to newer HTML (GitHub Pages sends `max-age=600`) therefore draws the new glyphs as clipped slivers. The link carries `?v=<time of the config load>` (`stylesheetUrl` in `astro.config.ts`) and the dev middleware sends `no-store`; keep both, and keep the query out of any URL comparison (`e2e/math.spec.ts` checks it).

The browser MathJax (the calculator, the figure editor's projection island) builds its font URL from `import.meta.env.BASE_URL`, which has no trailing slash (`/math-study`), so join with `/` (`fontUrl()` in `site/src/calculator/mathjax.ts`). It must also switch enrichment, speech, and braille off through `menuOptions.settings` — the menu settings override the individual `enable*` options, so leaving this out starts a speech worker that fails; see the `rust-wasm` skill for the calculator's version of this and `e2e/calculator.spec.ts`'s wait for `document.fonts.ready` before checking for failed font requests.

## Starlight interaction

Starlight adds `margin-top: 1rem` to every element that follows a sibling inside `.sl-markdown-content` (`markdown.css`), unless the element is inside a `.not-content` ancestor. MathJax's CHTML output is made of adjacent custom elements (`mjx-num`/`mjx-dbox` in a fraction, `mjx-mo`/`mjx-script` in a superscript), so without the exclusion, fractions, limits, and proof-tree labels are pulled apart. `rehype-mathjax.ts` adds the `not-content` class to each `mjx-container`, and `e2e/math.spec.ts` fails if any element inside one gets a 16px top margin. Do the same for any other component that renders adjacent custom elements (a figure's `<Figure>` output also gets `not-content`, for the same reason).

## Autospacing

Source text has no spaces between Japanese and Latin letters, digits, inline code, or math (textlint enforces it; see the `write-content` skill); typesetting adds a 1/4em gap, the pLaTeX default for `\xkanjiskip`, without changing the DOM text. CSS `text-autospace` cannot do this: its width is fixed at 1/8em and it ignores inline math (an atomic box). So `.sl-markdown-content` is `no-autospace`, and `rehype-autospace.ts`, which runs after `rehypeMathjax`, marks the non-Japanese side of each boundary with `autospace-before`/`autospace-after`: a Latin run inside a text node is wrapped in a `<span>`, and inline `mjx-container` and inline `code` get the class on the element itself. Which characters count as Japanese or Latin, and the segmentation, are in `site/src/typesetting/autospace.ts` (shared with `components/typesetting/Autospaced.astro`, which spaces strings that components print from props, such as statement labels). It walks into MDX component nodes, looks through inline wrappers (links, emphasis), and skips `pre`, `svg`, and the inside of math.

`typesetting.css` gives the classes a margin of `var(--autospace)`, a registered `<length>` (`@property`) set to `0.25em` on every element except `code` and `mjx-container`, so those inherit the surrounding text's 1/4em instead of using their own smaller font size. Outside the body (sidebar, page title, TOC), `text-autospace: normal` still applies (1/8em). `e2e/typesetting.spec.ts` measures the gaps.

## Inline fractions

`rehype-mathjax.ts` passes the TeX of inline formulas through `slashFractions` (`site/src/math/inline-fraction.ts`, tokenizer in `tex-tokens.ts`) before rendering, turning `\frac{a}{b}` into `a/b` with parentheses only where needed. Display math and figure labels (math inside `span.figure-label`, kept stacked to match the TikZ export and the editor) are not rewritten. The rules and the rejected alternative (redefining `\frac`) are in `docs/tech-decisions.md`. A malformed `\frac` is passed through unchanged so MathJax still reports the error; keep it that way, because the error message shows the source TeX, not the rewritten one.

## Macros

Add a module under `site/src/math/macros/modules/`, one file per area, default-exporting a `MacroModule` (add to the file of the same area, or create a new one) — see the `write-content` skill for the naming convention and the existing macro list.
