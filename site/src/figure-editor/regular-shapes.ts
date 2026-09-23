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
 * 識別子は`p1`.. と`s1`..で，挿入のときに重ならない識別子へ付け替える．
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

/** 空間の座標(x, y, z)． */
type Vertex3 = readonly [number, number, number];

const SQUARE_EXPONENT = 2;

function distanceSquared(a: Vertex3, b: Vertex3): number {
  const [ax, ay, az] = a;
  const [bx, by, bz] = b;
  return (ax - bx) ** SQUARE_EXPONENT + (ay - by) ** SQUARE_EXPONENT + (az - bz) ** SQUARE_EXPONENT;
}

/** 2つの頂点の添字と，その間の距離の2乗． */
interface VertexPair {
  i: number;
  j: number;
  distanceSquared: number;
}

/** すべての頂点の組の，頂点間の距離の2乗． */
function allPairDistances(vertices: readonly Vertex3[]): readonly VertexPair[] {
  const pairs: VertexPair[] = [];
  for (let i = 0; i < vertices.length; i += 1) {
    for (let j = i + 1; j < vertices.length; j += 1) {
      pairs.push({ i, j, distanceSquared: distanceSquared(vertices[i], vertices[j]) });
    }
  }
  return pairs;
}

/** 最も近い頂点どうしの距離の2乗と，同じとみなす許容誤差(丸め誤差だけを吸収する，ごく小さい値)． */
const EDGE_TOLERANCE_FACTOR = 1e-6;

/**
 * 最も近い頂点どうしを稜で結ぶ．正多面体は，どの頂点も隣接する頂点まで同じ距離なので，
 * 最小の頂点間距離になる組がちょうど稜になる．
 */
function nearestNeighborEdges(
  vertices: readonly Vertex3[],
): readonly (readonly [number, number])[] {
  const pairs = allPairDistances(vertices);
  const minDistance = Math.min(...pairs.map((pair) => pair.distanceSquared));
  const tolerance = minDistance * EDGE_TOLERANCE_FACTOR;
  return pairs
    .filter((pair) => Math.abs(pair.distanceSquared - minDistance) < tolerance)
    .map((pair): [number, number] => [pair.i, pair.j]);
}

/** 正多面体の頂点(点)と稜(線分)．頂点の座標だけを与え，稜は最近接の組から決める． */
function polyhedronObjects(vertices: readonly Vertex3[]): JsonObject[] {
  const pointIds = vertices.map((_, index) => `p${index + 1}`);
  const points: JsonObject[] = vertices.map((vertex, index) => ({
    id: pointIds[index],
    type: 'point',
    at: vertex.map((coordinate) => round(coordinate)),
    dot: true,
  }));
  const segments: JsonObject[] = nearestNeighborEdges(vertices).map(([from, to], index) => ({
    id: `s${index + 1}`,
    type: 'segment',
    from: pointIds[from],
    to: pointIds[to],
  }));
  return [...points, ...segments];
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
  nearestNeighborEdges,
  polyhedronObjects,
  regularPolygonObjects,
};
export type { Vertex3 };
