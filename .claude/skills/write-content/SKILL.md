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
- Do not type spaces between Japanese and Latin text, digits, inline code, or math. The page adds a 1/4em gap when it is typeset (`rehype-autospace.ts`; see the `math-pipeline` skill).
- Use a full-width colon "：" after Japanese text, with no space after it ("公開先：`/x`"). Colons after Latin text stay half-width.
- Avoid AI-sounding writing: bold-prefix bullet lists ("**Item**：description"), hype, and emphasis in ordinary sentences.
- Write headings as names (noun phrases), not as sentences, clauses, questions, or verb phrases: "隠れる部分と見える部分の分割", not "曲線を隠れる部分と見える部分に分ける". Read the list of headings alone; every entry should read as a name. When a heading is renamed, check the links and tests that use its anchor.
- Place the comma by sentence structure, not by length or habit. Put it right after a sentence-initial topic ("画面は，原点を通り𝒅に垂直な平面である．"), at a boundary between clauses that each have their own predicate ("〜ので，", "〜なら，", "〜であり，"), and between the items of a list of nouns ("$x$軸，$y$軸，$z$軸"). Do not put a comma inside a phrase: not between the modifiers of one noun ("原点を通り𝒅に垂直な平面"), not after an adverbial phrase that leads straight into its verb ("この平面の上に右向きと上向きの2本の単位ベクトルを決める"), and not after a single-word conjunction at the start of a sentence ("したがって", "また", "そこで"). A sentence that needs many commas should be split. Decide sentence by sentence; a mechanical script moves commas to the wrong places. The limit of three commas below is only a ceiling.
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

Math is rendered at build time by MathJax 4. Write inline math as `$…$` and display math as `$$…$$`. The details and the rendered examples are on the test page (`site/src/content/docs/dev/notation.mdx`). In inline math, write fractions as `\frac` too: the build turns them into slashed fractions and adds parentheses where needed (`$\frac{a+b}{2}$` shows as (a+b)/2), while display math keeps them stacked. Do not type `a/b` by hand just for the inline look.

- Set the names of points and variables in italic everywhere, in prose (`$O$`, not a bare `O`) and in figure labels (`"tex": "$O$"`). A plain-text label is set in roman as text, so use it only for words. Use roman (`\mathrm{…}`) only for things that are conventionally upright, such as operator names.
- Write a vector as a bold italic letter (`\boldsymbol{d}`, not the upright `\mathbf{d}`). Write its components as a square-bracket matrix, `\begin{bmatrix} … \end{bmatrix}`, a column vector by default. A row vector separates its entries with `&` and has no commas. Points and coordinates follow the same rule; keep round brackets for intervals, function arguments, and pairs of parameters.
- Custom macros are defined as modules in `site/src/math/macros/modules/`, one file per area, default-exporting a `MacroModule`; `macros/index.ts` collects the files with `import.meta.glob`, so there is no registry to edit, and `merge.ts` fails the build when two modules define the same name. After changing them, update the list on the test page. Macros are named by meaning (`\RealNumbers`, not `\R`); do not add short aliases.
  - Number sets use the names of the LaTeX `numbersets` package: `\NaturalNumbers`, `\Integers`, `\RationalNumbers`, `\RealNumbers`, `\ComplexNumbers`. They take an optional style, `\RealNumbers[bfup]`, with `bb` (blackboard bold, the default), `bfup` (upright bold), and `bfit` (italic bold). `\NumberSet[style]{X}` sets any letter the same way. (MathJax has no `\csname`, and macro expansion inserts a space between a control sequence and a following letter, so the style is picked through an environment name instead — `numbersets.ts`'s `environments` export.)
  - The others are `\abs{x}`, `\norm{v}`, `\set{…}`, and `\rank`.
  - `\colored{red}{x}` colors part of a formula with a figure color (`gray`, `red`, `blue`, `green`, `orange`, `purple`) — the `html` extension of MathJax (`worker.ts`) adds the class `math-color-red`, and `site/src/styles/figure.css` maps it to the palette variable, so a formula and the figure next to it can share colors (for example the columns of a matrix and the arrows they describe). It works in build-time math only; browser MathJax (the calculator, the projection island) does not load the extension.
- An undefined macro or a TeX syntax error fails the build. The message shows the file line and the TeX source.
- Write proof trees in the `bussproofs` notation with `\AxiomC`, `\UnaryInfC`, `\BinaryInfC`, `\TrinaryInfC`, `\RightLabel`, and `\LeftLabel`. Write a sequent as one formula inside `\AxiomC{$\Gamma \vdash A$}`. The `\Axiom…\fCenter…` form renders but gets no speech string.
- Each formula gets an English speech string in `aria-label`. Japanese inside `\text{…}` is read one character at a time, and `\norm` is read as "metric". These are limits of MathJax's speech engine.
- Text inside math is not checked by textlint, so a lone Latin letter as a variable is fine inside `$…$`.

## Definitions and theorems

Import the kinds you use, right after the frontmatter: `import { Definition, Lemma, Proposition, Theorem, Corollary } from '@/components/statement';`. `Example` (例), `Problem` (問題), and `Answer` (解答) are also available and share the same counter. Put blank lines before and after the tags so that the content is parsed as Markdown.

- Each element gets a label at build time: kind + page identifier + sequence number (`定理abs-3`). The counter is per page and shared by all kinds.
- `name="…"` adds a phrase to the heading. `id="…"` gives a stable identifier for references and the anchor. Without `id`, the anchor is the label, which changes when a statement is inserted before it.
- `<Ref to="id" />` links to a statement on the same page, and `<Ref page="pageId" to="id" />` to one on another page. Do not import `Ref`. Only statements with an `id` can be referenced.
- `<Proof of="id">` sets the proof title to the label of that statement. Do not combine it with `title`.
- The page identifier is the frontmatter `pageId`, or the file name (`index.mdx` uses the directory name). Use letters, digits, hyphens, and underscores, and keep it unique across the site. Pages referenced from other pages need lowercase ASCII file names. When the identifier makes labels too long, give the page a short name in `pageId` (`pageId: cont` gives `定理cont-7`); labels and `<Ref page>` both use it.
- To add a new statement kind, extend `STATEMENT_KINDS` in `site/src/plugins/statements/collect.ts` and add a wrapper component under `site/src/components/statement/`.
- Give a figure that the text refers to an `id` and a `caption` (`<Figure src="…" id="fig-x" caption="…" />`), and refer to it with `<Ref to="fig-x" />`; the reader then sees "図page-1" instead of a vague "図の". A figure reference points to a figure of the same page. Figures without `id` are not numbered.
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
- A new component in MDX needs no lint configuration. `lint:mdx` is remark-lint (`.remarkrc.mjs`), not markdownlint; remark parses MDX the same way the build does, so an unclosed tag or a broken `{…}` expression fails at commit time, with the file position. Keep its remark plugins (`remark-mdx`, `remark-math`) in step with `astro.config.ts` — without `remark-math`, `{…}` inside `$$…$$` is read as a JSX expression. `remark-cli` prints a re-serialized copy of a file to stdout and rewrites some text (`[x]` becomes `\[x]`), so always pass `--no-stdout` and never use `--output`.
- Headings must be unique within a page (`no-duplicate-heading`), so give repeated section names a distinguishing word.
- To change the writing rules, edit `.textlintrc.yml` and `.textlint-prh.yml`. Give each `prh` rule `specs` with before and after examples.

## Transcriptions

A transcription reproduces one of the author's own earlier articles (such as the PDFs in their GitHub repositories) as a page. The existing ones are `topics/locus.mdx`, `topics/recurrence-guess.mdx`, and `topics/trigonometric-functions.mdx`.

- Start the page with `<Aside title="書き起こしについて">` saying only that the page is a Claude Code transcription of the article uploaded as a PDF to the named repository ("…にPDFでアップロードした記事の，Claude Codeによる書き起こしである．"). The `description` and the home card say the same, so that the use of AI is explicit.
- Keep the content and the argument. Fix obvious typos and mistakes, and rewrite sentences until they pass textlint like any other page; never exclude a page from textlint. Around display math, end the sentence before the formula ("次の式が成り立つ．") instead of continuing it after the formula.
- Write custom LaTeX macros of the original out in standard or site macros (`\apply{f}{x}` becomes `f(x)`). Write half-open intervals with `\lparen`, `\rparen`, `\lbrack`, or `\rbrack` (`\lparen a,b]`), because textlint checks bracket pairs in math too. Footnotes become `<Detail>` after the sentence's period, citations link to a 参考文献 section, and figures are redrawn as scenes in `site/src/figures/`.
- textlint counts half-width commas inside math toward the three-comma limit (`\set{\, … \,}` counts two), and the text inside one `<Detail>` as a single sentence.

## Writing examples

- Use natural sentences in notation examples. Avoid words like "test" or "sample", and mechanical repetition.
- Keep the code-block source and the rendered example identical. Editing only one of them makes them drift apart.
