import { parseJson, withField } from './json';
import type { Json, JsonObject } from './json';

const NUMBER_TEXT = /^-?(?:\d+\.?\d*|\.\d+)(?:e[-+]?\d+)?$/iu;

/** 入力欄の文字列を，数か式にする．数の形なら数，そうでなければ式(文字列)である． */
function boundFromText(text: string): Json {
  const trimmed = text.trim();
  return NUMBER_TEXT.test(trimmed) ? Number(trimmed) : trimmed;
}

/** 数か式を，入力欄の文字列にする． */
function boundToText(value: Json | undefined): string {
  if (typeof value === 'number') {
    return String(value);
  }
  return typeof value === 'string' ? value : '';
}

/** 数の入力欄の文字列を，数にする．数の形でなければ，文字列のまま返し，エンジンの検査に任せる． */
const numberFromText = boundFromText;

/** `setItem`の入力． */
interface ItemUpdate {
  list: readonly Json[];
  index: number;
  item: Json;
  /** 結果の長さ．足りない所は，空の文字列で埋める． */
  count: number;
}

/** 配列の`index`番目を`item`にした，長さ`count`の配列． */
function setItem({ list, index, item, count }: ItemUpdate): Json[] {
  return Array.from({ length: count }, (_, at) => (at === index ? item : (list[at] ?? '')));
}

/** 位置が，点の式(`A + B`など，文字列)か． */
function isPointExpression(value: Json | undefined): value is string {
  return typeof value === 'string';
}

/** 入力欄の種類ごとの，文字列の読み方． */
const PARSERS = {
  text: (text: string): Json => text,
  number: numberFromText,
  bound: boundFromText,
} as const;

type ItemKind = keyof typeof PARSERS;

/** 入力欄の文字列を，種類に応じた値にする． */
function parseItem(kind: ItemKind, text: string): Json {
  return PARSERS[kind](text);
}

/** `commitField`の入力． */
interface FieldUpdate {
  object: JsonObject;
  key: string;
  value: Json;
  /** 省略できる項目か．省略できて値が空なら，項目を取り除く． */
  optional?: boolean;
}

/** 項目の値を書き換えた，新しいオブジェクト．省略できる項目が空なら，取り除く． */
function commitField({ object, key, value, optional = false }: FieldUpdate): JsonObject {
  const empty = value === '' || (Array.isArray(value) && value.every((item) => item === ''));
  return withField(object, key, optional && empty ? undefined : value);
}

/** JSONを直接書く欄の編集の結果．`keep`は，直前の値のままにして，理由(空でもよい)を示す． */
type JsonEdit = { kind: 'keep'; message: string } | { kind: 'set'; value: Json | undefined };

/** JSONを直接書く欄の文字列を読む．空の欄は，省略できる項目なら，項目を取り除く． */
function readJsonField(text: string, optional: boolean): JsonEdit {
  if (text.trim() === '') {
    return optional ? { kind: 'set', value: undefined } : { kind: 'keep', message: '' };
  }
  const parsed = parseJson(text);
  return parsed.ok
    ? { kind: 'set', value: parsed.value }
    : { kind: 'keep', message: parsed.message };
}

export {
  boundFromText,
  boundToText,
  commitField,
  isPointExpression,
  numberFromText,
  parseItem,
  readJsonField,
  setItem,
};
export type { FieldUpdate, ItemKind, ItemUpdate, JsonEdit };
