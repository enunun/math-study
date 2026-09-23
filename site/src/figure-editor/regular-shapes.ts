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

export { regularPolygonObjects };
