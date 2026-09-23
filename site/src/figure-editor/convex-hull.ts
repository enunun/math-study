/** 空間の座標(x, y, z)． */
type Vertex3 = readonly [number, number, number];

function add(a: Vertex3, b: Vertex3): Vertex3 {
  const [ax, ay, az] = a;
  const [bx, by, bz] = b;
  return [ax + bx, ay + by, az + bz];
}

function subtract(a: Vertex3, b: Vertex3): Vertex3 {
  const [ax, ay, az] = a;
  const [bx, by, bz] = b;
  return [ax - bx, ay - by, az - bz];
}

function scaled(a: Vertex3, factor: number): Vertex3 {
  const [x, y, z] = a;
  return [x * factor, y * factor, z * factor];
}

function cross(a: Vertex3, b: Vertex3): Vertex3 {
  const [ax, ay, az] = a;
  const [bx, by, bz] = b;
  return [ay * bz - az * by, az * bx - ax * bz, ax * by - ay * bx];
}

function dot(a: Vertex3, b: Vertex3): number {
  const [ax, ay, az] = a;
  const [bx, by, bz] = b;
  return ax * bx + ay * by + az * bz;
}

function length(a: Vertex3): number {
  return Math.sqrt(dot(a, a));
}

function averageOf(points: readonly Vertex3[]): Vertex3 {
  let sum: Vertex3 = [0, 0, 0];
  for (const point of points) {
    sum = add(sum, point);
  }
  return scaled(sum, 1 / points.length);
}

function maxAbsCoordinate(points: readonly Vertex3[]): number {
  let max = 1;
  for (const point of points) {
    for (const coordinate of point) {
      max = Math.max(max, Math.abs(coordinate));
    }
  }
  return max;
}

/** 平行でない適当な向き．`normal`とほぼ平行なときは，別の軸を使う． */
const NON_PARALLEL_THRESHOLD = 0.9;

function arbitraryDirection(normal: Vertex3): Vertex3 {
  const [nx] = normal;
  return Math.abs(nx) < NON_PARALLEL_THRESHOLD ? [1, 0, 0] : [0, 1, 0];
}

/**
 * 面(頂点の番号)を，その面の中心から見て，外向きの法線のまわりに，反時計回りに並べ替える．
 * `normal`と直交する2つの単位ベクトル`u`，`v`(`u × v = normal`，右手系)を作り，各頂点の
 * `u`，`v`成分の偏角(`atan2`)で並べる．
 */
function orderFaceVertices(
  indices: readonly number[],
  vertices: readonly Vertex3[],
  normal: Vertex3,
): number[] {
  const points = indices.map((index) => vertices[index]);
  const center = averageOf(points);
  const rawU = cross(normal, arbitraryDirection(normal));
  const u = scaled(rawU, 1 / length(rawU));
  const v = cross(normal, u);
  const angleOf = (point: Vertex3): number => {
    const offset = subtract(point, center);
    return Math.atan2(dot(v, offset), dot(u, offset));
  };
  return indices
    .map((index, position) => ({ index, angle: angleOf(points[position]) }))
    .toSorted((a, b) => a.angle - b.angle)
    .map((entry) => entry.index);
}

/** すべての頂点の組(3個ずつ，昇順)． */
function allTriples(count: number): [number, number, number][] {
  const triples: [number, number, number][] = [];
  for (let i = 0; i < count; i += 1) {
    for (let j = i + 1; j < count; j += 1) {
      for (let k = j + 1; k < count; k += 1) {
        triples.push([i, j, k]);
      }
    }
  }
  return triples;
}

/** 凸包を作る平面ごとの，許容誤差の係数(頂点座標の大きさに対する割合)．丸め誤差だけを吸収する． */
const HULL_TOLERANCE_FACTOR = 1e-9;
const MIN_FACE_VERTICES = 3;

interface SupportingPlane {
  normal: Vertex3;
  onPlane: readonly number[];
}

/** 凸包を求めるときに共通で使う，頂点の重心と許容誤差． */
interface HullContext {
  center: Vertex3;
  tolerance: number;
}

/** 平面`dot(normal, x) = offset`の形の，向きつきの平面． */
interface Plane {
  normal: Vertex3;
  offset: number;
}

/** 平面に対する，頂点の分類．支持平面なら，その上にある頂点(面になる)を返す． */
function classifyAgainstPlane(
  vertices: readonly Vertex3[],
  plane: Plane,
  tolerance: number,
): readonly number[] | undefined {
  const signedDistance = (vertex: Vertex3): number => dot(plane.normal, vertex) - plane.offset;
  const isOutside = vertices.some((vertex) => signedDistance(vertex) > tolerance);
  if (isOutside) {
    return undefined;
  }
  return vertices
    .map((vertex, index) => ({ index, distance: signedDistance(vertex) }))
    .filter((entry) => Math.abs(entry.distance) <= tolerance)
    .map((entry) => entry.index);
}

/**
 * 3頂点(`corners`)を通る平面の，外向きの単位法線．3頂点が一直線に並ぶ(退化した)ときは`undefined`．
 * 「外向き」は，`context.center`(頂点全体の重心)から見て，最初の頂点より遠い側とする．
 */
function outwardNormalOf(
  corners: readonly [Vertex3, Vertex3, Vertex3],
  context: HullContext,
): Vertex3 | undefined {
  const [a, b, c] = corners;
  const raw = cross(subtract(b, a), subtract(c, a));
  const rawLength = length(raw);
  if (rawLength < context.tolerance) {
    return undefined;
  }
  const inward = scaled(raw, 1 / rawLength);
  return dot(inward, subtract(a, context.center)) >= 0 ? inward : scaled(inward, -1);
}

/**
 * 3頂点(`triple`)を通る平面が，凸包の支持平面か調べる．支持平面とは，ほかのすべての頂点が
 * その片側(か，その平面の上)にある平面である．支持平面なら，外向きの法線と，その平面の上に
 * ある頂点(面になる)を返す．3頂点が一直線に並ぶ(退化した)ときと，支持平面でないときは`undefined`．
 */
function supportingPlaneAt(
  vertices: readonly Vertex3[],
  triple: readonly [number, number, number],
  context: HullContext,
): SupportingPlane | undefined {
  const [i, j, k] = triple;
  const a = vertices[i];
  const normal = outwardNormalOf([a, vertices[j], vertices[k]], context);
  if (normal === undefined) {
    return undefined;
  }
  const plane: Plane = { normal, offset: dot(normal, a) };
  const onPlane = classifyAgainstPlane(vertices, plane, context.tolerance);
  return onPlane === undefined || onPlane.length < MIN_FACE_VERTICES
    ? undefined
    : { normal, onPlane };
}

/** 同じ頂点の集まりの面(向きの違う法線から，同じ平面が2回見つかることがある)を，1つにまとめる． */
function dedupeByVertexSet(planes: readonly SupportingPlane[]): SupportingPlane[] {
  const seen = new Set<string>();
  const unique: SupportingPlane[] = [];
  for (const plane of planes) {
    const key = plane.onPlane.toSorted((a, b) => a - b).join(',');
    if (!seen.has(key)) {
      seen.add(key);
      unique.push(plane);
    }
  }
  return unique;
}

/**
 * 頂点の集まり(すべて凸包の頂点，内部の点はない)から，凸包の面を求める．正多面体の稜は，
 * 手で書き出さず，この面から自動的に決める(隣り合う面が共有する辺として)．
 */
function convexHullFaces(vertices: readonly Vertex3[]): number[][] {
  const context: HullContext = {
    center: averageOf(vertices),
    tolerance: maxAbsCoordinate(vertices) * HULL_TOLERANCE_FACTOR,
  };
  const planes = allTriples(vertices.length)
    .map((triple) => supportingPlaneAt(vertices, triple, context))
    .filter((plane): plane is SupportingPlane => plane !== undefined);
  return dedupeByVertexSet(planes).map((plane) =>
    orderFaceVertices(plane.onPlane, vertices, plane.normal),
  );
}

export { convexHullFaces };
export type { Vertex3 };
