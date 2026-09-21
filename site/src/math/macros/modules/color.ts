import type { MacroModule } from '../types';

/** 引数を2つ取るマクロの，引数の数． */
const TWO_ARGUMENTS = 2;

/**
 * 式の一部に，図と同じ色を付ける．色の名前は，図の`style.color`と同じ(`gray`，`red`，`blue`，`green`，`orange`，`purple`)で，
 * 明るい背景と暗い背景で，図の色と同じに切り替わる(`styles/figure.css`)．
 */
const color: MacroModule = {
  macros: {
    colored: [String.raw`\class{math-color-#1}{#2}`, TWO_ARGUMENTS],
  },
};

export default color;
