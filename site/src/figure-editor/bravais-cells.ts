import type { JsonObject } from './json';

/** 格子定数の媒介変数．長さは`LENGTH_RANGE`，角度は種類ごとの範囲で動かせる． */
interface LatticeConstant {
  id: string;
  value: number;
  range: readonly [number, number];
}

/** 長さの範囲と，角度の範囲(一般の角度と，菱面体の角度)． */
const LENGTH_MIN = 1;
const LENGTH_MAX = 4;
const LENGTH_RANGE = [LENGTH_MIN, LENGTH_MAX] as const;
const ANGLE_MIN = 60;
const ANGLE_MAX = 120;
const ANGLE_RANGE = [ANGLE_MIN, ANGLE_MAX] as const;
const RHOMBOHEDRAL_ANGLE_MIN = 40;
const RHOMBOHEDRAL_ANGLE_MAX = 110;

function length(id: string, value: number): LatticeConstant {
  return { id, value, range: LENGTH_RANGE };
}

function angle(
  id: string,
  value: number,
  range: readonly [number, number] = ANGLE_RANGE,
): LatticeConstant {
  return { id, value, range };
}

/** 度で書いた角度の媒介変数の，余弦と正弦の式． */
function cosOf(id: string): string {
  return `cos(${id}*pi/180)`;
}

function sinOf(id: string): string {
  return `sin(${id}*pi/180)`;
}

/** 格子ベクトルの座標の式．平面では2つ，空間では3つのベクトルで，各座標は式である． */
type Basis = readonly (readonly string[])[];

/**
 * 一般の格子ベクトル．aはx軸の向き，bはxy平面の上でaと角γをなし，cはaと角β，bと角αをなす．
 * 長さと角度には，媒介変数の名前か，式を渡す．
 */
function generalBasis(
  lengths: readonly [string, string, string],
  angles: readonly [string, string, string],
): Basis {
  const [a, b, c] = lengths;
  const [alpha, beta, gamma] = angles;
  const cy = `(${cosOf(alpha)} - ${cosOf(beta)}*${cosOf(gamma)})/${sinOf(gamma)}`;
  return [
    [a, '0', '0'],
    [`${b}*${cosOf(gamma)}`, `${b}*${sinOf(gamma)}`, '0'],
    [`${c}*${cosOf(beta)}`, `${c}*${cy}`, `${c}*sqrt(1 - ${cosOf(beta)}^2 - (${cy})^2)`],
  ];
}

/** 3つの辺が直交する格子ベクトル(直方晶，正方晶，立方晶)． */
function orthogonalBasis(a: string, b: string, c: string): Basis {
  return [
    [a, '0', '0'],
    ['0', b, '0'],
    ['0', '0', c],
  ];
}

/** 心の点の置き方．単純(P)，体心(I)，面心(F)，底心(S，ab面の中心)． */
type Centering = 'P' | 'I' | 'F' | 'S';

/** 格子点の印．単位胞の頂点の格子点と，心の格子点は，色を分ける． */
const CORNER_STYLE: JsonObject = { color: 'blue' };
const CENTER_STYLE: JsonObject = { color: 'red' };

function latticePoint(id: string, at: JsonObject['at'], style: JsonObject): JsonObject {
  return { id, type: 'point', at, dot: true, style };
}

/** 空間の単位胞の頂点(原点，A，B，Cと，その和)．Oから数えて，各頂点の識別子と点の式． */
const SPACE_CORNERS: readonly (readonly [string, string])[] = [
  ['AB', 'A + B'],
  ['AC', 'A + C'],
  ['BC', 'B + C'],
  ['ABC', 'A + B + C'],
];

/** 空間の単位胞の12本の辺(両端の頂点の識別子)． */
const SPACE_EDGES: readonly (readonly [string, string])[] = [
  ['O', 'A'],
  ['O', 'B'],
  ['O', 'C'],
  ['A', 'AB'],
  ['A', 'AC'],
  ['B', 'AB'],
  ['B', 'BC'],
  ['C', 'AC'],
  ['C', 'BC'],
  ['AB', 'ABC'],
  ['AC', 'ABC'],
  ['BC', 'ABC'],
];

/** 心の格子点の点の式．面心は6つの面の中心，底心はab面(上下の2面)の中心である． */
const CENTERS: Readonly<Record<Centering, readonly string[]>> = {
  P: [],
  I: ['(A + B + C) / 2'],
  F: [
    '(A + B) / 2',
    '(A + B) / 2 + C',
    '(A + C) / 2',
    '(A + C) / 2 + B',
    '(B + C) / 2',
    '(B + C) / 2 + A',
  ],
  S: ['(A + B) / 2', '(A + B) / 2 + C'],
};

function parameterObjects(constants: readonly LatticeConstant[]): JsonObject[] {
  return constants.map(({ id, value, range }) => ({
    id,
    type: 'parameter',
    value,
    range: [...range],
  }));
}

function edgeObjects(edges: readonly (readonly [string, string])[]): JsonObject[] {
  return edges.map(([from, to], index) => ({
    id: `edge${index + 1}`,
    type: 'segment',
    from,
    to,
  }));
}

/** 空間のブラベー格子の単位胞．媒介変数，格子ベクトルの先端，残りの頂点，心の点，辺の順に並べる． */
function spaceCell(
  constants: readonly LatticeConstant[],
  basis: Basis,
  centering: Centering,
): JsonObject[] {
  const [a = [], b = [], c = []] = basis;
  return [
    ...parameterObjects(constants),
    latticePoint('O', [0, 0, 0], CORNER_STYLE),
    latticePoint('A', [...a], CORNER_STYLE),
    latticePoint('B', [...b], CORNER_STYLE),
    latticePoint('C', [...c], CORNER_STYLE),
    ...SPACE_CORNERS.map(([id, at]) => latticePoint(id, at, CORNER_STYLE)),
    ...CENTERS[centering].map((at, index) => latticePoint(`center${index + 1}`, at, CENTER_STYLE)),
    ...edgeObjects(SPACE_EDGES),
  ];
}

/** 平面の単位胞の4本の辺． */
const PLANE_EDGES: readonly (readonly [string, string])[] = [
  ['O', 'A'],
  ['O', 'B'],
  ['A', 'AB'],
  ['B', 'AB'],
];

/** 平面のブラベー格子の単位胞．`centered`なら，胞の中心にも格子点がある． */
function planeCell(
  constants: readonly LatticeConstant[],
  basis: Basis,
  centered: boolean,
): JsonObject[] {
  const [a = [], b = []] = basis;
  return [
    ...parameterObjects(constants),
    latticePoint('O', [0, 0], CORNER_STYLE),
    latticePoint('A', [...a], CORNER_STYLE),
    latticePoint('B', [...b], CORNER_STYLE),
    latticePoint('AB', 'A + B', CORNER_STYLE),
    ...(centered ? [latticePoint('center', '(A + B) / 2', CENTER_STYLE)] : []),
    ...edgeObjects(PLANE_EDGES),
  ];
}

export {
  RHOMBOHEDRAL_ANGLE_MAX,
  RHOMBOHEDRAL_ANGLE_MIN,
  angle,
  cosOf,
  generalBasis,
  length,
  orthogonalBasis,
  planeCell,
  sinOf,
  spaceCell,
};
export type { Basis, Centering, LatticeConstant };
