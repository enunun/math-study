---
name: math-pipeline
description: How math (MathJax 4) is rendered, at build time and in the browser, and the pitfalls specific to that pipeline (the Worker, the shared CSS file and its cache-busting, the Starlight/not-content interaction, autospacing between Japanese and Latin text). Use before touching site/src/plugins/rehype-mathjax.ts, site/src/plugins/rehype-autospace.ts, site/src/math/, or site/src/integrations/mathjax.ts.
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

Source text has no spaces between Japanese and Latin letters, digits, inline code, or math (textlint enforces it; see the `write-content` skill); typesetting adds a 1/8em gap, like TeX's `\xkanjiskip`, without changing the DOM text. `site/src/styles/typesetting.css` sets `text-autospace: normal` for text and inline code (no gap next to punctuation). Inline math is an atomic box that `text-autospace` ignores (measured: gap 0), so `rehype-autospace.ts`, which runs after `rehypeMathjax`, adds `autospace-before`/`autospace-after` to an inline `mjx-container` when its neighbor is Han, Hiragana, Katakana, or `ー`, and the CSS gives those classes a 0.125em margin. Code blocks and the inside of math are `no-autospace`. `e2e/typesetting.spec.ts` measures the gaps.

## Macros

Add a module under `site/src/math/macros/modules/`, one file per area, default-exporting a `MacroModule` (add to the file of the same area, or create a new one) — see the `write-content` skill for the naming convention and the existing macro list.
