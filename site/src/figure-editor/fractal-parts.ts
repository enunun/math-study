import type { Json, JsonObject } from './json';

/**
 * 相似変換の手順．`ratio`倍に縮め，`angle`度回し，`to`へ平行移動する．縮める・回す・動かすのうち，
 * 何もしないものは書かない．フラクタルの各変換は，この形で書く．
 */
function similarity(ratio: Json, angle: number, to: readonly [Json, Json]): JsonObject[] {
  const steps: JsonObject[] = [{ scale: ratio }];
  if (angle !== 0) {
    steps.push({ rotate: angle });
  }
  if (to[0] !== 0 || to[1] !== 0) {
    steps.push({ translate: [...to] });
  }
  return steps;
}

/** 変換に使う，正確な値の式． */
const THIRD = '1/3';
const HALF = '1/2';
const INV_SQRT2 = '1/sqrt(2)';

/** 長さ1の線分．曲線のフラクタルの基本図形． */
const UNIT_SEGMENT: Json = [
  [0, 0],
  [1, 0],
];
/** 1辺1の正方形．面のフラクタルの基本図形(`closed`で閉じる)． */
const UNIT_SQUARE: Json = [
  [0, 0],
  [1, 0],
  [1, 1],
  [0, 1],
];
/** 1辺1の正三角形． */
const UNIT_TRIANGLE: Json = [
  [0, 0],
  [1, 0],
  ['1/2', 'sqrt(3)/2'],
];

/** 数か式の符号を変える．式は，括弧でくくって`-`を付ける． */
function negate(value: number | string): number | string {
  return typeof value === 'number' ? -value : `-(${value})`;
}

/**
 * 1辺1で作った図形を，`center`が原点に来るように動かしてから，原点のまわりに`size`倍にする変換．
 * フラクタルの定義(`transforms`)は正確な分数のまま，大きさは`scale`の1か所で決まる．
 */
function placement(
  size: number,
  center: readonly [number | string, number | string],
): JsonObject[] {
  return [{ translate: [negate(center[0]), negate(center[1])] }, { scale: size }];
}

interface FractalSpec {
  base: Json;
  closed?: boolean;
  transforms: readonly JsonObject[][];
  depth: number;
  transform: readonly JsonObject[];
  allDepths?: boolean;
}

function fractal(spec: FractalSpec): JsonObject {
  return {
    id: 'fractal',
    type: 'fractal',
    base: spec.base,
    ...(spec.closed === true ? { closed: true } : {}),
    transforms: spec.transforms.map((steps) => [...steps]),
    depth: spec.depth,
    ...(spec.allDepths === true ? { all_depths: true } : {}),
    transform: [...spec.transform],
  };
}

export {
  fractal,
  HALF,
  INV_SQRT2,
  placement,
  similarity,
  THIRD,
  UNIT_SEGMENT,
  UNIT_SQUARE,
  UNIT_TRIANGLE,
};
