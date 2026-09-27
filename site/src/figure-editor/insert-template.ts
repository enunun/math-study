import { uniqueId } from './draft';
import type { SceneDraft, ViewKind } from './draft';
import { fieldsFor } from './fields';
import type { FieldSpec } from './fields';
import { isJsonObject, stringOf, toJsonObject, withField } from './json';
import type { Json, JsonObject } from './json';
import {
  NAMED_IN_EXPRESSIONS,
  expressionNamesOf,
  freeName,
  rewriteExpressions,
} from './rename-expressions';

/** 変換の手順の写像(`map`)を，付け替えた識別子に書き換える． */
function remapStep(step: Json, remap: ReadonlyMap<string, string>): Json {
  const map = isJsonObject(step) ? remap.get(stringOf(step, 'map')) : undefined;
  return map === undefined ? step : withField(toJsonObject(step), 'map', map);
}

/** 1つの項目の中の参照を，付け替えた識別子に書き換えた値．参照でない項目は，そのまま返す． */
function remapField(
  object: JsonObject,
  spec: FieldSpec,
  remap: ReadonlyMap<string, string>,
): Json | undefined {
  if (!('key' in spec)) {
    return undefined;
  }
  const value = object[spec.key];
  if (spec.kind === 'reference' && typeof value === 'string') {
    return remap.get(value) ?? value;
  }
  if ((spec.kind === 'references' || spec.kind === 'transform') && Array.isArray(value)) {
    return spec.kind === 'transform'
      ? value.map((step) => remapStep(step, remap))
      : value.map((item) => (typeof item === 'string' ? (remap.get(item) ?? item) : item));
  }
  return value;
}

/**
 * テンプレートの中の参照(`from`・`to`・`of`など，変換の手順の写像`map`も)を，付け替えた識別子に書き換える．
 * グループの外を指す参照(既存の点を指すときなど)は，`remap`にないので，そのまま残す．
 */
function rewriteReferences(
  object: JsonObject,
  kind: ViewKind,
  remap: ReadonlyMap<string, string>,
): JsonObject {
  const next: JsonObject = { ...object };
  for (const spec of fieldsFor(object, kind)) {
    const value = remapField(object, spec, remap);
    if (value !== undefined && 'key' in spec) {
      next[spec.key] = value;
    }
  }
  return next;
}

interface RenamedGroup {
  renamed: readonly JsonObject[];
  /** 元の識別子から，付け替えた識別子． */
  remap: ReadonlyMap<string, string>;
  /** 式の中の，元の名前から付け替えた名前．媒介変数，関数，点の座標(`p_x`など)の名前である． */
  names: ReadonlyMap<string, string>;
}

/**
 * グループの各オブジェクトに，重ならない識別子を付け，元の識別子からの対応(`remap`)を作る．
 * `uniqueId`に渡す，これまでに置いたオブジェクトの並び(`placed`)は，末尾に足すだけにする．
 */
function renameGroup(draft: SceneDraft, group: readonly JsonObject[]): RenamedGroup {
  const remap = new Map<string, string>();
  const names = new Map<string, string>();
  const placed: JsonObject[] = [...draft.objects];
  const renamed = group.map((object) => {
    const type = stringOf(object, 'type');
    const oldId = stringOf(object, 'id');
    const newId = NAMED_IN_EXPRESSIONS.has(type) ? freeName(oldId, placed) : uniqueId(type, placed);
    const created = { ...object, id: newId };
    placed.push(created);
    remap.set(oldId, newId);
    for (const [from, to] of expressionNamesOf({ type, oldId, newId })) {
      names.set(from, to);
    }
    return created;
  });
  return { renamed, remap, names };
}

/**
 * オブジェクトのまとまり(テンプレート)を，重ならない識別子に付け替えてから，図に追加する．
 * まとまりの中でのお互いの参照(六角形の辺が指す頂点など)と，式の中の名前(媒介変数，関数，点の座標)も，
 * 付け替えた先に合わせて書き換える．媒介変数と関数は，ぶつからなければ名前をそのまま使う．
 */
function insertObjects(
  draft: SceneDraft,
  kind: ViewKind,
  group: readonly JsonObject[],
): SceneDraft {
  const { renamed, remap, names } = renameGroup(draft, group);
  const rewritten = renamed.map((object) =>
    rewriteExpressions(rewriteReferences(object, kind, remap), kind, names),
  );
  return { ...draft, objects: [...draft.objects, ...rewritten] };
}

export { insertObjects };
