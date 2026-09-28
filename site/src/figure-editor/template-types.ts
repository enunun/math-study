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

/** 部品のテンプレートの，種類ごとのまとまり．ツールバーの選択欄では，まとまりごとに見出しが付く． */
interface ObjectTemplateGroup {
  label: string;
  templates: readonly ObjectTemplate[];
}

/** 部品のテンプレートの分類(図形，曲線など)．ツールバーでは，分類を選んでから部品を選ぶ． */
interface ObjectTemplateCategory {
  label: string;
  groups: readonly ObjectTemplateGroup[];
}

/** 図全体を置き換える，図のテンプレートと見本．どちらも，シーンをそのまま持つ． */
interface SceneTemplate {
  id: string;
  label: string;
  scene: SceneDraft;
}

export type { ObjectTemplate, ObjectTemplateCategory, ObjectTemplateGroup, SceneTemplate };
