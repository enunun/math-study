import { convexHullFaces } from './convex-hull';
import type { Vertex3 } from './convex-hull';
import type { JsonObject } from './json';

/** 座標を書き出すときに丸める単位．10進で6桁にする． */
const DECIMAL_BASE = 10;
const ROUNDING_DECIMALS = 6;
const ROUNDING_UNIT = DECIMAL_BASE ** ROUNDING_DECIMALS;

function round(value: number): number {
  return Math.round(value * ROUNDING_UNIT) / ROUNDING_UNIT;
}

/** 正n角形の既定の外接円の半径． */
const DEFAULT_CIRCUMRADIUS = 2;
/** 1周(ラジアン)． */
const FULL_TURN_FACTOR = 2;
const FULL_TURN = FULL_TURN_FACTOR * Math.PI;

/**
 * 正n角形の頂点(点)と辺(線分)．中心を原点に置き，頂点の1つを右(角度0)に取る．
 * 識別子は`p1`.. と`s1`..で，挿入のときに重ならない識別子へ付け替える．平面の図は，稜を隠す・
 * 隠されるという概念がないので，複体ではなく，点と線分で書く．
 */
function regularPolygonObjects(n: number, radius: number = DEFAULT_CIRCUMRADIUS): JsonObject[] {
  const pointIds = Array.from({ length: n }, (_, index) => `p${index + 1}`);
  const points: JsonObject[] = pointIds.map((id, index) => {
    const angle = (FULL_TURN * index) / n;
    return {
      id,
      type: 'point',
      at: [round(radius * Math.cos(angle)), round(radius * Math.sin(angle))],
      dot: true,
    };
  });
  const segments: JsonObject[] = pointIds.map((id, index) => ({
    id: `s${index + 1}`,
    type: 'segment',
    from: id,
    to: pointIds[(index + 1) % n],
  }));
  return [...points, ...segments];
}

/**
 * 正多面体の複体(`complex`)．頂点の座標だけを与え，凸包から面を(したがって稜も)自動的に求める．
 * 1つのオブジェクトなので，挿入したあとの移動・削除・スタイル変更が1回の操作で済む．
 */
function polyhedronObjects(vertices: readonly Vertex3[]): JsonObject[] {
  return [
    {
      id: 'c',
      type: 'complex',
      vertices: vertices.map((vertex) => vertex.map((coordinate) => round(coordinate))),
      faces: convexHullFaces(vertices),
    },
  ];
}

/** 黄金比．正20面体・正12面体の頂点の座標に使う． */
const GOLDEN_RATIO_RADICAND = 5;
const GOLDEN_RATIO_DIVISOR = 2;
const PHI = (1 + Math.sqrt(GOLDEN_RATIO_RADICAND)) / GOLDEN_RATIO_DIVISOR;
const INV_PHI = 1 / PHI;

/** 正4面体の頂点．立方体の頂点を1つおきに取る． */
const TETRAHEDRON_VERTICES: readonly Vertex3[] = [
  [1, 1, 1],
  [1, -1, -1],
  [-1, 1, -1],
  [-1, -1, 1],
];

/** 立方体(正6面体)の頂点． */
const CUBE_VERTICES: readonly Vertex3[] = [
  [1, 1, 1],
  [1, 1, -1],
  [1, -1, 1],
  [1, -1, -1],
  [-1, 1, 1],
  [-1, 1, -1],
  [-1, -1, 1],
  [-1, -1, -1],
];

/** 正8面体の頂点．座標軸上の6点． */
const OCTAHEDRON_VERTICES: readonly Vertex3[] = [
  [1, 0, 0],
  [-1, 0, 0],
  [0, 1, 0],
  [0, -1, 0],
  [0, 0, 1],
  [0, 0, -1],
];

/** 正20面体の頂点．3枚の黄金長方形(1：φ)の頂点を合わせたもの． */
const ICOSAHEDRON_VERTICES: readonly Vertex3[] = [
  [0, 1, PHI],
  [0, 1, -PHI],
  [0, -1, PHI],
  [0, -1, -PHI],
  [1, PHI, 0],
  [1, -PHI, 0],
  [-1, PHI, 0],
  [-1, -PHI, 0],
  [PHI, 0, 1],
  [PHI, 0, -1],
  [-PHI, 0, 1],
  [-PHI, 0, -1],
];

/** 正12面体の頂点．立方体の8頂点と，3方向の黄金長方形の頂点を合わせたもの． */
const DODECAHEDRON_VERTICES: readonly Vertex3[] = [
  [1, 1, 1],
  [1, 1, -1],
  [1, -1, 1],
  [1, -1, -1],
  [-1, 1, 1],
  [-1, 1, -1],
  [-1, -1, 1],
  [-1, -1, -1],
  [0, INV_PHI, PHI],
  [0, INV_PHI, -PHI],
  [0, -INV_PHI, PHI],
  [0, -INV_PHI, -PHI],
  [INV_PHI, PHI, 0],
  [INV_PHI, -PHI, 0],
  [-INV_PHI, PHI, 0],
  [-INV_PHI, -PHI, 0],
  [PHI, 0, INV_PHI],
  [PHI, 0, -INV_PHI],
  [-PHI, 0, INV_PHI],
  [-PHI, 0, -INV_PHI],
];

export {
  CUBE_VERTICES,
  DODECAHEDRON_VERTICES,
  ICOSAHEDRON_VERTICES,
  OCTAHEDRON_VERTICES,
  TETRAHEDRON_VERTICES,
  polyhedronObjects,
  regularPolygonObjects,
};
export type { Vertex3 } from './convex-hull';
