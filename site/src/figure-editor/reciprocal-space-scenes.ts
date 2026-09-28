import { generalBasis } from './bravais-cells';
import { SCENE_VERSION } from './draft';
import type { SceneDraft } from './draft';
import type { JsonObject } from './json';
import { RECIPROCAL_STYLE, REAL_STYLE, parameter } from './reciprocal-parts';
import type { SceneTemplate } from './template-types';

/**
 * 空間の逆格子の見本．逆格子の図は，実格子の図の右(画面の右向き)にずらして描く．
 */

type Triple = readonly [string, string, string];

const SPACE_AZIMUTH = -50;
const SPACE_ELEVATION = 30;
const DEGREES_PER_HALF_TURN = 180;
/** 丸めた座標の桁数(小数)． */
const COORDINATE_DIGITS = 3;
const AZIMUTH = (SPACE_AZIMUTH * Math.PI) / DEGREES_PER_HALF_TURN;
/** 画面の右向き(-sin φ, cos φ, 0)の，x成分とy成分． */
const SCREEN_RIGHT = [
  Number((-Math.sin(AZIMUTH)).toFixed(COORDINATE_DIGITS)),
  Number(Math.cos(AZIMUTH).toFixed(COORDINATE_DIGITS)),
] as const;

/** 画面の右向きへ`distance`(式)だけ進んだ点の，x座標とy座標の式． */
function toRight(distance: string): readonly [string, string] {
  return [`(${distance})*${SCREEN_RIGHT[0]}`, `(${distance})*${SCREEN_RIGHT[1]}`];
}

const EDGE_STYLE: JsonObject = { color: 'gray' };

/** 基本ベクトル3本の描き方．`letter`は，ラベルのTeXの文字(`a`や`b`)である． */
interface VectorLook {
  letter: string;
  style: JsonObject;
}

const REAL_LOOK: VectorLook = { letter: 'a', style: REAL_STYLE };
const RECIPROCAL_LOOK: VectorLook = { letter: 'b', style: RECIPROCAL_STYLE };

/** 原点`origin`から，3つの先端`tips`への基本ベクトルと，そのラベル． */
function basisVectors(
  origin: string,
  tips: readonly string[],
  { letter, style }: VectorLook,
): JsonObject[] {
  return tips.flatMap((tip, index): JsonObject[] => [
    { id: `${tip}_vector`, type: 'vector', from: origin, to: tip, style },
    {
      id: `${tip}_label`,
      type: 'label',
      at: [`${tip}_x`, `${tip}_y`, `${tip}_z`],
      anchor: 'south west',
      tex: String.raw`$\boldsymbol{${letter}}_${index + 1}$`,
    },
  ]);
}

/** 点`p`と`q`を位置ベクトルとみた外積p × qの成分の式(点の座標の名前`p_x`などで書く)． */
function cross(p: string, q: string): Triple {
  return [
    `${p}_y*${q}_z - ${p}_z*${q}_y`,
    `${p}_z*${q}_x - ${p}_x*${q}_z`,
    `${p}_x*${q}_y - ${p}_y*${q}_x`,
  ];
}

/** 単位胞の辺の両端．0は原点，1から3は基本ベクトルの先端，2つ以上の数字はその和の頂点である． */
const CELL_EDGES: readonly (readonly [string, string])[] = [
  ['0', '1'],
  ['0', '2'],
  ['0', '3'],
  ['1', '12'],
  ['1', '13'],
  ['2', '12'],
  ['2', '23'],
  ['3', '13'],
  ['3', '23'],
  ['12', '123'],
  ['13', '123'],
  ['23', '123'],
];

/**
 * 単位胞の12本の辺．`ids`は，原点と3つの先端の識別子である．残りの頂点は，点の式(先端の和から，
 * 原点を足しすぎた分を引く)で書く．
 */
function cellEdges(prefix: string, ids: readonly [string, string, string, string]): JsonObject[] {
  const [origin] = ids;
  const idOf = (name: string): string =>
    name.length === 1 ? (ids[Number(name)] ?? origin) : `${prefix}_${name}`;
  const corners = ['12', '13', '23', '123'].map((name) => {
    const tips = [...name].map((digit) => idOf(digit));
    return {
      id: `${prefix}_${name}`,
      type: 'point',
      at: `${tips.join(' + ')} - ${tips.length - 1}*${origin}`,
    };
  });
  const edges = CELL_EDGES.map(([from, to], index) => ({
    id: `${prefix}_edge${index + 1}`,
    type: 'segment',
    from: idOf(from),
    to: idOf(to),
    style: EDGE_STYLE,
  }));
  return [...corners, ...edges];
}

/** 三斜晶．逆格子の原点O*を，実格子の原点から画面の右へ`TRICLINIC_SHIFT`だけずらす． */
const TRICLINIC_SHIFT = '3.2';
const TRICLINIC_OFFSET = toRight(TRICLINIC_SHIFT);
const TRICLINIC_BASIS = generalBasis(['a', 'b', 'c'], ['angle_alpha', 'angle_beta', 'angle_gamma']);
const LENGTH_MIN = 0.8;
const LENGTH_MAX = 1.4;
const LENGTH_RANGE = [LENGTH_MIN, LENGTH_MAX] as const;
/** 角度の範囲．単位胞がつぶれて体積が0にならないよう，90度の近くに限る． */
const ANGLE_MIN = 75;
const ANGLE_MAX = 105;
const ANGLE_RANGE = [ANGLE_MIN, ANGLE_MAX] as const;
const A_LENGTH = 1;
const B_LENGTH = 1.1;
const C_LENGTH = 1.25;
const ALPHA = 80;
const BETA = 95;
const GAMMA = 85;
const TRIPLE_PRODUCT = ((): string => {
  const [x, y, z] = cross('A2', 'A3');
  return `A1_x*(${x}) + A1_y*(${y}) + A1_z*(${z})`;
})();

/** 逆格子の基本ベクトルの先端．b = (first × second) / Vを，O*から描く． */
function reciprocalTip(id: string, first: string, second: string): JsonObject {
  const [x, y, z] = cross(first, second);
  const [dx, dy] = TRICLINIC_OFFSET;
  return {
    id,
    type: 'point',
    at: [
      `${dx} + (${x})/(${TRIPLE_PRODUCT})`,
      `${dy} + (${y})/(${TRIPLE_PRODUCT})`,
      `(${z})/(${TRIPLE_PRODUCT})`,
    ],
  };
}

function originPoint(id: string, at: readonly (number | string)[], label: string): JsonObject {
  return { id, type: 'point', at: [...at], label, anchor: 'north east', dot: true };
}

const TRICLINIC_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '空間の逆格子(三斜晶)．実格子の基本ベクトルa_1，a_2，a_3(左)から，逆格子の基本ベクトル(右)をb_1 = (a_2 × a_3)/V，b_2 = (a_3 × a_1)/V，b_3 = (a_1 × a_2)/Vで作る．V = a_1・(a_2 × a_3)は単位胞の体積である．b_1はa_2とa_3の張る面に垂直で，a_i・b_j = δ_ijを満たす．格子定数をスライダーで動かすと，実格子の単位胞が薄くなる向きに，逆格子の単位胞が伸びることがわかる．',
  view: { azimuth: SPACE_AZIMUTH, elevation: SPACE_ELEVATION, unit: '1.5cm' },
  objects: [
    parameter('a', A_LENGTH, LENGTH_RANGE),
    parameter('b', B_LENGTH, LENGTH_RANGE),
    parameter('c', C_LENGTH, LENGTH_RANGE),
    parameter('angle_alpha', ALPHA, ANGLE_RANGE),
    parameter('angle_beta', BETA, ANGLE_RANGE),
    parameter('angle_gamma', GAMMA, ANGLE_RANGE),
    originPoint('O', [0, 0, 0], 'O'),
    ...['A1', 'A2', 'A3'].map((id, index) => ({
      id,
      type: 'point',
      at: [...(TRICLINIC_BASIS[index] ?? [])],
    })),
    ...cellEdges('real', ['O', 'A1', 'A2', 'A3']),
    ...basisVectors('O', ['A1', 'A2', 'A3'], REAL_LOOK),
    originPoint('O_star', [...TRICLINIC_OFFSET, 0], 'O^{*}'),
    reciprocalTip('B1', 'A2', 'A3'),
    reciprocalTip('B2', 'A3', 'A1'),
    reciprocalTip('B3', 'A1', 'A2'),
    ...cellEdges('reciprocal', ['O_star', 'B1', 'B2', 'B3']),
    ...basisVectors('O_star', ['B1', 'B2', 'B3'], RECIPROCAL_LOOK),
  ],
};

const RECIPROCAL_TRICLINIC_SAMPLE: SceneTemplate = {
  id: 'reciprocalTriclinic',
  label: '空間の逆格子(三斜晶)',
  scene: TRICLINIC_SCENE,
};

export {
  EDGE_STYLE,
  RECIPROCAL_LOOK,
  RECIPROCAL_TRICLINIC_SAMPLE,
  REAL_LOOK,
  SCREEN_RIGHT,
  SPACE_AZIMUTH,
  SPACE_ELEVATION,
  basisVectors,
  originPoint,
  toRight,
};
export type { Triple };
