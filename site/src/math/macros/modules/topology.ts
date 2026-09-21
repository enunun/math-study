import type { MacroModule } from '../types';

/** 引数を2つ取るマクロの，引数の数． */
const TWO_ARGUMENTS = 2;

/** 位相と，距離空間のマクロ．開球，閉包，逆像． */
const topology: MacroModule = {
  macros: {
    openball: [String.raw`B\left(#1, #2\right)`, TWO_ARGUMENTS],
    closure: [String.raw`\overline{#1}`, 1],
    preimage: [String.raw`#1^{-1}\left(#2\right)`, TWO_ARGUMENTS],
  },
};

export default topology;
