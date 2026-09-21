// MDXの検査．構文の誤り(閉じていないタグ，式の誤り)は，ビルドと同じ構文木で読んで検出する．
// astro.config.tsのremarkPlugins(remark-math)と，読み込む構文の拡張をそろえる．
import remarkFrontmatter from 'remark-frontmatter';
import remarkLint from 'remark-lint';
import remarkLintFencedCodeFlag from 'remark-lint-fenced-code-flag';
import remarkLintHeadingIncrement from 'remark-lint-heading-increment';
import remarkLintNoDuplicateHeadings from 'remark-lint-no-duplicate-headings';
import remarkLintUnorderedListMarkerStyle from 'remark-lint-unordered-list-marker-style';
import remarkMath from 'remark-math';
import remarkMdx from 'remark-mdx';
import remarkPresetLintConsistent from 'remark-preset-lint-consistent';
import remarkPresetLintRecommended from 'remark-preset-lint-recommended';
import remarkValidateLinks from 'remark-validate-links';

const config = {
  plugins: [
    remarkMdx,
    remarkMath,
    [remarkFrontmatter, ['yaml']],
    remarkLint,
    remarkPresetLintRecommended,
    remarkPresetLintConsistent,
    // 見出しは，1段ずつ深くし，同じページで重ねない．
    remarkLintHeadingIncrement,
    remarkLintNoDuplicateHeadings,
    [remarkLintUnorderedListMarkerStyle, '-'],
    remarkLintFencedCodeFlag,
    // ページ内のリンクの飛び先(#見出し)が存在することを確かめる．
    remarkValidateLinks,
  ],
};

export default config;
