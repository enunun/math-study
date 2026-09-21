import type { RehypePlugins } from '@astrojs/markdown-remark';

import { rehypeAutospace } from './rehype-autospace';
import { rehypeFigures } from './rehype-figures';
import type { Options as FiguresOptions } from './rehype-figures';
import { rehypeFocusableTables } from './rehype-focusable-tables';
import { rehypeMathjax } from './rehype-mathjax';
import type { Options as MathjaxOptions } from './rehype-mathjax';
import { rehypeStatements } from './rehype-statements';
import type { Options as StatementsOptions } from './rehype-statements';

interface PipelineOptions {
  statements: StatementsOptions;
  figures: FiguresOptions;
  mathjax: MathjaxOptions;
}

/**
 * Markdownの後段(rehype)で使うプラグインを，実行する順に返す．順序には，次の制約がある．
 * - 定義，定理，式の番号付け(rehypeStatements)は，式の描画より前に行う．式のTeXと，要素のidを書き換えるためである．
 * - 図(rehypeFigures)は，式の描画より前に行う．図のラベルを，行内の数式として置き，後段のMathJaxに描画させるためである．
 * - 和文の隣の数式への隙間(rehypeAutospace)は，式の描画より後に行う．描画済みの数式の要素に，クラスを付けるためである．
 */
function buildRehypePlugins({ statements, figures, mathjax }: PipelineOptions): RehypePlugins {
  return [
    [rehypeStatements, statements],
    rehypeFocusableTables,
    [rehypeFigures, figures],
    [rehypeMathjax, mathjax],
    rehypeAutospace,
  ];
}

export { buildRehypePlugins };
