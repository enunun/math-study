import { uniqueId } from './draft';
import type { SceneDraft, ViewKind } from './draft';
import { fieldsFor } from './fields';
import type { FieldSpec } from './fields';
import { isJsonObject, stringOf, toJsonObject, withField } from './json';
import type { Json, JsonObject } from './json';

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
  remap: ReadonlyMap<string, string>;
}

/**
 * グループの各オブジェクトに，重ならない識別子を付け，元の識別子からの対応(`remap`)を作る．
 * `uniqueId`に渡す，これまでに置いたオブジェクトの並び(`placed`)は，末尾に足すだけにする．
 */
function renameGroup(draft: SceneDraft, group: readonly JsonObject[]): RenamedGroup {
  const remap = new Map<string, string>();
  const renamed: JsonObject[] = [];
  const placed: JsonObject[] = [...draft.objects];
  for (const object of group) {
    const newId = uniqueId(stringOf(object, 'type'), placed);
    const created = { ...object, id: newId };
    placed.push(created);
    remap.set(stringOf(object, 'id'), newId);
    renamed.push(created);
  }
  return { renamed, remap };
}

/**
 * オブジェクトのまとまり(テンプレート)を，重ならない識別子に付け替えてから，図に追加する．
 * まとまりの中でのお互いの参照(六角形の辺が指す頂点など)も，付け替えた先に合わせて書き換える．
 */
function insertObjects(
  draft: SceneDraft,
  kind: ViewKind,
  group: readonly JsonObject[],
): SceneDraft {
  const { renamed, remap } = renameGroup(draft, group);
  const rewritten = renamed.map((object) => rewriteReferences(object, kind, remap));
  return { ...draft, objects: [...draft.objects, ...rewritten] };
}

export { insertObjects };
