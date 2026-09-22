import type { ValidateFunction } from 'ajv';
import Ajv2020 from 'ajv/dist/2020.js';

import schema from '@/../public/schema/scene.schema.json';
import { emptyDraft, stringifyDraft } from '@/figure-editor/draft';
import type { SceneDraft, ViewKind } from '@/figure-editor/draft';

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

/** オブジェクトなら，そのオブジェクト．そうでなければ，空のオブジェクト． */
function objectOf(value: unknown): Record<string, unknown> {
  return isRecord(value) ? value : {};
}

/** 信頼できる自分のJSONを，オブジェクトとして読む(`JSON.parse`は`any`を返すので，一度`unknown`を経由する)． */
function parsedJson(text: string): Record<string, unknown> {
  const value: unknown = JSON.parse(text);
  if (!isRecord(value)) {
    throw new Error('シーンのJSONは，オブジェクトで書く．');
  }
  return value;
}

// strictRequired: `grid`，`surface`，`curve`は，anyOf/oneOfの枝の中で，親のpropertiesにある項目を
// requiredにする(「x_stepかy_stepの少なくとも一方」のような，選べる必須項目を書く，一般的な書き方)．
// 厳格さは落とさない．
const ajv = new Ajv2020({ allErrors: true, strict: true, strictRequired: false });
const validate: ValidateFunction = ajv.compile(schema);

/** 最小限の，平面か空間のシーン．型ごとの検査は，オブジェクトを1つ足して書き換える． */
function minimalScene(kind: ViewKind): SceneDraft {
  return emptyDraft(kind);
}

function minimalObject(kind: ViewKind): Record<string, unknown> {
  return parsedJson(stringifyDraft(minimalScene(kind)));
}

export { minimalObject, minimalScene, objectOf, parsedJson, validate };
