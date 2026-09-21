---
name: verify-site
description: How to verify a change to the site (build, lint, type check, E2E tests, screenshots for visual checks, and the items that cannot be checked automatically). Use it after changing pages, components, styles, or configuration, and before reporting the work as done.
---

# verify-site

Confirm that a change builds, works under the same base path as production, and behaves correctly in a real browser.

## Procedure

1. Run `mise run check`. It runs formatting, oxlint, remark-lint (MDX), markdownlint, textlint, the type check (`astro check`), the Vitest unit tests, and the build. CI runs the same thing.
2. For changes that affect rendering or behavior, run `mise run e2e`. It builds `site/dist` and runs the tests in `e2e/`.
3. For changes that affect appearance, take screenshots and look at the images (next section).
4. After committing and pushing, confirm that CI succeeded (the section after next).

## Checking appearance

Take screenshots with `mise run screenshot`, then open the PNG with the Read tool.

```sh
mise run screenshot -- dev/notation/ /tmp/notation.png --full
mise run screenshot -- dev/notation/ /tmp/dark.png --theme dark
mise run screenshot -- dev/notation/ /tmp/mobile.png --width 390 --full
mise run screenshot -- dev/notation/ /tmp/detail.png --clip ".detail" --click ".detail-toggle"
mise run screenshot -- dev/notation/ /tmp/math.png --scroll "h3#導出木"
```

- Write the page path relative to the production base path (`/math-study/`).
- Check the light and dark themes, and both a wide window and a narrow one (about 390px).
- `--click` performs interactions before the shot, in order. `--clip` captures only the given element. `--scroll` scrolls to an element first, which is the way to look at a section of a long page. `--full` captures the whole page (large; the image is downscaled when read).
- Running `mise run screenshot` without arguments prints the usage.

## Automated browser checks

`mise run e2e` runs three kinds of tests, and CI runs them in the `build` job before `deploy`, so a failure blocks publishing.

- Behavior specs (`fold`, `detail`, `math`, `statements`): what each feature does.
- `accessibility.spec.ts`: axe on every page, in the light and dark themes, at 1280px and 390px, with all folds opened. Any violation fails.
- `site.spec.ts`: every page opens without console errors, page errors, failed requests, or 4xx/5xx responses, and every internal link (including `#hash` targets) resolves.

Pages are enumerated from `site/dist` (`e2e/routes.ts`), so a new page is covered without editing the tests. Only Chromium is used.

When axe reports a violation, fix the cause. If it is in third-party markup you cannot change, exclude that one element or rule in `accessibility.spec.ts` (`AxeBuilder#exclude`, `#disableRules`) with a comment that says why. Do not loosen the check for every page.

Scrollable regions must be keyboard-focusable (axe rule `scrollable-region-focusable`). This site handles it in three places: display math gets `tabindex="0"` in `rehype-mathjax.ts`, tables get it in `rehype-focusable-tables.ts`, and code blocks wrap (`expressiveCode.defaultProps.wrap`) so they do not scroll. A new component that scrolls horizontally needs the same treatment.

## Checking CI

A push triggers GitHub Actions, which runs `build` (`mise run check`, then `mise run e2e`) and `deploy`. Check the result through the public API.

```sh
git rev-parse HEAD
curl -s "https://api.github.com/repos/enunun/math-study/actions/workflows/deploy.yml/runs?branch=main&per_page=3"
```

Find the run whose `head_sha` matches the local commit and read its `status` and `conclusion`. If it failed, reproduce it locally: CI has no generated files such as `site/.astro`, so something can pass locally and fail in CI. Delete them (`rm -rf site/.astro site/dist`) and rerun `mise run check`.

## Adding E2E tests

- Put tests in `e2e/*.spec.ts`. Helpers that are not specs (such as `routes.ts`) must not end in `.spec.ts`. Open pages by a path relative to the base path (`page.goto('dev/notation/')`).
- Find elements by what the user sees (`getByRole`, and so on). Compare visible text with `toHaveText(…, { useInnerText: true })`, which excludes text in hidden elements.
- Check the JavaScript-disabled rendering with `test.use({ javaScriptEnabled: false })`.
- For print behavior, dispatch the `beforeprint` and `afterprint` events, and check print styles with `page.emulateMedia({ media: 'print' })`.
- Recreate external state (URL hashes, search highlights) with `evaluate`.
- The strict lint rules apply to tests too. The rules turned off only for `e2e/` are in `.oxlintrc.jsonc`.

## What cannot be checked automatically

Check these by hand, in a local browser or with assistive technology.

- Japanese screen reader output (NVDA, VoiceOver).
- Whether browser find-in-page (Ctrl+F) opens a closed fold. It differs between browsers.
- Real printing and print preview.
- Real highlighting when opening a page from the site search (Pagefind).
- Visual regressions such as a shifted fraction or a proof-tree label. Screenshot comparison is deliberately not automated (fonts and rendering differ between environments, and the images would live in the repository). Look at `mise run screenshot` output after changing math, styles, or components, and add a layout assertion to a spec when a specific bug is worth guarding, as `math.spec.ts` does for the Starlight margin.
- Firefox and Safari. Only Chromium is installed in the container and CI.
- Speech quality of formulas. The `aria-label` strings are generated in English; listen to them with a screen reader if it matters.

## Troubleshooting

- Browser missing or wrong version: run `mise run browsers`.
- `astro preview` exits immediately: in agent environments it runs as a background server by design. Check it with `astro preview status` and stop it with `astro preview stop`. E2E tests use their own server (`e2e/serve.ts`).
- Port 4322 is in use: stop the existing process. Playwright reuses an existing server.
- E2E shows stale content: `mise run e2e` builds first. Running `pnpm exec playwright test` directly may serve an old `site/dist`.
- Type-aware lint reports many `no-unsafe-*` errors: `site/.astro/types.d.ts` is missing. Run `mise run sync`.
- Math looks wrong or the build hangs: run `mise run test` first (the plugin's unit tests cover macros, proof trees, speech labels, error positions, and concurrent pages). Then look at `mise run screenshot -- dev/notation/ /tmp/math.png --scroll "h3#導出木"`. A build that never exits means the MathJax Worker was not terminated.
