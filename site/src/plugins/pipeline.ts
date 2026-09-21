import type { RehypePlugins } from '@astrojs/markdown-remark';

import { rehypeAutospace } from './rehype-autospace';
import { rehypeFocusableTables } from './rehype-focusable-tables';
import { rehypeMathjax } from './rehype-mathjax';
import type { Options as MathjaxOptions } from './rehype-mathjax';
import { rehypeStatements } from './rehype-statements';
import type { Options as StatementsOptions } from './rehype-statements';

interface PipelineOptions {
  statements: StatementsOptions;
  mathjax: MathjaxOptions;
}

/**
 * Markdownの後段(rehype)で使うプラグインを，実行する順に返す．順序には，次の制約がある．
 * - 定義，定理，式の番号付け(rehypeStatements)は，式の描画より前に行う．式のTeXと，要素のidを書き換えるためである．
 * - 和文の隣の数式への隙間(rehypeAutospace)は，式の描画より後に行う．描画済みの数式の要素に，クラスを付けるためである．
 */
function buildRehypePlugins({ statements, mathjax }: PipelineOptions): RehypePlugins {
  return [
    [rehypeStatements, statements],
    rehypeFocusableTables,
    [rehypeMathjax, mathjax],
    rehypeAutospace,
  ];
}

export { buildRehypePlugins };
