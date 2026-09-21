---
name: write-content
description: Rules and notation for writing the site content (Markdown, MDX) and README. Covers Japanese writing rules, the folding components, and how to run and handle the text lint. Use it before writing or editing content.
---

# write-content

Rules and notation for the site content (`site/src/content/**`) and `README.md`, which are written in Japanese. textlint checks the prose, remark-lint checks MDX, and markdownlint checks `README.md` and `docs/`. Everything else meant for Claude (`CLAUDE.md`, skills, `docs/**`) is written in English.

## Writing rules for Japanese text

- Use the plain form (である調). This covers body text and list items; headings are unconstrained.
- Use the full-width comma "，" (U+FF0C) and the full-width period "．" (U+FF0E). Never use "、" or "。".
- Put no space between Japanese and Latin text, digits, or inline code. Write "MDXの記法" and "`mise run dev`を実行". Do not write "MDX の記法".
- Do not type spaces between Japanese and Latin text, digits, inline code, or math. The page adds a 1/8em gap when it is typeset (`text-autospace` and `rehype-autospace.ts`).
- Use a full-width colon "：" after Japanese text, with no space after it ("公開先：`/x`"). Colons after Latin text stay half-width.
- Avoid AI-sounding writing: bold-prefix bullet lists ("**Item**：description"), hype, and emphasis in ordinary sentences.
- Keep each sentence within 100 characters and within three commas. Do not repeat the same particle within a sentence.
- A lone Latin letter outside math (a variable such as x written as plain text) is flagged as an "unnatural alphabet". Write variables inside `$…$`.

Code, code blocks, and inline code are not checked.

## Folding components

Write this import at the top of the MDX file, right after the frontmatter.

```mdx
import { Detail, Proof, Remark } from '@/components/fold';
```

- `<Proof title="定理1.2">…</Proof>`: hides the details of a proof. It has a frame and the label 証明. Put blank lines before and after the tags so that the content is parsed as Markdown.
- `<Remark title="…">…</Remark>`: a long supplement that needs several paragraphs or lists. It has the label 補足. The blank-line rule is the same as for `Proof`.
- `<Detail>…</Detail>`: a short supplement placed after a sentence. While closed, the sentence is followed by "［…］". Pressing it reveals the supplement as a continuation. The author writes the period of the preceding sentence; the component never adds punctuation. Put only content that fits on one line inside.

```mdx
実数の二乗は0以上である．<Detail>実際，正の数の二乗は正であり，負の数の二乗も正である．0の二乗は0である．</Detail>
```

`Proof` and `Remark` accept `open`, which shows them expanded from the start. They can be nested. Full examples and the verification checklist are on the test page (`site/src/content/docs/dev/notation.mdx`). Update that page whenever the notation changes.

## Math

Math is rendered at build time by MathJax 4. Write inline math as `$…$` and display math as `$$…$$`. The details and the rendered examples are on the test page (`site/src/content/docs/dev/notation.mdx`).

- Write a vector as a bold italic letter (`\boldsymbol{d}`, not the upright `\mathbf{d}`). Write its components as a square-bracket matrix, `\begin{bmatrix} … \end{bmatrix}`, a column vector by default. A row vector separates its entries with `&` and has no commas. Points and coordinates follow the same rule; keep round brackets for intervals, function arguments, and pairs of parameters.
- Custom macros are defined as modules in `site/src/math/macros/modules/`, one file per area. After changing them, update the list on the test page.
  - Number sets use the names of the LaTeX `numbersets` package: `\NaturalNumbers`, `\Integers`, `\RationalNumbers`, `\RealNumbers`, `\ComplexNumbers`. They take an optional style, `\RealNumbers[bfup]`, with `bb` (blackboard bold, the default), `bfup` (upright bold), and `bfit` (italic bold). `\NumberSet[style]{X}` sets any letter the same way.
  - The others are `\abs{x}`, `\norm{v}`, `\set{…}`, and `\rank`.
  - `\colored{red}{x}` colors part of a formula with a figure color (`gray`, `red`, `blue`, `green`, `orange`, `purple`), so that a formula and the figure next to it can share colors (for example the columns of a matrix and the arrows they describe).
- An undefined macro or a TeX syntax error fails the build. The message shows the file line and the TeX source.
- Write proof trees in the `bussproofs` notation with `\AxiomC`, `\UnaryInfC`, `\BinaryInfC`, `\TrinaryInfC`, `\RightLabel`, and `\LeftLabel`. Write a sequent as one formula inside `\AxiomC{$\Gamma \vdash A$}`. The `\Axiom…\fCenter…` form renders but gets no speech string.
- Each formula gets an English speech string in `aria-label`. Japanese inside `\text{…}` is read one character at a time, and `\norm` is read as "metric". These are limits of MathJax's speech engine.
- Text inside math is not checked by textlint, so a lone Latin letter as a variable is fine inside `$…$`.

## Definitions and theorems

Import the kinds you use, right after the frontmatter: `import { Definition, Lemma, Proposition, Theorem, Corollary } from '@/components/statement';`. Put blank lines before and after the tags so that the content is parsed as Markdown.

- Each element gets a label at build time: kind + page identifier + sequence number (`定理abs-3`). The counter is per page and shared by all kinds.
- `name="…"` adds a phrase to the heading. `id="…"` gives a stable identifier for references and the anchor. Without `id`, the anchor is the label, which changes when a statement is inserted before it.
- `<Ref to="id" />` links to a statement on the same page, and `<Ref page="pageId" to="id" />` to one on another page. Do not import `Ref`. Only statements with an `id` can be referenced.
- `<Proof of="id">` sets the proof title to the label of that statement. Do not combine it with `title`.
- The page identifier is the frontmatter `pageId`, or the file name (`index.mdx` uses the directory name). Use letters, digits, hyphens, and underscores, and keep it unique across the site. Pages referenced from other pages need lowercase ASCII file names. When the identifier makes labels too long, give the page a short name in `pageId` (`pageId: cont` gives `定理cont-7`); labels and `<Ref page>` both use it.
- Format MDX by hand: oxfmt does not touch it, because it breaks paragraphs that contain inline JSX. Keep each paragraph on one line.

Bad identifiers, duplicate identifiers, and references that point nowhere fail the build. The examples are on the test page (`site/src/content/docs/dev/notation.mdx`), and `dev/abs.mdx` is the page they refer to.

## Equation numbers

A display formula (`$$…$$`, on its own) that contains `\label{id}` gets a number. Formulas without `\label` are not numbered.

- The number is shown to the right as `(pageId-n)` and is counted per page in document order, separately from the statement counter. `<Ref to="id" />` (or `<Ref page="pageId" to="id" />`) links to it with the text `式(pageId-n)`.
- The identifier follows the same rules as a statement identifier and must not repeat a statement identifier on the same page.
- Write one `\label` per formula. For several aligned lines, use `aligned` inside one `$$…$$`. `\label` in inline math fails the build.
- Do not use `\ref` or `\eqref` inside math: MathJax renders an unknown reference as `(???)`, so the build rejects them. Reference equations from prose with `<Ref />`.
- `<Proof of="…">` accepts only statement identifiers, not equation identifiers.

## Keeping the explanations in sync

The notation page (`dev/notation.mdx`) is the reference for writers, and `dev/internals.mdx` explains how the system works (who does what, what is custom, how to extend it). When you change notation, macros, numbering, or the pipeline, update the matching page in the same commit, and keep the "Code map" in `CLAUDE.md` current.

## Lint

```sh
mise run lint:text
mise run lint:mdx
mise run lint:markdown
pnpm exec textlint --fix site/src/content README.md
```

- The checked files are `site/src/content` and `README.md` (textlint), `site/src/content/**/*.mdx` (remark-lint, `mise run lint:mdx`), and `README.md` and `docs/` (markdownlint). Staged files are also checked at commit time.
- `textlint --fix` only repairs what can be fixed automatically (spaces, colons, punctuation). Review the diff afterwards; a fix can change the meaning, for example a space inside a quoted string.
- In MDX, `textlint-disable` comments do not work. Rewrite the text to avoid a false positive.
- A new component in MDX needs no lint configuration. An unclosed tag or a broken `{…}` expression fails `lint:mdx` with the file position.
- Headings must be unique within a page (`no-duplicate-heading`), so give repeated section names a distinguishing word.
- To change the writing rules, edit `.textlintrc.yml` and `.textlint-prh.yml`. Give each `prh` rule `specs` with before and after examples.

## Writing examples

- Use natural sentences in notation examples. Avoid words like "test" or "sample", and mechanical repetition.
- Keep the code-block source and the rendered example identical. Editing only one of them makes them drift apart.
