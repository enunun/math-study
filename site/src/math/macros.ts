/** 展開後の式，引数の数，先頭の引数を省略したときの既定値(`[…]`で書く任意引数)． */
type Macro = string | [string, number] | [string, number, string];
type Macros = Record<string, Macro>;

/** 環境の名前と，`\begin`と`\end`の位置に置く式． */
type Environments = Record<string, [string, string]>;

/**
 * 数の集合のマクロは，LaTeXのnumbersetsパッケージ(https://github.com/enunun/numbersets)に合わせる．
 * `\NumberSet[スタイル]{N}`が文字を組み，`\RealNumbers[スタイル]`のような名前付きの集合が，それを呼ぶ．
 * スタイルは，パッケージのプリセットと同じbb(黒板太字)，bfup(立体の太字)，bfit(斜体の太字)で，
 * 省略するとbbになる．未知のスタイルは，未知の環境として，ビルドの失敗になる．
 */
const numberSetStyles = {
  bb: String.raw`\mathbb`,
  bfup: String.raw`\mathbf`,
  bfit: String.raw`\boldsymbol`,
} as const;

const defaultNumberSetStyle = 'bb';

/**
 * スタイルごとの環境の名前の接頭辞．
 * 任意引数の値から，スタイルごとの定義を選ぶために，環境を使う．
 * MathJaxには`\csname`がなく，マクロでは，制御綴りの直後に引数を置くと空白が挟まれて名前を作れない．
 * `\begin{…#1}`のように環境の名前の中なら，引数を文字列として連結できる．
 */
const STYLE_ENVIRONMENT_PREFIX = 'numbersetstyle';

/** `\NumberSet`の引数の数．スタイル(任意引数)と，組む文字． */
const NUMBER_SET_ARGUMENT_COUNT = 2;

/** 引数を2つ取るマクロの，引数の数． */
const TWO_ARGUMENTS = 2;

/** numbersetsの`\DeclareNumberSetCommand{名前}{文字}`に相当する，名前付きの数の集合のマクロ． */
function declareNumberSetCommand(symbol: string): Macro {
  return [String.raw`\NumberSet[#1]{${symbol}}`, 1, defaultNumberSetStyle];
}

/**
 * 自作マクロ．文書の中では，`\RealNumbers`や`\abs{x}`のように書く．
 * 値が文字列のときは展開後の式，`[式, 引数の数]`のときは，`#1`などの引数を取る式を表す．
 * 3つ目の要素があるときは，先頭の引数が，`[…]`で書く任意引数になる．
 * ビルド時の描画と，ブラウザでの描画で，同じ定義を使う．
 */
const macros: Macros = {
  NumberSet: [
    String.raw`\begin{${STYLE_ENVIRONMENT_PREFIX}#1}{#2}\end{${STYLE_ENVIRONMENT_PREFIX}#1}`,
    NUMBER_SET_ARGUMENT_COUNT,
    defaultNumberSetStyle,
  ],
  NaturalNumbers: declareNumberSetCommand('N'),
  Integers: declareNumberSetCommand('Z'),
  RationalNumbers: declareNumberSetCommand('Q'),
  RealNumbers: declareNumberSetCommand('R'),
  ComplexNumbers: declareNumberSetCommand('C'),
  abs: [String.raw`\left|#1\right|`, 1],
  norm: [String.raw`\left\|#1\right\|`, 1],
  set: [String.raw`\left\{#1\right\}`, 1],
  rank: String.raw`\operatorname{rank}`,
  openball: [String.raw`B\left(#1, #2\right)`, TWO_ARGUMENTS],
  closure: [String.raw`\overline{#1}`, 1],
  preimage: [String.raw`#1^{-1}\left(#2\right)`, TWO_ARGUMENTS],
};

/**
 * `\NumberSet`が，スタイルごとに呼ぶ環境．
 * `\begin`のコードは，残りの文字列の前に連結して読まれる．
 * `\begin{…}{N}`が`\mathbb{N}`になるよう，開始のコードは，引数を待つ`\mathbb`だけにする．
 */
const environments: Environments = Object.fromEntries(
  Object.entries(numberSetStyles).map(([style, command]) => [
    `${STYLE_ENVIRONMENT_PREFIX}${style}`,
    [command, ''] satisfies [string, string],
  ]),
);

export { environments, macros };
export type { Environments, Macro, Macros };
