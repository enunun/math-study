import { COORDINATE_NAMES } from './coordinate-fields';
import type { ViewKind } from './draft';
import { fieldsFor } from './fields';
import type { FieldSpec } from './fields';
import { isJsonObject, stringOf } from './json';
import type { Json, JsonObject } from './json';

/** 式の中の名前．点の座標の名前(`p_x`)も，1つの名前として読む． */
const NAME = /[A-Za-z_][A-Za-z0-9_]*/gu;

/** 式の中の名前を，付け替えた名前に書き換える． */
function renameInExpression(source: string, names: ReadonlyMap<string, string>): string {
  return source.replaceAll(NAME, (name) => names.get(name) ?? name);
}

/** JSONの中で，式でない項目．名前のTeX，色，写像の識別子(参照として別に書き換える)である． */
const NOT_EXPRESSIONS: ReadonlySet<string> = new Set(['label', 'tex', 'color', 'map']);

/** JSONの値の中の，式の名前を書き換える．式でない項目(`NOT_EXPRESSIONS`)は変えない． */
function renameInJson(value: Json, names: ReadonlyMap<string, string>): Json {
  if (typeof value === 'string') {
    return renameInExpression(value, names);
  }
  if (Array.isArray(value)) {
    return value.map((item) => renameInJson(item, names));
  }
  if (isJsonObject(value)) {
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => [
        key,
        NOT_EXPRESSIONS.has(key) ? item : renameInJson(item, names),
      ]),
    );
  }
  return value;
}

/** 文字列で書く項目のうち，式であるもの(グラフや曲面の式，等値線の関数)． */
const EXPRESSION_TEXTS: ReadonlySet<string> = new Set(['expr', 'level']);

/** 中身がすべて式である項目の種類(数か式，位置，範囲，JSON，変換の手順)． */
const EXPRESSION_KINDS: ReadonlySet<FieldSpec['kind']> = new Set([
  'bound',
  'position',
  'domain2',
  'json',
  'transform',
]);

/** 項目が式を持つか．名前(`vars`など)や，ラベルのTeX，選択肢は，式ではない． */
function holdsExpressions(spec: FieldSpec): boolean {
  if (spec.kind === 'list') {
    return spec.item !== 'text' || EXPRESSION_TEXTS.has(spec.key);
  }
  if (spec.kind === 'text') {
    return EXPRESSION_TEXTS.has(spec.key);
  }
  return EXPRESSION_KINDS.has(spec.kind);
}

/** オブジェクトの，式を持つ項目の中の名前(媒介変数，関数，点)を，付け替えた名前に書き換える． */
function rewriteExpressions(
  object: JsonObject,
  kind: ViewKind,
  names: ReadonlyMap<string, string>,
): JsonObject {
  if (names.size === 0) {
    return object;
  }
  const next: JsonObject = { ...object };
  for (const spec of fieldsFor(object, kind)) {
    const value = 'key' in spec ? object[spec.key] : undefined;
    if (value !== undefined && 'key' in spec && holdsExpressions(spec)) {
      next[spec.key] = renameInJson(value, names);
    }
  }
  return next;
}

/** 式から名前で呼ぶ種類．識別子が式の名前になるので，ぶつからなければ名前をそのまま使う． */
const NAMED_IN_EXPRESSIONS: ReadonlySet<string> = new Set(['parameter', 'function']);

/** `id`が空いていれば`id`，使われていれば，後ろに2から順に番号を付けた，空いている名前． */
function freeName(id: string, placed: readonly JsonObject[]): string {
  const taken = new Set(placed.map((object) => stringOf(object, 'id')));
  if (id !== '' && !taken.has(id)) {
    return id;
  }
  let number = 2;
  while (taken.has(`${id}${number}`)) {
    number += 1;
  }
  return `${id}${number}`;
}

/** 付け替えたオブジェクトの，種類と，元の識別子と，新しい識別子． */
interface Renaming {
  type: string;
  oldId: string;
  newId: string;
}

/**
 * 付け替えで変わる，式の中の名前の組(元の名前，新しい名前)．媒介変数と関数は識別子そのもの，点は，
 * 点の式(`A + B`など)の中の識別子と，座標の名前(`<id>_x`など)が式に現れる．
 */
function expressionNamesOf({ type, oldId, newId }: Renaming): [string, string][] {
  if (oldId === newId) {
    return [];
  }
  if (NAMED_IN_EXPRESSIONS.has(type)) {
    return [[oldId, newId]];
  }
  if (type === 'point') {
    const coordinates = COORDINATE_NAMES.space.map((axis): [string, string] => [
      `${oldId}_${axis}`,
      `${newId}_${axis}`,
    ]);
    return [[oldId, newId], ...coordinates];
  }
  return [];
}

export { NAMED_IN_EXPRESSIONS, expressionNamesOf, freeName, rewriteExpressions };
