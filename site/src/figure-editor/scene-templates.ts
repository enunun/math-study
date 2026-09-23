import { SCENE_VERSION, defaultView } from './draft';
import type { SceneDraft } from './draft';
import type { SceneTemplate } from './template-types';

/** 空間の図の軸を引く範囲．追加する軸の既定(`defaults.json`)と同じにする． */
const SPACE_AXIS_RANGE = 3;

/** 平面の図のx軸とy軸．範囲を省くので，見える範囲いっぱいに引く． */
const PLANE_AXES = [
  { id: 'x_axis', type: 'axis', direction: 'x', label: 'x' },
  { id: 'y_axis', type: 'axis', direction: 'y', label: 'y' },
];

/** 座標軸だけの，平面の図．見える範囲は，新しい図と同じ． */
const PLANE_AXES_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description: '座標軸だけの平面の図．',
  view: defaultView('plane'),
  objects: PLANE_AXES,
};

/** 座標軸と，1ごとの格子の，平面の図．格子を先に置き，軸を上に重ねる． */
const PLANE_GRID_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description: '座標軸と格子だけの平面の図．',
  view: defaultView('plane'),
  objects: [
    { id: 'grid', type: 'grid', x_step: 1, y_step: 1, style: { color: 'gray' } },
    ...PLANE_AXES,
  ],
};

/** 座標軸だけの，空間の図．見る向きは，新しい図と同じ． */
const SPACE_AXES_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description: '座標軸だけの空間の図．',
  view: defaultView('space'),
  objects: (['x', 'y', 'z'] as const).map((direction) => ({
    id: `${direction}_axis`,
    type: 'axis',
    direction,
    range: [-SPACE_AXIS_RANGE, SPACE_AXIS_RANGE],
    label: direction,
  })),
};

/** 図のテンプレート．中身のない出発点で，選ぶと今の図を置き換える． */
const SCENE_TEMPLATES: readonly SceneTemplate[] = [
  { id: 'plane-axes', label: '座標軸(平面)', scene: PLANE_AXES_SCENE },
  { id: 'plane-grid', label: '座標軸と格子(平面)', scene: PLANE_GRID_SCENE },
  { id: 'space-axes', label: '座標軸(空間)', scene: SPACE_AXES_SCENE },
];

export { SCENE_TEMPLATES };
