/** 空間のベクトル． */
type Vector3 = readonly [number, number, number];

/** カメラの基底．`d`はカメラへ向かう向き，`r`は画面の右，`u`は画面の上である． */
interface Camera {
  d: Vector3;
  r: Vector3;
  u: Vector3;
}

/** 点の画面の位置(`x`，`y`)と，奥行き．奥行きは，カメラに近いほど大きい． */
interface ScreenPoint {
  x: number;
  y: number;
  depth: number;
}

/** 度をラジアンにする倍率． */
/** 半周の角度(度)． */
const HALF_TURN_DEGREES = 180;
const RADIANS_PER_DEGREE = Math.PI / HALF_TURN_DEGREES;

function dot(a: Vector3, b: Vector3): number {
  const [ax, ay, az] = a;
  const [bx, by, bz] = b;
  return ax * bx + ay * by + az * bz;
}

/**
 * 方位角と仰角(度)から，カメラの基底を作る．図のエンジン(`crates/figure/src/space.rs`)の平行投影と同じである．
 * `d = (cos e cos a, cos e sin a, sin e)`，`r = (-sin a, cos a, 0)`，`u = d × r`である．
 */
function camera(azimuthDegrees: number, elevationDegrees: number): Camera {
  const a = azimuthDegrees * RADIANS_PER_DEGREE;
  const e = elevationDegrees * RADIANS_PER_DEGREE;
  return {
    d: [Math.cos(e) * Math.cos(a), Math.cos(e) * Math.sin(a), Math.sin(e)],
    r: [-Math.sin(a), Math.cos(a), 0],
    u: [-Math.sin(e) * Math.cos(a), -Math.sin(e) * Math.sin(a), Math.cos(e)],
  };
}

/** 点の画面の位置と奥行き． */
function screenOf({ d, r, u }: Camera, point: Vector3): ScreenPoint {
  return { x: dot(point, r), y: dot(point, u), depth: dot(point, d) };
}

export { camera, dot, screenOf };
export type { Camera, ScreenPoint, Vector3 };
