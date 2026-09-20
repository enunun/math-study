---
name: verify-site
description: How to verify a change to the site (build, lint, type check, E2E tests, screenshots for visual checks, and the items that cannot be checked automatically). Use it after changing pages, components, styles, or configuration, and before reporting the work as done.
---

# verify-site

Confirm that a change builds, works under the same base path as production, and behaves correctly in a real browser.

## Procedure

1. Run `mise run check`. It runs formatting, oxlint, markdownlint, textlint, the type check (`astro check`), and the build. CI runs the same thing.
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
```

- Write the page path relative to the production base path (`/math-study/`).
- Check the light and dark themes, and both a wide window and a narrow one (about 390px).
- `--click` performs interactions before the shot, in order. `--clip` captures only the given element. `--full` captures the whole page.
- Running `mise run screenshot` without arguments prints the usage.

## Checking CI

A push triggers GitHub Actions, which runs `build` and `deploy`. Check the result through the public API.

```sh
git rev-parse HEAD
curl -s "https://api.github.com/repos/enunun/math-study/actions/workflows/deploy.yml/runs?branch=main&per_page=3"
```

Find the run whose `head_sha` matches the local commit and read its `status` and `conclusion`. If it failed, reproduce it locally: CI has no generated files such as `site/.astro`, so something can pass locally and fail in CI. Delete them (`rm -rf site/.astro site/dist`) and rerun `mise run check`.

## Adding E2E tests

- Put tests in `e2e/*.spec.ts`. Open pages by a path relative to the base path (`page.goto('dev/notation/')`).
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

## Troubleshooting

- Browser missing or wrong version: run `mise run browsers`.
- `astro preview` exits immediately: in agent environments it runs as a background server by design. Check it with `astro preview status` and stop it with `astro preview stop`. E2E tests use their own server (`e2e/serve.ts`).
- Port 4322 is in use: stop the existing process. Playwright reuses an existing server.
- E2E shows stale content: `mise run e2e` builds first. Running `pnpm exec playwright test` directly may serve an old `site/dist`.
- Type-aware lint reports many `no-unsafe-*` errors: `site/.astro/types.d.ts` is missing. Run `mise run sync`.
