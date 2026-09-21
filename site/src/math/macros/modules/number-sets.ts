import type { Macro, MacroModule } from '../types';

/**
 * 数の集合のマクロ．LaTeXのnumbersetsパッケージ(https://github.com/enunun/numbersets)に合わせる．
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

/** numbersetsの`\DeclareNumberSetCommand{名前}{文字}`に相当する，名前付きの数の集合のマクロ． */
function declareNumberSetCommand(symbol: string): Macro {
  return [String.raw`\NumberSet[#1]{${symbol}}`, 1, defaultNumberSetStyle];
}

const numberSets: MacroModule = {
  macros: {
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
  },
  /**
   * `\NumberSet`が，スタイルごとに呼ぶ環境．
   * `\begin`のコードは，残りの文字列の前に連結して読まれる．
   * `\begin{…}{N}`が`\mathbb{N}`になるよう，開始のコードは，引数を待つ`\mathbb`だけにする．
   */
  environments: Object.fromEntries(
    Object.entries(numberSetStyles).map(([style, command]) => [
      `${STYLE_ENVIRONMENT_PREFIX}${style}`,
      [command, ''] satisfies [string, string],
    ]),
  ),
};

export default numberSets;
