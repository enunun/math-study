/**
 * 行内の数式の分数(`\frac`)を，斜線の分数に書き換える．
 * 本文の行の中で，分子と分母を縦に積むと，文字が小さくなり，行の間隔も乱れるためである．
 * 読み違えのないよう，必要なところにだけ括弧を補う．
 * - 分子は，因子を掛け合わせた並び(`3\pi`，`x^2`)でなければ，括弧で囲む(`(a+b)/2`)．
 * - 分母は，1つの因子でなければ，括弧で囲む(`1/(2x)`)．
 * - 分数の後ろに因子や添字が続くときは，分数全体を括弧で囲む(`(1/2)x`)．
 */

import { skipSpaces, textAt, tokenize, tokenText } from './tex-tokens';
import type { Token } from './tex-tokens';

const FRAC = String.raw`\frac`;
const LEFT = String.raw`\left`;

/** 命令の名前を空白で区切って並べた文字列から，命令の集合を作る． */
function commands(names: string): Set<string> {
  return new Set(names.split(/\s+/u).map((name) => `\\${name}`));
}

/** 1つの因子として扱う記号の命令．ギリシャ文字など． */
const SYMBOLS = commands(
  `
  alpha beta gamma delta epsilon varepsilon zeta eta theta vartheta iota kappa lambda mu nu xi
  pi varpi rho varrho sigma varsigma tau upsilon phi varphi chi psi omega
  Gamma Delta Theta Lambda Xi Pi Sigma Upsilon Phi Psi Omega
  infty ell hbar partial nabla
`.trim(),
);

/** 引数を1つ取り，全体で1つの因子になる命令．根号，書体，アクセント． */
const WRAPPERS = commands(
  `
  sqrt mathrm mathbf mathit mathsf mathtt mathcal mathbb mathfrak boldsymbol bm
  overline bar hat widehat tilde widetilde vec dot ddot
`.trim(),
);

/**
 * 命令の引数になれる字句か．閉じていない群(`{`)や，引数の欠けた命令は，書き換えずにMathJaxへ渡し，構文の誤りとして報告させる．
 */
function isArgument(token: Token | undefined): token is Token {
  return token !== undefined && token.kind !== 'space' && !['{', '}'].includes(tokenText(token));
}

/** 命令の引数．群なら中身，それ以外は字句そのもの． */
function argumentText(token: Token): string {
  return token.kind === 'group' ? token.inner : tokenText(token);
}

/** `open`の位置の開き括弧に対応する，閉じ括弧の次の位置． */
function afterClosingParen(tokens: readonly Token[], open: number): number | undefined {
  let depth = 0;
  for (let index = open; index < tokens.length; index += 1) {
    const text = textAt(tokens, index);
    if (text === '(' || text === ')') {
      depth += text === '(' ? 1 : -1;
      if (depth === 0) {
        return index + 1;
      }
    }
  }
  return undefined;
}

/** 根号の次数(`\sqrt[3]{x}`)があれば，その後ろの位置．なければ`index`のまま． */
function skipOptionalArgument(tokens: readonly Token[], index: number): number {
  if (textAt(tokens, index) !== '[') {
    return index;
  }
  const close = tokens.findIndex((token, at) => at > index && tokenText(token) === ']');
  return close === -1 ? tokens.length : close + 1;
}

/** 数字の続く間を1つの数として読み，その次の位置を返す． */
function afterNumber(tokens: readonly Token[], index: number): number {
  let end = index;
  while (/^[0-9.]$/u.test(textAt(tokens, end))) {
    end += 1;
  }
  return end;
}

/** 添字1つの字句の数．`^`か`_`と，その引数． */
const SCRIPT_LENGTH = 2;

/** 添字を読み飛ばした位置． */
function skipScripts(tokens: readonly Token[], index: number): number {
  let end = index;
  while (['^', '_'].includes(textAt(tokens, end))) {
    end += SCRIPT_LENGTH;
  }
  return end;
}

/**
 * `index`から1つの因子(添字を除く)を読み，その次の位置を返す．因子でなければ，undefined．
 * 群(`{…}`)は，ここでは因子とし，中身が1つの因子かどうかは，呼び出し側が確かめる．
 */
function afterFactorBody(tokens: readonly Token[], index: number): number | undefined {
  const text = textAt(tokens, index);
  if (/^[0-9.]$/u.test(text)) {
    return afterNumber(tokens, index);
  }
  if (/^[A-Za-z]$/u.test(text) || SYMBOLS.has(text) || tokens[index]?.kind === 'group') {
    return index + 1;
  }
  if (WRAPPERS.has(text)) {
    return skipOptionalArgument(tokens, index + 1) + 1;
  }
  return text === '(' ? afterClosingParen(tokens, index) : undefined;
}

/**
 * 式を因子に分けて，その数を返す．因子の並びとして読めないとき(演算子や関数を含むとき)は，undefinedにする．
 * 因子は，文字，数，記号の命令，根号や書体の命令，括弧で囲んだ部分，中身が1つの因子の群で，添字を付けてもよい．
 */
function countFactors(tex: string): number | undefined {
  const tokens = tokenize(tex).filter((token) => token.kind !== 'space');
  let count = 0;
  for (let index = 0; index < tokens.length; count += 1) {
    const token = tokens[index];
    const end = afterFactorBody(tokens, index);
    if (end === undefined || (token?.kind === 'group' && countFactors(token.inner) !== 1)) {
      return undefined;
    }
    index = skipScripts(tokens, end);
  }
  return count === 0 ? undefined : count;
}

/** 分数の後ろに続くと，掛け算や添字として読める字句か． */
function continuesAsFactor(token: Token | undefined): boolean {
  if (token === undefined || token.kind === 'space') {
    return false;
  }
  if (token.kind === 'group') {
    return true;
  }
  if (token.kind === 'command') {
    // 記号の命令のほか，大文字で始まる命令(`\Integers`などの自作のマクロ)も因子として扱う．
    const { text } = token;
    return (
      SYMBOLS.has(text) ||
      WRAPPERS.has(text) ||
      [LEFT, FRAC].includes(text) ||
      /^\\[A-Z]/u.test(text)
    );
  }
  return /^[A-Za-z0-9(^_]$/u.test(token.text);
}

function parenthesize(tex: string): string {
  return `(${tex})`;
}

/** 分子と分母から，斜線の分数を作る． */
function slash(numerator: string, denominator: string): string {
  const top = countFactors(numerator) === undefined ? parenthesize(numerator) : numerator;
  const bottom = countFactors(denominator) === 1 ? denominator : parenthesize(denominator);
  return `${top}/${bottom}`;
}

/** `\frac`の引数と，分数の次の位置．`wrap`は，後ろに因子が続き，分数全体を括弧で囲むかどうか． */
interface Fraction {
  numerator: string;
  denominator: string;
  wrap: boolean;
  end: number;
}

/** `index`が`\frac`なら，その引数を読む．`\frac`でないか，引数が欠けていれば，undefined． */
function readFraction(tokens: readonly Token[], index: number): Fraction | undefined {
  const token = tokens[index];
  const numeratorAt = skipSpaces(tokens, index + 1);
  const denominatorAt = skipSpaces(tokens, numeratorAt + 1);
  const numerator = tokens[numeratorAt];
  const denominator = tokens[denominatorAt];
  if (
    token?.kind !== 'command' ||
    token.text !== FRAC ||
    !isArgument(numerator) ||
    !isArgument(denominator)
  ) {
    return undefined;
  }
  return {
    numerator: argumentText(numerator),
    denominator: argumentText(denominator),
    wrap: continuesAsFactor(tokens[skipSpaces(tokens, denominatorAt + 1)]),
    end: denominatorAt + 1,
  };
}

/** `index`の字句を書き換えた文字列と，次の位置．分数と群の中身は，`rewrite`で書き換える． */
function rewriteAt(
  tokens: readonly Token[],
  index: number,
  rewrite: (tex: string) => string,
): { text: string; end: number } {
  const fraction = readFraction(tokens, index);
  if (fraction === undefined) {
    const token = tokens[index];
    const text = token?.kind === 'group' ? `{${rewrite(token.inner)}}` : textAt(tokens, index);
    return { text, end: index + 1 };
  }
  const text = slash(rewrite(fraction.numerator), rewrite(fraction.denominator));
  return { text: fraction.wrap ? parenthesize(text) : text, end: fraction.end };
}

/** 行内の式のTeXの，すべての`\frac`を，斜線の分数に書き換える．群の内側の分数も書き換える． */
function slashFractions(tex: string): string {
  const tokens = tokenize(tex);
  const parts: string[] = [];
  for (let index = 0; index < tokens.length;) {
    const { text, end } = rewriteAt(tokens, index, slashFractions);
    parts.push(text);
    index = end;
  }
  return parts.join('');
}

export { slashFractions };
