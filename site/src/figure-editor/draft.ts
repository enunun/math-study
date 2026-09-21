import defaults from './defaults.json';
import { arrayOf, isJsonObject, objectOf, parseJson, stringOf, toJsonObject } from './json';
import type { JsonObject } from './json';

/** 編集中の図のシーン．エンジンが読む形のJSONと同じで，誤りがあっても持てる． */
interface SceneDraft {
  version: string;
  description: string;
  view: JsonObject;
  objects: JsonObject[];
}

/** 図の種類．`view`の形で決まる． */
type ViewKind = 'plane' | 'space';

/** エンジンの版．新しい図の`version`に使う． */
const SCENE_VERSION = '0.1.0';

/** JSONの字下げの幅． */
const INDENT = 2;

/** 図の種類ごとの，`view`の初期値． */
function defaultView(kind: ViewKind): JsonObject {
  return toJsonObject(defaults.views[kind]);
}

/** 新しい，空の図． */
function emptyDraft(kind: ViewKind = 'plane'): SceneDraft {
  return { version: SCENE_VERSION, description: '新しい図', view: defaultView(kind), objects: [] };
}

/** `view`の形から，平面の図か，空間の図かを決める． */
function viewKind(draft: SceneDraft): ViewKind {
  return 'azimuth' in draft.view || 'elevation' in draft.view ? 'space' : 'plane';
}

type ParseResult = { ok: true; draft: SceneDraft } | { ok: false; message: string };

/** JSONの文字列を，図のシーンにする．エンジンの検査より前の，形だけを確かめる． */
function parseDraft(text: string): ParseResult {
  const parsed = parseJson(text);
  if (!parsed.ok) {
    return parsed;
  }
  const { value } = parsed;
  if (!isJsonObject(value)) {
    return { ok: false, message: 'シーンは，JSONのオブジェクトで書く．' };
  }
  const objects = arrayOf(value, 'objects').filter((object) => isJsonObject(object));
  if (objects.length !== arrayOf(value, 'objects').length) {
    return { ok: false, message: 'objectsの要素は，オブジェクトで書く．' };
  }
  return {
    ok: true,
    draft: {
      version: stringOf(value, 'version') || SCENE_VERSION,
      description: stringOf(value, 'description'),
      view: objectOf(value, 'view'),
      objects,
    },
  };
}

/** 図のシーンを，読みやすいJSONの文字列にする． */
function stringifyDraft(draft: SceneDraft): string {
  return JSON.stringify(draft, undefined, INDENT);
}

/** `base`から始まる，まだ使っていない識別子．`base`が空なら`object`から始める． */
function uniqueId(base: string, objects: readonly JsonObject[]): string {
  const taken = new Set(objects.map((object) => stringOf(object, 'id')));
  const stem = base === '' ? 'object' : base;
  let number = 1;
  while (taken.has(`${stem}${number}`)) {
    number += 1;
  }
  return `${stem}${number}`;
}

function addObject(draft: SceneDraft, object: JsonObject): SceneDraft {
  return { ...draft, objects: [...draft.objects, object] };
}

function replaceObject(draft: SceneDraft, index: number, object: JsonObject): SceneDraft {
  return { ...draft, objects: draft.objects.map((old, at) => (at === index ? object : old)) };
}

function removeObject(draft: SceneDraft, index: number): SceneDraft {
  return { ...draft, objects: draft.objects.filter((_, at) => at !== index) };
}

/** オブジェクトを，`delta`だけ前(負)か後ろ(正)へ動かす．端を越えるときは，そのままにする． */
function moveObject(draft: SceneDraft, index: number, delta: number): SceneDraft {
  const target = index + delta;
  const moved = draft.objects[index];
  if (moved === undefined || target < 0 || target >= draft.objects.length) {
    return draft;
  }
  const rest = draft.objects.filter((_, at) => at !== index);
  return { ...draft, objects: [...rest.slice(0, target), moved, ...rest.slice(target)] };
}

/** オブジェクトを選んでいないことを表す添字． */
const NO_SELECTION = -1;

/** 識別子が`id`のオブジェクトの添字．なければ`NO_SELECTION`． */
function indexOfId(draft: SceneDraft, id: string): number {
  return draft.objects.findIndex((object) => stringOf(object, 'id') === id);
}

export {
  NO_SELECTION,
  indexOfId,
  addObject,
  defaultView,
  emptyDraft,
  moveObject,
  parseDraft,
  removeObject,
  replaceObject,
  SCENE_VERSION,
  stringifyDraft,
  uniqueId,
  viewKind,
};
export type { SceneDraft, ViewKind };
