import { SCENE_VERSION, defaultView, emptyDraft } from './draft';
import type { SceneDraft, ViewKind } from './draft';
import type { JsonObject } from './json';
import type { SceneTemplate } from './template-types';

/** 空間の図の軸と格子を引く範囲．追加する軸の既定(`defaults.json`)と同じにする． */
const SPACE_AXIS_RANGE = 3;
/** 格子の刻み． */
const GRID_STEP = 1;
const GRID_STYLE: JsonObject = { color: 'gray' };

/** 原点Oの名前．軸の交わる所の右上に置く． */
function originLabel(kind: ViewKind): JsonObject {
  return {
    id: 'origin_label',
    type: 'label',
    at: kind === 'space' ? [0, 0, 0] : [0, 0],
    anchor: 'north east',
    tex: '$O$',
  };
}

/** 平面の図のx軸とy軸と原点．軸は，範囲を省くので，見える範囲いっぱいに引く． */
const PLANE_AXES: readonly JsonObject[] = [
  { id: 'x_axis', type: 'axis', direction: 'x', label: 'x' },
  { id: 'y_axis', type: 'axis', direction: 'y', label: 'y' },
  originLabel('plane'),
];

/** 空間の図のx軸，y軸，z軸と原点． */
const SPACE_AXES: readonly JsonObject[] = [
  ...(['x', 'y', 'z'] as const).map((direction) => ({
    id: `${direction}_axis`,
    type: 'axis',
    direction,
    range: [-SPACE_AXIS_RANGE, SPACE_AXIS_RANGE],
    label: direction,
  })),
  originLabel('space'),
];

/** 見える範囲が新しい図と同じで，`objects`だけを持つ図． */
function sceneOf(kind: ViewKind, description: string, objects: readonly JsonObject[]): SceneDraft {
  return { version: SCENE_VERSION, description, view: defaultView(kind), objects: [...objects] };
}

/**
 * 図のテンプレート．中身のない出発点で，選ぶと今の図を置き換える．座標軸のある図は，どれも原点Oを持つ．
 * 格子は，軸の下に敷く(先に置く)．空間の図の格子は，xy平面の上に引く．
 */
const SCENE_TEMPLATES: readonly SceneTemplate[] = [
  { id: 'plane-empty', label: '空の図(平面)', scene: emptyDraft('plane') },
  { id: 'space-empty', label: '空の図(空間)', scene: emptyDraft('space') },
  {
    id: 'plane-axes',
    label: '座標軸(平面)',
    scene: sceneOf('plane', '座標軸だけの平面の図．', PLANE_AXES),
  },
  {
    id: 'plane-grid',
    label: '座標軸と格子(平面)',
    scene: sceneOf('plane', '座標軸と格子だけの平面の図．', [
      { id: 'grid', type: 'grid', x_step: GRID_STEP, y_step: GRID_STEP, style: GRID_STYLE },
      ...PLANE_AXES,
    ]),
  },
  {
    id: 'space-axes',
    label: '座標軸(空間)',
    scene: sceneOf('space', '座標軸だけの空間の図．', SPACE_AXES),
  },
  {
    id: 'space-grid',
    label: '座標軸と格子(空間)',
    scene: sceneOf('space', '座標軸と，xy平面の格子だけの空間の図．', [
      {
        id: 'grid',
        type: 'grid',
        x_step: GRID_STEP,
        y_step: GRID_STEP,
        x_range: [-SPACE_AXIS_RANGE, SPACE_AXIS_RANGE],
        y_range: [-SPACE_AXIS_RANGE, SPACE_AXIS_RANGE],
        style: GRID_STYLE,
      },
      ...SPACE_AXES,
    ]),
  },
];

export { SCENE_TEMPLATES };
