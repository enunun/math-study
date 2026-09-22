/**
 * 定数だけの式(`2*pi`，`-pi/4`など)を読んで，数にする．曲面のワイヤーフレームで，範囲の端が
 * `pi`や`e`を使った式で書かれていても，補助の線を引けるようにするためだけに使う．
 *
 * `+`，`-`，`*`，`/`，`^`と丸括弧，定数の`pi`と`e`だけを読む．変数や関数(`sin`など)は読めず，
 * そのときは`undefined`を返す(媒介変数を使った範囲は，値が決まらないので，これでよい)．
 * エンジンの式の評価(`crates/figure/src/expr/`)とは別の実装であり，正しさを保証するものではない．
 */

const CONSTANTS: Readonly<Record<string, number>> = { pi: Math.PI, e: Math.E };

type TokenKind = 'number' | 'name' | 'op' | 'lparen' | 'rparen';

interface Token {
  kind: TokenKind;
  text: string;
}

const NUMBER_TOKEN = /^\d+(?:\.\d+)?/u;
const NAME_TOKEN = /^[a-zA-Z_]\w*/u;
const OPERATORS = new Set(['+', '-', '*', '/', '^']);

/** 1文字の字句(演算子か丸括弧)．読めない文字なら`undefined`を返す． */
function singleCharToken(ch: string): Token | undefined {
  if (OPERATORS.has(ch)) {
    return { kind: 'op', text: ch };
  }
  if (ch === '(' || ch === ')') {
    return { kind: ch === '(' ? 'lparen' : 'rparen', text: ch };
  }
  return undefined;
}

/** 先頭の1個の字句と，その後ろに残る文字列．読めない文字なら`undefined`を返す． */
function nextToken(rest: string): [Token, string] | undefined {
  const number = NUMBER_TOKEN.exec(rest);
  if (number !== null) {
    return [{ kind: 'number', text: number[0] }, rest.slice(number[0].length)];
  }
  const name = NAME_TOKEN.exec(rest);
  if (name !== null) {
    return [{ kind: 'name', text: name[0] }, rest.slice(name[0].length)];
  }
  const token = singleCharToken(rest[0] ?? '');
  return token === undefined ? undefined : [token, rest.slice(1)];
}

/** 式を，字句の並びにする．読めない文字があれば`undefined`を返す． */
function tokenize(expr: string): Token[] | undefined {
  const tokens: Token[] = [];
  let rest = expr.trim();
  while (rest !== '') {
    const found = nextToken(rest);
    if (found === undefined) {
      return undefined;
    }
    const [token, remaining] = found;
    tokens.push(token);
    rest = remaining.trim();
  }
  return tokens;
}

/** 字句の並びを，読み進める位置． */
interface Cursor {
  tokens: readonly Token[];
  at: number;
}

function peek(cursor: Cursor): Token | undefined {
  return cursor.tokens[cursor.at];
}

function take(cursor: Cursor): Token | undefined {
  const token = peek(cursor);
  cursor.at += 1;
  return token;
}

function isOperator(token: Token | undefined, text: string): boolean {
  return token?.kind === 'op' && token.text === text;
}

/**
 * 式の文法(優先順位の低い方から)．`atom`が丸括弧の中で`expr`を呼ぶので，互いに再帰する．
 * メソッドどうしを，この`parse`オブジェクトを通して呼び，定義の前後を気にしなくてよいようにする．
 *
 *   expr  := term (('+'|'-') term)*
 *   term  := unary (('*'|'/') unary)*
 *   unary := ('-'|'+') unary | power
 *   power := atom ('^' unary)?
 *   atom  := number | name | '(' expr ')'
 */
const parse = {
  /** 丸括弧の中の式を読み，閉じ括弧まで確かめる． */
  parenthesized(cursor: Cursor): number | undefined {
    const value = parse.expr(cursor);
    return value !== undefined && take(cursor)?.kind === 'rparen' ? value : undefined;
  },

  atom(cursor: Cursor): number | undefined {
    const token = take(cursor);
    if (token === undefined) {
      return undefined;
    }
    if (token.kind === 'number') {
      return Number(token.text);
    }
    if (token.kind === 'name') {
      return CONSTANTS[token.text];
    }
    return token.kind === 'lparen' ? parse.parenthesized(cursor) : undefined;
  },

  /** `^`は右結合で，指数の側には単項の`+`・`-`も書ける(`2^-1`など)． */
  power(cursor: Cursor): number | undefined {
    const base = parse.atom(cursor);
    if (base === undefined || !isOperator(peek(cursor), '^')) {
      return base;
    }
    take(cursor);
    const exponent = parse.unary(cursor);
    return exponent === undefined ? undefined : base ** exponent;
  },

  unary(cursor: Cursor): number | undefined {
    const token = peek(cursor);
    if (!isOperator(token, '-') && !isOperator(token, '+')) {
      return parse.power(cursor);
    }
    take(cursor);
    const value = parse.unary(cursor);
    if (value === undefined) {
      return undefined;
    }
    return token?.text === '-' ? -value : value;
  },

  term(cursor: Cursor): number | undefined {
    let value = parse.unary(cursor);
    while (
      value !== undefined &&
      (isOperator(peek(cursor), '*') || isOperator(peek(cursor), '/'))
    ) {
      const op = take(cursor);
      const rhs = parse.unary(cursor);
      if (rhs === undefined) {
        return undefined;
      }
      value = op?.text === '*' ? value * rhs : value / rhs;
    }
    return value;
  },

  expr(cursor: Cursor): number | undefined {
    let value = parse.term(cursor);
    while (
      value !== undefined &&
      (isOperator(peek(cursor), '+') || isOperator(peek(cursor), '-'))
    ) {
      const op = take(cursor);
      const rhs = parse.term(cursor);
      if (rhs === undefined) {
        return undefined;
      }
      value = op?.text === '+' ? value + rhs : value - rhs;
    }
    return value;
  },
};

/** 定数だけの式を，数にする．読めないか，途中で終わらなければ`undefined`を返す． */
function evaluateConstant(expr: string): number | undefined {
  const tokens = tokenize(expr);
  if (tokens === undefined || tokens.length === 0) {
    return undefined;
  }
  const cursor: Cursor = { tokens, at: 0 };
  const value = parse.expr(cursor);
  return value !== undefined && Number.isFinite(value) && cursor.at === tokens.length
    ? value
    : undefined;
}

export { evaluateConstant };
