import { SCENE_VERSION } from './draft';
import type { SceneDraft } from './draft';
import type { JsonObject } from './json';
import {
  A_ANCHORS,
  A_NAMES,
  B_ANCHORS,
  B_NAMES,
  FAMILY_STYLE,
  GAMMA,
  G_STYLE,
  INDICES,
  PATCH_END,
  REAL,
  RECIPROCAL,
  RECIPROCAL_BASIS,
  RECIPROCAL_STYLE,
  REAL_STYLE,
  basisVectors,
  indexParameter,
  integersBetween,
  latticeGrid,
  latticeLines,
  latticePoints,
  originPoint,
  parameter,
} from './reciprocal-parts';
import type { Pair } from './reciprocal-parts';
import type { SceneTemplate } from './template-types';

/**
 * 平面の逆格子の見本．実格子(左)と逆格子(右)を並べ，同じ媒介変数で両方を動かす．
 */

const PLANE_X_END = 7.6;
const PLANE_Y_END = 3.2;
const PLANE_UNIT = '0.8cm';
/** 下端のラベルを，見える範囲の下端から離す距離． */
const LABEL_MARGIN = 0.2;

const LENGTH_MIN = 1;
const LENGTH_MAX = 1.6;
const LENGTH_RANGE = [LENGTH_MIN, LENGTH_MAX] as const;
const A_VALUE = 1.2;
const B_VALUE = 1.4;
const GAMMA_VALUE = 70;
const GAMMA_MIN = 60;
const GAMMA_MAX = 120;
const GAMMA_RANGE = [GAMMA_MIN, GAMMA_MAX] as const;
/** 格子面の見本の角γの範囲．逆格子点が，どの値でも見える範囲に収まるよう狭める． */
const PLANES_GAMMA_VALUE = 80;
const PLANES_GAMMA_MIN = 70;
const PLANES_GAMMA_MAX = 110;
const PLANES_GAMMA_RANGE = [PLANES_GAMMA_MIN, PLANES_GAMMA_MAX] as const;

const GUIDE_STYLE: JsonObject = { color: 'gray', line: 'dashed' };
/** 逆格子の原点を通る，実格子の基本ベクトルの向きの破線の，片側の長さ． */
const GUIDE_REACH = 1.3;

function planeView(): SceneDraft['view'] {
  return {
    x: [-PLANE_X_END, PLANE_X_END],
    y: [-PLANE_Y_END, PLANE_Y_END],
    unit: { x: PLANE_UNIT, y: PLANE_UNIT },
  };
}

function bottomLabel(tex: string): JsonObject {
  return {
    id: 'condition',
    type: 'label',
    at: [0, -PLANE_Y_END + LABEL_MARGIN],
    anchor: 'south',
    tex,
  };
}

function reciprocalVectors(): JsonObject[] {
  return basisVectors({
    ids: ['O_star', 'B1', 'B2'],
    lattice: RECIPROCAL,
    names: B_NAMES,
    style: RECIPROCAL_STYLE,
    anchors: B_ANCHORS,
  });
}

/** 逆格子の原点O*を通る，実格子の基本ベクトルの向きの破線(b_1 ⊥ a_2，b_2 ⊥ a_1を見るため)． */
function guideLines(): JsonObject[] {
  const [first, second] = REAL.basis;
  const guides = [
    { id: 'guide1', vector: first, length: 'a', anchor: 'east', tex: A_NAMES[0] },
    { id: 'guide2', vector: second, length: 'b', anchor: 'north east', tex: A_NAMES[1] },
  ];
  return guides.flatMap(({ id, vector: [x, y], length, anchor, tex }): JsonObject[] => {
    const end = (sign: number): string[] => [
      `${RECIPROCAL.origin} + ${sign * GUIDE_REACH}*(${x})/${length}`,
      `${sign * GUIDE_REACH}*(${y})/${length}`,
    ];
    return [
      { id: `${id}_from`, type: 'point', at: end(-1) },
      { id: `${id}_to`, type: 'point', at: end(1) },
      { id, type: 'segment', from: `${id}_from`, to: `${id}_to`, style: GUIDE_STYLE },
      { id: `${id}_label`, type: 'label', at: `${id}_from`, anchor, tex: `$${tex}$` },
    ];
  });
}

const RECIPROCAL_LATTICE_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '実格子(左)と逆格子(右)．実格子の基本ベクトルa_1，a_2に対し，逆格子の基本ベクトルb_1，b_2は，a_i・b_j = δ_ijで決まる．b_1はa_2に，b_2はa_1に垂直で(右の破線はa_1，a_2の向き)，長さは1/(a sin γ)，1/(b sin γ)である．格子定数a，b，γをスライダーで動かすと，実格子の辺が伸びれば逆格子の辺が縮み，実格子の角γに対して逆格子の角は180° − γになることがわかる．',
  view: planeView(),
  objects: [
    parameter('a', A_VALUE, LENGTH_RANGE),
    parameter('b', B_VALUE, LENGTH_RANGE),
    parameter(GAMMA, GAMMA_VALUE, GAMMA_RANGE),
    ...latticeGrid('real', REAL),
    ...latticePoints('real', REAL, INDICES),
    originPoint('O', REAL, 'O'),
    ...basisVectors({
      ids: ['O', 'A1', 'A2'],
      lattice: REAL,
      names: A_NAMES,
      style: REAL_STYLE,
      anchors: A_ANCHORS,
    }),
    ...latticeGrid('reciprocal', RECIPROCAL),
    ...latticePoints('reciprocal', RECIPROCAL, INDICES),
    ...guideLines(),
    originPoint('O_star', RECIPROCAL, 'O^{*}'),
    ...reciprocalVectors(),
    bottomLabel(String.raw`$\boldsymbol{a}_i\cdot\boldsymbol{b}_j=\delta_{ij}$`),
  ],
};

/** 逆格子ベクトルG = h b_1 + k b_2の成分(逆格子の原点からの)． */
const G_COMPONENTS: Pair = [
  `h*(${RECIPROCAL_BASIS[0][0]}) + k*(${RECIPROCAL_BASIS[1][0]})`,
  `h*(${RECIPROCAL_BASIS[0][1]}) + k*(${RECIPROCAL_BASIS[1][1]})`,
];
/**
 * |G|^2．h = k = 0で0になるので，割る式では，見えないほど小さな数を足して，値を有限に保つ
 * (そのときの格子面の間隔の矢印は，長さ0になる)．
 */
const G_SQUARED = `(${G_COMPONENTS[0]})^2 + (${G_COMPONENTS[1]})^2 + 0.000000000001`;
/** 指数h，kの範囲(-INDEX_ENDからINDEX_ENDまで)． */
const INDEX_END = 2;
const INDEX_RANGE = [-INDEX_END, INDEX_END] as const;
/** 格子面の族を描く値の範囲．|h u + k v|の最大(|h| + |k|)×(格子の座標の範囲)を覆う． */
const FAMILY_END = Math.ceil((INDEX_END + INDEX_END) * PATCH_END);

const PLANES_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '格子面の族と逆格子点．指数(h, k)の格子面(平面の図では格子の線)は，格子の座標で書くとh u + k v = n(nは整数)であり，その間隔をdとする．逆格子ベクトルG = h b_1 + k b_2は，この族に垂直で，長さは1/dである．実格子の原点から最も近い面に下ろした矢印dは，Gと同じ向きを向く．逆格子点の1つ1つが，格子面の族の1つに当たる．スライダーで指数h，k(整数)や格子定数を変えて，面の間隔と逆格子点の位置が逆数の関係で動くことを確かめる．',
  view: planeView(),
  objects: [
    parameter('a', A_VALUE, LENGTH_RANGE),
    parameter('b', B_VALUE, LENGTH_RANGE),
    parameter(GAMMA, PLANES_GAMMA_VALUE, PLANES_GAMMA_RANGE),
    indexParameter('h', 1, INDEX_RANGE),
    indexParameter('k', 1, INDEX_RANGE),
    ...latticeGrid('real', REAL),
    latticeLines({
      id: 'planes',
      lattice: REAL,
      level: 'h*u + k*v',
      values: integersBetween(-FAMILY_END, FAMILY_END),
      style: FAMILY_STYLE,
    }),
    ...latticePoints('real', REAL, INDICES),
    originPoint('O', REAL, 'O'),
    {
      id: 'D',
      type: 'point',
      at: [
        `${REAL.origin} + (${G_COMPONENTS[0]})/(${G_SQUARED})`,
        `(${G_COMPONENTS[1]})/(${G_SQUARED})`,
      ],
    },
    { id: 'd', type: 'vector', from: 'O', to: 'D', style: G_STYLE },
    { id: 'd_label', type: 'label', at: '(O + D) / 2', anchor: 'north west', tex: '$d$' },
    ...latticePoints('reciprocal', RECIPROCAL, integersBetween(-INDEX_END, INDEX_END)),
    originPoint('O_star', RECIPROCAL, 'O^{*}'),
    ...reciprocalVectors(),
    {
      id: 'G',
      type: 'point',
      at: [`${RECIPROCAL.origin} + ${G_COMPONENTS[0]}`, G_COMPONENTS[1]],
      dot: true,
      style: { color: 'green' },
    },
    { id: 'G_vector', type: 'vector', from: 'O_star', to: 'G', style: G_STYLE },
    {
      id: 'G_label',
      type: 'label',
      at: 'G',
      anchor: 'south west',
      tex: String.raw`$\boldsymbol{G}$`,
    },
    bottomLabel(
      String.raw`$\boldsymbol{G}=h\boldsymbol{b}_1+k\boldsymbol{b}_2,\ |\boldsymbol{G}|=1/d$`,
    ),
  ],
};

const RECIPROCAL_PLANE_SCENES: readonly SceneTemplate[] = [
  { id: 'reciprocalLattice', label: '実格子と逆格子', scene: RECIPROCAL_LATTICE_SCENE },
  { id: 'latticePlanes', label: '格子面と逆格子点', scene: PLANES_SCENE },
];

export { RECIPROCAL_PLANE_SCENES };
