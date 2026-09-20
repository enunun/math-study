type Macros = Record<string, string | [string, number]>;

/**
 * 自作マクロ．文書の中では，`\R`のように書く．
 * 値が文字列のときは展開後の式，`[式, 引数の数]`のときは，`#1`などの引数を取る式を表す．
 * ビルド時の描画と，ブラウザでの描画で，同じ定義を使う．
 */
const macros: Macros = {
  R: String.raw`\mathbb{R}`,
  N: String.raw`\mathbb{N}`,
  Z: String.raw`\mathbb{Z}`,
  Q: String.raw`\mathbb{Q}`,
  C: String.raw`\mathbb{C}`,
  abs: [String.raw`\left|#1\right|`, 1],
  norm: [String.raw`\left\|#1\right\|`, 1],
  set: [String.raw`\left\{#1\right\}`, 1],
  rank: String.raw`\operatorname{rank}`,
};

export { macros };
export type { Macros };
