import { cosOf, sinOf } from './bravais-cells';
import type { JsonObject } from './json';

/**
 * 逆格子の見本の部品．実格子の基本ベクトルa_iから，a_i・b_j = δ_ij(2πを付けない流儀)を満たす逆格子の
 * 基本ベクトルb_jを作る．格子の線は，格子の座標(u, v)から平面への写像を持つ等値線で描くので，
 * u = n，v = nの線が，そのまま格子の線になる．
 */

/** 2つの座標の式．平面の点と，平面のベクトルの成分に使う． */
type Pair = readonly [string, string];
/** 平面の2つの基本ベクトル． */
type PlaneBasis = readonly [Pair, Pair];

/** 平面の格子．原点のx座標(y座標は0)と，基本ベクトルの組． */
interface Lattice {
  origin: number;
  basis: PlaneBasis;
}

const GAMMA = 'angle_gamma';
const COS = cosOf(GAMMA);
const SIN = sinOf(GAMMA);

/** 実格子の基本ベクトルa_1，a_2(a_1はx軸の向き，a_2はa_1と角γをなす)． */
const REAL_BASIS: PlaneBasis = [
  ['a', '0'],
  [`b*${COS}`, `b*${SIN}`],
];
/** 逆格子の基本ベクトルb_1 ⊥ a_2，b_2 ⊥ a_1．長さは1/(a sin γ)と1/(b sin γ)である． */
const RECIPROCAL_BASIS: PlaneBasis = [
  ['1/a', `-${COS}/(a*${SIN})`],
  ['0', `1/(b*${SIN})`],
];

/** 平面の図の，実格子(左)と逆格子(右)の原点のx座標． */
const REAL_X = -3.8;
const RECIPROCAL_X = 3.8;
const REAL: Lattice = { origin: REAL_X, basis: REAL_BASIS };
const RECIPROCAL: Lattice = { origin: RECIPROCAL_X, basis: RECIPROCAL_BASIS };

/** 格子の線を引く，格子の座標の範囲．-1，0，1の線が，少しはみ出して交わる． */
const PATCH_END = 1.4;
const INDICES = [-1, 0, 1];

const LATTICE_STYLE: JsonObject = { color: 'gray' };
const REAL_STYLE: JsonObject = { color: 'blue', width: '1pt' };
const RECIPROCAL_STYLE: JsonObject = { color: 'red', width: '1pt' };
const G_STYLE: JsonObject = { color: 'green', width: '1pt' };
const FAMILY_STYLE: JsonObject = { color: 'blue' };

const A_NAMES = [String.raw`\boldsymbol{a}_1`, String.raw`\boldsymbol{a}_2`] as const;
const B_NAMES = [String.raw`\boldsymbol{b}_1`, String.raw`\boldsymbol{b}_2`] as const;
/** 基本ベクトルのラベルの位置．a_1は右向き，a_2は上向き，b_1は右下向き，b_2は上向きなので，先端の外側に置く． */
const A_ANCHORS = ['north', 'south west'] as const;
const B_ANCHORS = ['north west', 'south west'] as const;

/** 数を式の中に書く形(負の数は括弧に入れる)． */
function numberText(value: number): string {
  return value < 0 ? `(${value})` : String(value);
}

/** 添字を識別子に使う形(負の数は`m`で書く)． */
function indexName(value: number): string {
  return value < 0 ? `m${-value}` : String(value);
}

function integersBetween(low: number, high: number): number[] {
  return Array.from({ length: high - low + 1 }, (_, index) => low + index);
}

/** 格子の座標(u, v)に当たる点の式．u，vは式でよい． */
function latticeAt({ origin, basis }: Lattice, [u, v]: Pair): Pair {
  const [[x1, y1], [x2, y2]] = basis;
  return [`${origin} + (${u})*(${x1}) + (${v})*(${x2})`, `(${u})*(${y1}) + (${v})*(${y2})`];
}

interface LinesSpec {
  id: string;
  lattice: Lattice;
  /** 格子の座標(u, v)の関数．線は，これが`values`の各値になる所である． */
  level: string;
  values: readonly number[];
  style: JsonObject;
  /** 格子の座標の範囲(既定は`PATCH_END`)． */
  patch?: number;
}

/** 格子の線． */
function latticeLines({
  id,
  lattice,
  level,
  values,
  style,
  patch = PATCH_END,
}: LinesSpec): JsonObject {
  return {
    id,
    type: 'level_curve',
    vars: ['u', 'v'],
    expr: [...latticeAt(lattice, ['u', 'v'])],
    domain: [
      [-patch, patch],
      [-patch, patch],
    ],
    level,
    values: [...values],
    style,
  };
}

/** u = n，v = nの線(格子そのもの)． */
function latticeGrid(prefix: string, lattice: Lattice): JsonObject[] {
  return ['u', 'v'].map((level) =>
    latticeLines({
      id: `${prefix}_${level}`,
      lattice,
      level,
      values: INDICES,
      style: LATTICE_STYLE,
    }),
  );
}

/** 格子点の印．`indices`の組(u, v)の全部に付ける． */
function latticePoints(prefix: string, lattice: Lattice, indices: readonly number[]): JsonObject[] {
  return indices.flatMap((u) =>
    indices.map((v) => ({
      id: `${prefix}_${indexName(u)}_${indexName(v)}`,
      type: 'point',
      at: [...latticeAt(lattice, [numberText(u), numberText(v)])],
      dot: true,
      style: LATTICE_STYLE,
    })),
  );
}

interface VectorsSpec {
  /** 原点と，2つの基本ベクトルの先端の識別子． */
  ids: readonly [string, string, string];
  lattice: Lattice;
  /** ベクトルのTeXの名前． */
  names: readonly [string, string];
  style: JsonObject;
  anchors: readonly [string, string];
}

/** 2つの基本ベクトルの先端と，基本ベクトルと，先端のラベル． */
function basisVectors({ ids, lattice, names, style, anchors }: VectorsSpec): JsonObject[] {
  const [originId, ...tips] = ids;
  const units: readonly Pair[] = [
    ['1', '0'],
    ['0', '1'],
  ];
  return tips.flatMap((tip, index): JsonObject[] => [
    { id: tip, type: 'point', at: [...latticeAt(lattice, units[index] ?? ['0', '0'])] },
    { id: `${tip}_vector`, type: 'vector', from: originId, to: tip, style },
    {
      id: `${tip}_label`,
      type: 'label',
      at: tip,
      anchor: anchors[index],
      tex: `$${names[index] ?? ''}$`,
    },
  ]);
}

/** 原点．ラベルは，基本ベクトルと重ならない左下に置く． */
function originPoint(id: string, { origin }: Lattice, label: string): JsonObject {
  return { id, type: 'point', at: [origin, 0], label, anchor: 'north east', dot: true };
}

function parameter(id: string, value: number, range: readonly [number, number]): JsonObject {
  return { id, type: 'parameter', value, range: [...range] };
}

/** 整数の値だけを取る媒介変数(指数)． */
function indexParameter(id: string, value: number, range: readonly [number, number]): JsonObject {
  return { ...parameter(id, value, range), step: 1 };
}

export {
  A_ANCHORS,
  A_NAMES,
  B_ANCHORS,
  B_NAMES,
  COS,
  FAMILY_STYLE,
  GAMMA,
  G_STYLE,
  INDICES,
  LATTICE_STYLE,
  PATCH_END,
  REAL,
  RECIPROCAL,
  RECIPROCAL_BASIS,
  RECIPROCAL_STYLE,
  REAL_STYLE,
  SIN,
  basisVectors,
  indexName,
  indexParameter,
  integersBetween,
  latticeAt,
  latticeGrid,
  latticeLines,
  latticePoints,
  numberText,
  originPoint,
  parameter,
};
export type { Lattice, Pair, PlaneBasis };
