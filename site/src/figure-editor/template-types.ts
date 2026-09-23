import type { SceneDraft, ViewKind } from './draft';
import type { JsonObject } from './json';

/**
 * 今編集している図に挿入する，部品のテンプレート．`kind`の図(平面・空間)でだけ使える．
 * 識別子は，挿入のときに，図の中で重ならないものへ付け替える(`insert-template.ts`)．
 */
interface ObjectTemplate {
  id: string;
  label: string;
  kind: ViewKind;
  objects: readonly JsonObject[];
}

/** 部品のテンプレートの，種類ごとのまとまり．ツールバーでは，まとまりごとに1つの選択欄になる． */
interface ObjectTemplateGroup {
  label: string;
  templates: readonly ObjectTemplate[];
}

/** 図全体を置き換える，図のテンプレートと見本．どちらも，シーンをそのまま持つ． */
interface SceneTemplate {
  id: string;
  label: string;
  scene: SceneDraft;
}

export type { ObjectTemplate, ObjectTemplateGroup, SceneTemplate };
