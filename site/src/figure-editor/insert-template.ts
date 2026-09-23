import { uniqueId } from './draft';
import type { SceneDraft, ViewKind } from './draft';
import { fieldsFor } from './fields';
import { stringOf } from './json';
import type { JsonObject } from './json';

/**
 * テンプレートの中の参照(`from`・`to`・`of`など)を，付け替えた識別子に書き換える．
 * グループの外を指す参照(既存の点を指すときなど)は，`remap`にないので，そのまま残す．
 */
function rewriteReferences(
  object: JsonObject,
  kind: ViewKind,
  remap: ReadonlyMap<string, string>,
): JsonObject {
  const next: JsonObject = { ...object };
  for (const spec of fieldsFor(object, kind)) {
    if (spec.kind === 'reference') {
      const mapped = remap.get(stringOf(next, spec.key));
      if (mapped !== undefined) {
        next[spec.key] = mapped;
      }
    } else if (spec.kind === 'references') {
      const value = next[spec.key];
      if (Array.isArray(value)) {
        next[spec.key] = value.map((item) =>
          typeof item === 'string' ? (remap.get(item) ?? item) : item,
        );
      }
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
