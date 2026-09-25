<!-- rtk-instructions v2 -->
# RTK (Rust Token Killer)

Prefix every shell command with `rtk`, including each command in a `&&` chain — it is always safe (a dedicated filter cuts noisy output for tests, builds, git, and more; anything without one passes through unchanged). The full command reference is in the global `~/.claude/RTK.md` (already loaded). Meta commands: `rtk gain` (savings so far), `rtk discover` (missed opportunities in past sessions), `rtk proxy <cmd>` (run unfiltered, for debugging).
<!-- /rtk-instructions -->

# math-study

A study site for mathematics (Astro 7, Starlight, MDX). `README.md` (Japanese) gives the overview. `docs/tech-decisions.md` records the technical decisions and the facts verified along the way.

## Working conventions

- This is a solo project: work on `main` only. Do not use branches or pull requests. Commit finished work and push it to `main`, and split unrelated changes into separate commits.
- Pushing publishes the site through GitHub Pages. Local `mise run check` and `mise run e2e` already give enough confidence before pushing, so a task is done once it is pushed: do not wait for or poll GitHub Actions as part of finishing a task. Check CI only when the user asks.
- After a change, run `mise run check`. For changes that affect rendering or behavior, also run `mise run e2e` and check the appearance with `mise run screenshot`. The procedure is in the `verify-site` skill.
- Record what you learn in the same change, so the next session and the human maintainer can find it. Put durable knowledge where it belongs: `CLAUDE.md` for conventions and pitfalls, `.claude/skills/**` for procedures, `docs/tech-decisions.md` for decisions and verified facts, and `README.md` plus `site/src/content/docs/dev/internals.mdx` (Japanese, for humans) for how the system works. Do not leave findings only in the conversation or the scratchpad. If a change alters who does what, what is custom-built, or how a feature is used, update `internals.mdx` and the "Code map" below in the same commit. Skip this only when the information is already recorded.
- Read the `write-content` skill before writing site content or documentation.
- The figure feature (`crates/figure/`, design in `docs/graph-design.md`) is developed test-first: write the failing test, see it fail, then write the code that makes it pass. See the `figure-engine` skill.
- Run `system-development-skills:finalize-artifacts` before reporting a deliverable as done (see "Artifact Cleanup" below).
- Prefix commands with `rtk`. The hook is in `.claude/settings.json`; do not install it in the user-level settings.
- `enabledPlugins` in `.claude/settings.json` only enables a plugin that is already installed. If `system-development-skills:finalize-artifacts` is an `Unknown skill` (the plugin is missing from `~/.claude/plugins/installed_plugins.json`, as in a fresh container), ask the user to install it from `/plugin`; it loads in the next session.
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

- In agent environments (`CLAUDECODE=1`), `astro preview` starts a background server and the command itself exits (see the `verify-site` skill for how to check and stop it).
- In the container, `mise.toml` is read as the global config, so `mise lock` needs `--global`.
- Keep `typescript` on the 6.x line: `astro check` does not support TypeScript 7.
- pnpm 12 allows dependency build scripts only when listed. Add them to `allowBuilds` in `pnpm-workspace.yaml`.
- oxfmt does not format `.astro` files. Put logic in `.ts` files and keep `.astro` files thin.
- `mise run test` runs the Vitest unit tests (`site/src/**/*.test.ts`). `mise run check` includes them.
- Rust/Wasm build environment (toolchain pinning, `gcc` for native `cargo test`, TeX Live for TikZ verification): see the `rust-wasm` skill (the polynomial calculator) or the `figure-engine` skill (the figure engine); both share the same `mise.toml`-pinned toolchain.

## Code map

Build-time pipeline: MDX → remark (`remark-math`) → rehype (`rehypeStatements`, then `rehypeMathjax`) → Astro components with Starlight → static HTML. `site/astro.config.ts` wires it together. The human-facing explanation is `site/src/content/docs/dev/internals.mdx`. Math rendering internals (the Worker, the shared CSS file, autospacing) are in the `math-pipeline` skill; the figure engine and its editor are in the `figure-engine` skill; both are large enough that they are not repeated here.

- `site/src/math/macros/`, `site/src/integrations/mathjax.ts`, `site/src/plugins/rehype-mathjax.ts`: see the `math-pipeline` skill.
- `site/src/plugins/rehype-statements.ts`: the numbering and reference step (statements, equations, figures). It uses `plugins/statements/`: `tree.ts` (node types, `DocumentError`), `collect.ts` (statement kinds, page identifiers, numbering), `equations.ts` (`\label` → `\tag`), `math-sites.ts` (finds formulas in hast and mdast), `scan.ts` (reads a whole MDX file for the catalog), `catalog.ts` (index of other pages, page URLs). `inspect.ts`, `test-support.ts`, and `test-processor.ts` are unit-test helpers. See the `write-content` skill for the authoring side.
- `site/src/components/fold/` (`Proof`, `Remark`, `Detail`, `auto-open.ts`) and `site/src/components/statement/` (`Definition`, `Lemma`, `Proposition`, `Theorem`, `Corollary`, `Example`, `Problem`, `Answer`): the components. `Ref` has no component; the plugin replaces it.
- `site/src/content/docs/dev/continuity.mdx`: the reference article that uses every notation. The `dev/` pages are hidden from the sidebar and search but must stay reachable: link every new `dev/` page from the home page (`site/src/content/docs/index.mdx`). Public pages are grouped by category, one directory each (`topics/` 単発ネタ, `tools/` ツール): a new category needs a group in the `sidebar` of `site/astro.config.ts` and a heading with cards on the home page, and a new page in an existing category needs only a home card. `tools/` holds anything that helps write or draw mathematics, not only browser tools (`numbersets.mdx` introduces a LaTeX package). The home page uses the default `doc` template, not `splash`, because `splash` has no sidebar. The pages are not a sequence, so `pagination: false` hides Starlight's 前へ/次へ links. Since Starlight 0.39 a group is `{ label, items: [{ autogenerate: { directory } }] }`, not `{ label, autogenerate }`. `site/src/content.config.ts`: the `pageId` frontmatter schema. `site/src/content/docs/dev/`: the hidden pages (`notation.mdx` notation reference, `abs.mdx` cross-reference target, `internals.mdx` explanation).
- `crates/polynomial/`, `crates/polynomial-wasm/`, `site/src/calculator/`, `site/src/components/calculator/` (the polynomial calculator, `dev/calculator.mdx`): see the `rust-wasm` skill.
- `crates/figure/`, `crates/figure-wasm/`, `site/src/figure/`, `site/src/figure-editor/`, `site/src/components/figure-editor/`, `site/src/components/figure/`, `site/src/projection/`, `site/src/components/projection/`, `site/src/figures/`, `site/public/schema/scene.schema.json` (the figure engine and its editor, `dev/figure-first.mdx`, `dev/figure-space.mdx`, `dev/figure-scene.mdx`, `topics/figure-algorithms.mdx`, `topics/figure-fill.mdx`, `tools/figure-editor.mdx`): see the `figure-engine` skill. The scene format and the extension list are in `docs/graph-design.md`.
- `site/src/plugins/pipeline.ts`: the rehype plugins in execution order, with the ordering constraints (numbering before math rendering, autospace after it). Add a new rehype plugin there, not in `astro.config.ts`.
- `site/src/plugins/rehype-autospace.ts` and `site/src/styles/typesetting.css`: see the `math-pipeline` skill.
- `site/src/components/share/`: the SNS share links under every page. `Footer.astro` overrides Starlight's `Footer` (`components` in `site/astro.config.ts`) and puts `ShareButtons.astro` before the default footer. `share-links.ts` builds plain share URLs and loads no third-party scripts. To add a service, add it there.
- `site/src/plugins/rehype-focusable-tables.ts`: gives tables `tabindex="0"` so a horizontally scrolling table is keyboard-accessible.
- `e2e/`: Playwright specs and helpers (`routes.ts` lists every page in `site/dist` for the all-pages checks). `.remarkrc.mjs`, `.textlintrc.yml`, `.oxlintrc.jsonc`, `lefthook.yml`, `mise.toml`: lint and task configuration.

## Skills

- `verify-site`: how to verify a change (build, lint, E2E, screenshots, CI).
- `write-content`: how to write content and use the notation (writing rules, folding components, statements/equations/figures numbering, handling lint findings).
- `math-pipeline`: how math rendering itself works (the Worker, the shared CSS file, Starlight interaction, autospacing) — the internals behind what `write-content` documents for authors.
- `rust-wasm`: how the Rust/Wasm polynomial calculator works, and how to add or change it.
- `figure-engine`: how the Rust figure engine and its browser editor work, and how to add or change an object type.
- `system-development-skills:finalize-artifacts`: how to finish a deliverable. Provided by the `enunun/system-development-skills` plugin (see `extraKnownMarketplaces`/`enabledPlugins` in `.claude/settings.json`), not a local skill in this repository.

# Artifact Cleanup

## Golden Rule

**Whenever you produce an artifact, always run the `system-development-skills:finalize-artifacts` skill to clean it up before reporting the work as done.**

An artifact is any deliverable you create or substantially rewrite: documents, READMEs, code and code comments, config files, scripts, commit messages, PR descriptions, and so on.

- Invoke the skill via the Skill tool (`system-development-skills:finalize-artifacts`) after the artifact is written and before the final reply.
- The skill edits the artifact files in place. Do not append a changelog of the cleanup to the artifact; in the final reply, mention what changed in a sentence or two at most unless the user asks for a full report.
- Skip it only for replies that produce no artifact (answering questions, explaining code, running read-only commands).
