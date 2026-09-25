import type { MacroModule } from '../types';

/** 分野によらず使う，基本の記号．絶対値，ノルム，集合，半開区間の丸括弧． */
const basics: MacroModule = {
  macros: {
    abs: [String.raw`\left|#1\right|`, 1],
    norm: [String.raw`\left\|#1\right\|`, 1],
    set: [String.raw`\left\{#1\right\}`, 1],
    // 半開区間の片側の丸括弧．LaTeXのmathtoolsと同じ名前で，MathJaxの標準にはない．
    // 対になる括弧のない丸括弧を本文に書くと，textlintが括弧の対応の誤りとして指摘する．
    lparen: String.raw`\mathopen{(}`,
    rparen: String.raw`\mathclose{)}`,
  },
};

export default basics;
