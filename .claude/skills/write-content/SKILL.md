---
name: write-content
description: Rules and notation for writing the site content (Markdown, MDX) and README. Covers Japanese writing rules, the folding components, and how to run and handle the text lint. Use it before writing or editing content.
---

# write-content

Rules and notation for the site content (`site/src/content/**`) and `README.md`, which are written in Japanese. textlint and markdownlint check them mechanically. Everything else meant for Claude (`CLAUDE.md`, skills, `docs/**`) is written in English.

## Writing rules for Japanese text

- Use the plain form (である調). This covers body text and list items; headings are unconstrained.
- Use the full-width comma "，" (U+FF0C) and the full-width period "．" (U+FF0E). Never use "、" or "。".
- Put no space between Japanese and Latin text, digits, or inline code. Write "MDXの記法" and "`mise run dev`を実行". Do not write "MDX の記法".
- Use a full-width colon "：" after Japanese text, with no space after it ("公開先：`/x`"). Colons after Latin text stay half-width.
- Avoid AI-sounding writing: bold-prefix bullet lists ("**Item**：description"), hype, and emphasis in ordinary sentences.
- Keep each sentence within 100 characters and within three commas. Do not repeat the same particle within a sentence.
- A lone Latin letter (a variable such as x) is flagged as an "unnatural alphabet". Until math can be written with MathJax, use example sentences without variables.

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

## Lint

```sh
mise run lint:text
mise run lint:markdown
pnpm exec textlint --fix site/src/content README.md
```

- The checked files are `site/src/content` and `README.md` (textlint), plus `docs/` (markdownlint). Staged files are also checked at commit time.
- `textlint --fix` only repairs what can be fixed automatically (spaces, colons, punctuation). Review the diff afterwards; a fix can change the meaning, for example a space inside a quoted string.
- In MDX, `textlint-disable` comments do not work. Rewrite the text to avoid a false positive.
- When using a new component in MDX, add it to `allowed_elements` in `.markdownlint-cli2.jsonc`.
- To change the writing rules, edit `.textlintrc.yml` and `.textlint-prh.yml`. Give each `prh` rule `specs` with before and after examples.

## Writing examples

- Use natural sentences in notation examples. Avoid words like "test" or "sample", and mechanical repetition.
- Keep the code-block source and the rendered example identical. Editing only one of them makes them drift apart.
