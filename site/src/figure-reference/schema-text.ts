/**
 * JSON Schemaの項目を，リファレンスに載せる文にする．型は「数か式」のような名前にし，共通の部品には
 * その節へのリンクを付ける．説明の中の`コード`は，コードとして示す．
 */

/** JSONの値を，スキーマの節として読む．オブジェクトでなければ，空の節にする． */
function isSchemaNode(value: unknown): value is SchemaNode {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function asSchemaNode(value: unknown): SchemaNode {
  return isSchemaNode(value) ? value : {};
}

/** リファレンスが読む，JSON Schemaの一部． */
interface SchemaNode {
  $ref?: string;
  type?: string | string[];
  const?: unknown;
  enum?: readonly unknown[];
  items?: SchemaNode;
  minItems?: number;
  maxItems?: number;
  minimum?: number;
  maximum?: number;
  oneOf?: readonly SchemaNode[];
  anyOf?: readonly SchemaNode[];
  properties?: Readonly<Record<string, SchemaNode>>;
  required?: readonly string[];
  description?: string;
}

/** 文の一片．`code`ならコードとして，`href`があればリンクとして示す． */
interface TextPart {
  text: string;
  code?: boolean;
  href?: string;
}

/** 共通の部品の節．`$defs`の名前と，見出し．この順に並べる． */
const COMMON_DEFS: readonly (readonly [string, string])[] = [
  ['bound', '数か式'],
  ['positiveNumberOrExpression', '正の数か式'],
  ['numberPair', '数の組'],
  ['boundPair', '数か式の組'],
  ['identifier', '識別子'],
  ['nonBlankString', '文字列'],
  ['length', '長さ'],
  ['position', '位置'],
  ['anchor', '名前を置く位置'],
  ['style', 'スタイル'],
  ['color', '色'],
  ['line', '線の種類'],
  ['hidden', '隠れた部分'],
  ['arrow', '矢じり'],
  ['direction', '軸の向き'],
  ['fill', '塗り'],
  ['tick', '目盛'],
  ['transform', '変換'],
  ['transformStep', '変換の手順'],
];

const COMMON_TITLES: ReadonlyMap<string, string> = new Map(COMMON_DEFS);

/** 共通の部品の節の，ページの中のリンク先． */
function defAnchor(name: string): string {
  return `def-${name}`;
}

/** オブジェクトの種類の節の，ページの中のリンク先． */
function objectAnchor(type: string): string {
  return `object-${type}`;
}

/** `#/$defs/<name>`の`<name>`． */
function refName(ref: string): string {
  return ref.slice(ref.lastIndexOf('/') + 1);
}

const PRIMITIVE_NAMES: Readonly<Record<string, string>> = {
  number: '数',
  integer: '整数',
  string: '文字列',
  boolean: '真偽値(trueかfalse)',
  object: 'オブジェクト',
};

/** 並びの個数．決まっていれば「(3個)」，幅があれば「(1〜64個)」，なければ空． */
function countText(node: SchemaNode): string {
  const { minItems, maxItems } = node;
  if (minItems !== undefined && minItems === maxItems) {
    return `(${minItems}個)`;
  }
  if (minItems !== undefined && maxItems !== undefined) {
    return `(${minItems}〜${maxItems}個)`;
  }
  return minItems === undefined ? '' : `(${minItems}個以上)`;
}

/** 「A」「Bか」のような選択肢を，「か」でつなぐ． */
function joinWithOr(choices: readonly TextPart[][]): TextPart[] {
  return choices.flatMap((choice, index) => (index === 0 ? choice : [{ text: 'か' }, ...choice]));
}

/** 値の並び(列挙)を，コードの並びにする． */
function valueParts(values: readonly unknown[]): TextPart[] {
  return values.flatMap((value, index) => [
    ...(index === 0 ? [] : [{ text: '，' }]),
    { text: JSON.stringify(value), code: true },
  ]);
}

/** 参照(`$ref`)の型の文．共通の部品なら，その節へのリンクにする． */
function refParts(ref: string): TextPart[] {
  const name = refName(ref);
  const title = COMMON_TITLES.get(name);
  return [
    title === undefined ? { text: name, code: true } : { text: title, href: `#${defAnchor(name)}` },
  ];
}

/**
 * 参照(`$ref`)，決まった値(`const`)，列挙(`enum`)の型の文．どれでもなければ`undefined`．
 */
function namedParts(node: SchemaNode): TextPart[] | undefined {
  if (node.$ref !== undefined) {
    return refParts(node.$ref);
  }
  if (node.const !== undefined) {
    return [{ text: JSON.stringify(node.const), code: true }];
  }
  return node.enum === undefined ? undefined : [...valueParts(node.enum), { text: 'のどれか' }];
}

/** 数や文字列などの型の文． */
function primitiveParts(node: SchemaNode): TextPart[] {
  const types = Array.isArray(node.type) ? node.type : [node.type ?? 'object'];
  return joinWithOr(types.map((type) => [{ text: PRIMITIVE_NAMES[type] ?? type }]));
}

/** 項目の型の文．参照は共通の部品の名前(リンクつき)に，並びは「〇〇の並び(n個)」にする． */
function typeParts(node: SchemaNode): TextPart[] {
  const named = namedParts(node);
  if (named !== undefined) {
    return named;
  }
  const choices = node.oneOf ?? node.anyOf;
  if (choices !== undefined) {
    return joinWithOr(choices.map((choice) => typeParts(choice)));
  }
  if (node.type === 'array' && node.items !== undefined) {
    return [...typeParts(node.items), { text: `の並び${countText(node)}` }];
  }
  return primitiveParts(node);
}

/** スキーマの説明から，スキーマの書き方についての注記(「スキーマでは表せない」)を除く． */
function withoutSchemaNotes(text: string): string {
  return text.replaceAll('(スキーマでは表せない)', '');
}

/** 説明の文を，`コード`とほかの部分に分ける． */
function inlineParts(text: string): TextPart[] {
  return withoutSchemaNotes(text)
    .split(/(?<code>`[^`]*`)/u)
    .filter((piece) => piece !== '')
    .map((piece) =>
      piece.startsWith('`') ? { text: piece.slice(1, -1), code: true } : { text: piece },
    );
}

export {
  asSchemaNode,
  COMMON_DEFS,
  defAnchor,
  inlineParts,
  objectAnchor,
  refName,
  typeParts,
  withoutSchemaNotes,
};
export type { SchemaNode, TextPart };
