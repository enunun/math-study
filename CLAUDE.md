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
- Read the `write-content` skill before writing site content or documentation.
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
- Do not inline the MathJax CSS with a `<style>` element: the MDX output HTML-escapes its text (`&#x22;`), and browsers do not decode entities in `<style>`. The CSS is one shared file, `mathjax.css`. It is written to `dist/` at `astro:build:done` and served by a dev-server middleware; pages with math get a `<link>` to it. The fonts are copied to `site/public/mathjax-fonts/` (git-ignored) at config setup.
- Undefined macros and TeX syntax errors fail the build, with the file line and the TeX source in the message. MathJax's `TexError` does not extend `Error`, so read its `message` property instead of using `instanceof Error`.
- Starlight adds `margin-top: 1rem` to every element that follows a sibling inside `.sl-markdown-content` (`markdown.css`), unless the element is inside a `.not-content` ancestor. MathJax's CHTML output is made of adjacent custom elements (`mjx-num` and `mjx-dbox` in a fraction, `mjx-mo` and `mjx-script` in a superscript), so without the exclusion, fractions, limits, and proof-tree labels are pulled apart. `rehype-mathjax.ts` adds the `not-content` class to each `mjx-container`, and `e2e/math.spec.ts` fails if any element inside one gets a 16px top margin. Do the same for any other component that renders adjacent custom elements.
- Add math macros in `site/src/math/macros.ts` and update the list on the test page. The number-set macros follow the LaTeX `numbersets` package (`\NaturalNumbers`, `\Integers`, `\RationalNumbers`, `\RealNumbers`, `\ComplexNumbers`, and `\NumberSet[style]{X}` with the styles `bb`, `bfup`, `bfit`). MathJax has no `\csname`, and macro expansion inserts a space between a control sequence and a following letter, so the style is selected through an environment name (`\begin{numbersetstyle#1}`), which is why `macros.ts` also exports `environments`.
- Definitions and theorems (`<Definition>`, `<Lemma>`, `<Proposition>`, `<Theorem>`, `<Corollary>` from `@/components/statement`) are numbered at build time by `site/src/plugins/rehype-statements.ts`. The label is kind + page identifier + sequence number (`定理abs-3`), counted per page across kinds. The page identifier is the frontmatter `pageId` or the file name (`index.mdx` uses the directory name). It must be unique across the site and use only letters, digits, hyphens, and underscores, starting with a letter or digit. `<Ref to="id" page="pageId" />` becomes a link and `<Proof of="id">` sets the proof title; `Ref` needs no import because the plugin replaces it. Other pages are found by scanning `site/src/content/docs/**/*.mdx` (`plugins/statements/catalog.ts`), so pages referenced from other pages need lowercase ASCII file names. Bad identifiers, duplicates, and dangling references fail the build. To add a statement kind, extend `STATEMENT_KINDS` in `statements/collect.ts`, and add a wrapper component.
- oxfmt ignores `site/src/content/**/*.mdx`: it re-wraps paragraphs that contain inline JSX such as `<Ref />`, which splits inline math and inserts line breaks between Japanese words. Format MDX by hand, and keep each paragraph on one line.
- MDX is linted by remark-lint (`.remarkrc.mjs`, `mise run lint:mdx`), not markdownlint. remark parses MDX like the build does, so unclosed tags and broken `{…}` expressions fail at commit time, and components need no registration. Keep its remark plugins (`remark-mdx`, `remark-math`) in step with `astro.config.ts`: without `remark-math`, `{…}` inside `$$…$$` is read as a JSX expression. remark-cli prints a re-serialized copy of a single input file to stdout and rewrites some text (`[x]` becomes `\[x]`), so always pass `--no-stdout` and never use `--output`. markdownlint still checks `README.md`, `docs/`, and any plain `.md` in the content.
- Math macros are named by meaning in the numbersets style (`\RealNumbers`). Do not add short aliases such as `\R`.
- Speech strings are English only. Known quirks: `\norm` is read as "metric", and the `\Axiom…\fCenter…` form gets no speech.
- `mise run test` runs the Vitest unit tests (`site/src/**/*.test.ts`). `mise run check` includes them.

## Skills

- `verify-site`: how to verify a change (build, lint, E2E, screenshots).
- `write-content`: how to write content and use the notation (writing rules, folding components, handling lint findings).
- `finalize-artifacts`: how to finish a deliverable.

# Artifact Cleanup

## Golden Rule

**Whenever you produce an artifact, always run the `finalize-artifacts` skill (`.claude/skills/finalize-artifacts/`) to clean it up before reporting the work as done.**

An artifact is any deliverable you create or substantially rewrite: documents, READMEs, code and code comments, config files, scripts, commit messages, PR descriptions, and so on.

- Invoke the skill via the Skill tool (`finalize-artifacts`) after the artifact is written and before the final reply.
- The skill edits the artifact files in place. Do not append a changelog of the cleanup to the artifact; in the final reply, mention what changed in a sentence or two at most unless the user asks for a full report.
- Skip it only for replies that produce no artifact (answering questions, explaining code, running read-only commands).
