import type { MacroModule } from '../types';

/** 分野によらず使う，基本の記号．絶対値，ノルム，集合． */
const basics: MacroModule = {
  macros: {
    abs: [String.raw`\left|#1\right|`, 1],
    norm: [String.raw`\left\|#1\right\|`, 1],
    set: [String.raw`\left\{#1\right\}`, 1],
  },
};

export default basics;
