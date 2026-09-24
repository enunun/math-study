import { SCENE_VERSION } from './draft';
import type { SceneDraft } from './draft';
import type { SceneTemplate } from './template-types';

/** テイラー展開の図の見える範囲と，展開の次数． */
const TAYLOR_VIEW_X = 5;
const TAYLOR_VIEW_Y = 2;
const TAYLOR_ORDERS = [
  { order: 1, color: 'red' },
  { order: 3, color: 'orange' },
  { order: 5, color: 'green' },
  { order: 7, color: 'blue' },
] as const;

/**
 * 正弦関数と，原点のまわりのテイラー展開を1次，3次，5次，7次で打ち切った多項式．次数を上げるほど，
 * 原点から離れた所まで正弦関数に近づく．
 */
const TAYLOR_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '正弦関数y = sin xと，原点のまわりのテイラー展開を1次，3次，5次，7次で打ち切った多項式のグラフ．次数を上げるほど，広い範囲で正弦関数に近づく．',
  view: {
    x: [-TAYLOR_VIEW_X, TAYLOR_VIEW_X],
    y: [-TAYLOR_VIEW_Y, TAYLOR_VIEW_Y],
    unit: { x: '1cm', y: '1cm' },
  },
  objects: [
    { id: 'x_axis', type: 'axis', direction: 'x', label: 'x' },
    { id: 'y_axis', type: 'axis', direction: 'y', label: 'y' },
    { id: 'origin_label', type: 'label', at: [0, 0], anchor: 'north east', tex: '$O$' },
    {
      id: 'sine',
      type: 'graph',
      var: 'x',
      expr: 'sin(x)',
      domain: [-TAYLOR_VIEW_X, TAYLOR_VIEW_X],
    },
    ...TAYLOR_ORDERS.map(({ order, color }) => ({
      id: `taylor${order}`,
      type: 'taylor',
      of: 'sine',
      at: 0,
      order,
      style: { line: 'dashed', color },
    })),
  ],
};

/** 合成関数の図の見える範囲． */
const COMPOSITION_VIEW_X = 4;
const COMPOSITION_VIEW_Y = 1.5;

/**
 * 合成関数．関数f(u) = u^2とg(x) = sin xを定義し，g(x)と，合成したf(g(x)) = sin^2 xのグラフを描く．
 * グラフの式は，定義した関数の名前で書く．
 */
const COMPOSITION_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '合成関数．g(x) = sin xのグラフ(破線)と，f(u) = u^2を合成したf(g(x)) = sin^2 xのグラフ(実線)．',
  view: {
    x: [-COMPOSITION_VIEW_X, COMPOSITION_VIEW_X],
    y: [-COMPOSITION_VIEW_Y, COMPOSITION_VIEW_Y],
    unit: { x: '1cm', y: '1.5cm' },
  },
  objects: [
    { id: 'f', type: 'function', vars: ['u'], expr: 'u^2' },
    { id: 'g', type: 'function', vars: ['x'], expr: 'sin(x)' },
    { id: 'x_axis', type: 'axis', direction: 'x', label: 'x' },
    { id: 'y_axis', type: 'axis', direction: 'y', label: 'y' },
    { id: 'origin_label', type: 'label', at: [0, 0], anchor: 'north east', tex: '$O$' },
    {
      id: 'inner',
      type: 'graph',
      var: 'x',
      expr: 'g(x)',
      domain: [-COMPOSITION_VIEW_X, COMPOSITION_VIEW_X],
      style: { line: 'dashed', color: 'blue' },
    },
    {
      id: 'composite',
      type: 'graph',
      var: 'x',
      expr: 'f(g(x))',
      domain: [-COMPOSITION_VIEW_X, COMPOSITION_VIEW_X],
      style: { color: 'red' },
    },
  ],
};

/** 写像の図の見える範囲と，写す格子の範囲と刻み． */
const MAP_VIEW_LEFT = -1;
const MAP_VIEW_RIGHT = 3;
const MAP_VIEW_BOTTOM = -0.5;
const MAP_VIEW_TOP = 3;
const MAP_GRID_LEFT = 0.5;
const MAP_GRID_RIGHT = 1.5;
const MAP_GRID_TOP = 1;
const MAP_GRID_STEP = 0.25;

/**
 * 写像による格子の像．複素数の2乗(z = x + iyをz^2に写す)を，平面の写像F(x, y) = (x^2 - y^2, 2xy)として
 * 定義し，長方形の格子(灰色)を，その像(青)と並べる．像は，格子を写像で変換した`image`である．
 */
const MAP_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '写像F(x, y) = (x^2 - y^2, 2xy)(複素数の2乗)による格子の像．灰色の長方形の格子が，青い曲線の格子に写る．格子の線どうしの直交は，像でも保たれる．',
  view: {
    x: [MAP_VIEW_LEFT, MAP_VIEW_RIGHT],
    y: [MAP_VIEW_BOTTOM, MAP_VIEW_TOP],
    unit: { x: '1.2cm', y: '1.2cm' },
  },
  objects: [
    { id: 'F', type: 'map', vars: ['x', 'y'], expr: ['x^2 - y^2', '2*x*y'] },
    { id: 'x_axis', type: 'axis', direction: 'x', label: 'x' },
    { id: 'y_axis', type: 'axis', direction: 'y', label: 'y' },
    { id: 'origin_label', type: 'label', at: [0, 0], anchor: 'north east', tex: '$O$' },
    {
      id: 'grid',
      type: 'grid',
      x_step: MAP_GRID_STEP,
      y_step: MAP_GRID_STEP,
      x_range: [MAP_GRID_LEFT, MAP_GRID_RIGHT],
      y_range: [0, MAP_GRID_TOP],
      style: { color: 'gray', line: 'solid' },
    },
    {
      id: 'grid_image',
      type: 'image',
      of: 'grid',
      transform: [{ map: 'F' }],
      style: { color: 'blue', line: 'solid' },
    },
  ],
};

/** 関数(テイラー展開，合成関数)と写像の見本．`sample-scenes.ts`の平面の見本に並べる． */
const ANALYSIS_SAMPLE_SCENES: readonly SceneTemplate[] = [
  { id: 'taylor', label: 'テイラー展開', scene: TAYLOR_SCENE },
  { id: 'composition', label: '合成関数', scene: COMPOSITION_SCENE },
  { id: 'mapGrid', label: '写像による格子の像', scene: MAP_SCENE },
];

export { ANALYSIS_SAMPLE_SCENES };
