import type { Json, JsonObject } from './json';
import type { ObjectTemplate } from './template-types';

// ベジエ曲線・スプライン曲線とベジエ曲面のテンプレート．制御点(スプライン曲線では通る点)は数で書き，
// 挿入したあと，フォームで書き換えて形を変える．

/** 制御点の座標に使う値．平面は，横が`[-5, 5]`，縦が`[-3, 3]`の既定の範囲に収める． */
const NEAR = 2;
const FAR = 3;
const HIGH = 2.5;

const BEZIER_SPLINE_TEMPLATES: readonly ObjectTemplate[] = [
  {
    id: 'bezier2-plane',
    label: '2次ベジエ曲線(制御点3個)',
    kind: 'plane',
    objects: [
      {
        id: 'c',
        type: 'curve',
        bezier: [
          [-NEAR, -1],
          [0, NEAR],
          [NEAR, -1],
        ],
      },
    ],
  },
  {
    id: 'bezier3-plane',
    label: '3次ベジエ曲線(制御点4個)',
    kind: 'plane',
    objects: [
      {
        id: 'c',
        type: 'curve',
        bezier: [
          [-FAR, -1],
          [-1, NEAR],
          [1, -NEAR],
          [FAR, 1],
        ],
      },
    ],
  },
  {
    id: 'spline-plane',
    label: 'スプライン曲線(通る点5個)',
    kind: 'plane',
    objects: [
      {
        id: 'c',
        type: 'curve',
        spline: [
          [-FAR, 0],
          [-NEAR, NEAR],
          [0, 0],
          [NEAR, -NEAR],
          [FAR, 0],
        ],
      },
    ],
  },
  {
    id: 'bezier2-space',
    label: '2次ベジエ曲線(制御点3個)',
    kind: 'space',
    objects: [
      {
        id: 'c',
        type: 'curve',
        bezier: [
          [NEAR, 0, 0],
          [0, 0, HIGH],
          [0, NEAR, 0],
        ],
      },
    ],
  },
  {
    id: 'bezier3-space',
    label: '3次ベジエ曲線(制御点4個)',
    kind: 'space',
    objects: [
      {
        id: 'c',
        type: 'curve',
        bezier: [
          [NEAR, -NEAR, 0],
          [NEAR, NEAR, 1],
          [-NEAR, NEAR, NEAR],
          [-NEAR, -NEAR, HIGH],
        ],
      },
    ],
  },
  {
    id: 'spline-space',
    label: 'スプライン曲線(通る点5個)',
    kind: 'space',
    objects: [
      {
        id: 'c',
        type: 'curve',
        spline: [
          [NEAR, 0, 0],
          [0, NEAR, 1],
          [-NEAR, 0, NEAR],
          [0, -NEAR, HIGH],
          [NEAR, 0, HIGH],
        ],
      },
    ],
  },
];

/** ベジエ曲面の制御点を並べる範囲(x，yともに`[-NET_HALF, NET_HALF]`)と，制御点の高さ． */
const NET_HALF = 2;
const NET_WIDTH = 4;
const NET_HIGH = 1.5;
/** ベジエ曲面のワイヤーフレームの刻み．変数の範囲は0から1なので，4等分する． */
const BEZIER_STEP = 0.25;

/**
 * 制御点の網．x，yは，`[-NET_HALF, NET_HALF]`を等間隔に分け，zは`heights`で与える．
 * `heights`の行はxの方向，列はyの方向に並べる．
 */
function bezierNet(heights: readonly (readonly number[])[]): Json[] {
  const rows = heights.length;
  return heights.map((row, i) =>
    row.map((z, j) => [
      -NET_HALF + (NET_WIDTH * i) / (rows - 1),
      -NET_HALF + (NET_WIDTH * j) / (row.length - 1),
      z,
    ]),
  );
}

/** ベジエ曲面．縁は，網の外周の制御点で決まる本当の縁なので描く．制御点の網も描く． */
function bezierSurface(heights: readonly (readonly number[])[]): JsonObject {
  return {
    id: 's',
    type: 'surface',
    bezier: bezierNet(heights),
    boundary: true,
    wireframe: {},
    wireframe_step: [BEZIER_STEP, BEZIER_STEP],
    control_net: {},
  };
}

const BEZIER_SURFACE_TEMPLATES: readonly ObjectTemplate[] = [
  {
    id: 'bezier-bilinear',
    label: '双1次ベジエ曲面(制御点2×2個)',
    kind: 'space',
    objects: [
      bezierSurface([
        [-1, 1],
        [1, -1],
      ]),
    ],
  },
  {
    id: 'bezier-biquadratic',
    label: '双2次ベジエ曲面(制御点3×3個)',
    kind: 'space',
    objects: [
      bezierSurface([
        [0, 1, 0],
        [1, NET_HIGH, 1],
        [0, 1, 0],
      ]),
    ],
  },
  {
    id: 'bezier-bicubic',
    label: '双3次ベジエ曲面(制御点4×4個)',
    kind: 'space',
    objects: [
      bezierSurface([
        [0, 1, 0, -1],
        [1, NET_HIGH, 1, 0],
        [0, 1, NET_HIGH, 1],
        [-1, 0, 1, 0],
      ]),
    ],
  },
];

export { BEZIER_SPLINE_TEMPLATES, BEZIER_SURFACE_TEMPLATES };
