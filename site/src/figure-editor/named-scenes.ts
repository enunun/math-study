import { SCENE_VERSION } from './draft';
import type { SceneDraft } from './draft';

interface SceneTemplate {
  id: string;
  label: string;
  scene: SceneDraft;
}

/** メビウスの帯の見る向きと軸の範囲． */
const MOBIUS_AZIMUTH = 55;
const MOBIUS_ELEVATION = 30;
const MOBIUS_AXIS_RANGE = 3;
const MOBIUS_Z_AXIS_RANGE = 1.5;
/** 円周1周分(ラジアン)．帯に沿った角度`u`の定義域に使う，式(`pi`)で正確な値． */
const FULL_TURN_EXPR = '2*pi';

/** メビウスの帯．帯の中心の半径2，帯の幅1の，パラメータ表示の曲面．`u`が帯に沿った角度，`v`が幅方向． */
const MOBIUS_STRIP_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    'メビウスの帯．半周ひねりながら1周する帯で，表と裏の区別がない(向き付け不可能な)曲面の代表例．',
  view: { azimuth: MOBIUS_AZIMUTH, elevation: MOBIUS_ELEVATION, unit: '1cm' },
  objects: [
    {
      id: 'x_axis',
      type: 'axis',
      direction: 'x',
      range: [-MOBIUS_AXIS_RANGE, MOBIUS_AXIS_RANGE],
      label: 'x',
    },
    {
      id: 'y_axis',
      type: 'axis',
      direction: 'y',
      range: [-MOBIUS_AXIS_RANGE, MOBIUS_AXIS_RANGE],
      label: 'y',
    },
    {
      id: 'z_axis',
      type: 'axis',
      direction: 'z',
      range: [-MOBIUS_Z_AXIS_RANGE, MOBIUS_Z_AXIS_RANGE],
      label: 'z',
    },
    {
      id: 'band',
      type: 'surface',
      vars: ['u', 'v'],
      expr: [
        '(2 + v / 2 * cos(u / 2)) * cos(u)',
        '(2 + v / 2 * cos(u / 2)) * sin(u)',
        'v / 2 * sin(u / 2)',
      ],
      domain: [
        [0, FULL_TURN_EXPR],
        [-1, 1],
      ],
    },
  ],
};

/** コッホ曲線の見える範囲と，基本図形(線分)の長さ，繰り返す回数． */
const KOCH_VIEW_X_MAX = 5;
const KOCH_VIEW_Y_MAX = 2;
const KOCH_VIEW_Y_MIN = 0.5;
const KOCH_BASE_LENGTH = 4;
const KOCH_MIDPOINT_X = 2;
const KOCH_BUMP_ANGLE = 60;
const KOCH_DEPTH = 5;

/**
 * コッホ曲線．長さ4の線分を，3等分した真ん中を，正三角形の2辺で置き換える変換を，5回繰り返す．
 * 変換は，拡大縮小(1/3)・回転・平行移動を式(分数と`sqrt`)で正確に書く．
 */
const KOCH_CURVE_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    'コッホ曲線．線分を3等分し，真ん中を正三角形の2辺で置き換える操作を繰り返してできる，どこも微分できない曲線．',
  view: {
    x: [-1, KOCH_VIEW_X_MAX],
    y: [-KOCH_VIEW_Y_MIN, KOCH_VIEW_Y_MAX],
    unit: { x: '1.5cm', y: '1.5cm' },
  },
  objects: [
    { id: 'x_axis', type: 'axis', direction: 'x', label: 'x' },
    { id: 'y_axis', type: 'axis', direction: 'y', label: 'y' },
    {
      id: 'koch',
      type: 'fractal',
      base: [
        [0, 0],
        [KOCH_BASE_LENGTH, 0],
      ],
      closed: false,
      transforms: [
        [{ scale: ['1/3', '1/3'] }, { translate: [0, 0] }],
        [{ scale: ['1/3', '1/3'] }, { rotate: KOCH_BUMP_ANGLE }, { translate: ['4/3', 0] }],
        [
          { scale: ['1/3', '1/3'] },
          { rotate: -KOCH_BUMP_ANGLE },
          { translate: [KOCH_MIDPOINT_X, '2*sqrt(3)/3'] },
        ],
        [{ scale: ['1/3', '1/3'] }, { translate: ['8/3', 0] }],
      ],
      depth: KOCH_DEPTH,
      style: { color: 'blue' },
    },
  ],
};

const NAMED_SCENE_TEMPLATES: readonly SceneTemplate[] = [
  { id: 'mobius', label: 'メビウスの帯', scene: MOBIUS_STRIP_SCENE },
  { id: 'koch', label: 'コッホ曲線', scene: KOCH_CURVE_SCENE },
];

export { NAMED_SCENE_TEMPLATES };
export type { SceneTemplate };
