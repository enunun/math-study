/** TeXの式を，字句(命令，群，空白，文字)に分ける．式の書き換え(`inline-fraction.ts`)が使う． */

/** 字句．命令(`\frac`，`\{`)，波括弧で囲んだ群(`{…}`)，空白，1文字のいずれか． */
type Token =
  | { kind: 'command'; text: string }
  | { kind: 'group'; inner: string }
  | { kind: 'space'; text: string }
  | { kind: 'char'; text: string };

interface Read {
  token: Token;
  end: number;
}

/** `start`の`{`に対応する`}`の位置．エスケープした`\{`と`\}`は数えない．閉じていなければ，undefined． */
function closingBrace(tex: string, start: number): number | undefined {
  let depth = 0;
  for (let index = start; index < tex.length; index += 1) {
    const char = tex[index];
    if (char === '\\') {
      index += 1;
    } else if (char === '{' || char === '}') {
      depth += char === '{' ? 1 : -1;
      if (depth === 0) {
        return index;
      }
    }
  }
  return undefined;
}

function readCommand(tex: string, start: number): Read {
  const word = /^\\(?:[A-Za-z]+|.)?/su.exec(tex.slice(start))?.[0] ?? '\\';
  return { token: { kind: 'command', text: word }, end: start + word.length };
}

function readGroup(tex: string, start: number): Read {
  const close = closingBrace(tex, start);
  // 閉じていない群は，1文字の`{`として残す．
  return close === undefined
    ? { token: { kind: 'char', text: '{' }, end: start + 1 }
    : { token: { kind: 'group', inner: tex.slice(start + 1, close) }, end: close + 1 };
}

/** `start`から始まる字句と，その次の位置． */
function readToken(tex: string, start: number): Read {
  const char = tex.charAt(start);
  if (char === '\\') {
    return readCommand(tex, start);
  }
  if (char === '{') {
    return readGroup(tex, start);
  }
  const space = /^\s+/u.exec(tex.slice(start))?.[0];
  if (space !== undefined) {
    return { token: { kind: 'space', text: space }, end: start + space.length };
  }
  const codePoint = String.fromCodePoint(tex.codePointAt(start) ?? 0);
  return { token: { kind: 'char', text: codePoint }, end: start + codePoint.length };
}

function tokenize(tex: string): Token[] {
  const tokens: Token[] = [];
  for (let index = 0; index < tex.length;) {
    const { token, end } = readToken(tex, index);
    tokens.push(token);
    index = end;
  }
  return tokens;
}

function tokenText(token: Token): string {
  return token.kind === 'group' ? `{${token.inner}}` : token.text;
}

/** `index`の字句の文字列．列の外なら空文字列． */
function textAt(tokens: readonly Token[], index: number): string {
  const token = tokens[index];
  return token === undefined ? '' : tokenText(token);
}

/** `index`から空白を飛ばした，次の字句の位置． */
function skipSpaces(tokens: readonly Token[], index: number): number {
  return tokens[index]?.kind === 'space' ? index + 1 : index;
}

export { skipSpaces, textAt, tokenize, tokenText };
export type { Token };
