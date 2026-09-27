import type { JsonObject } from './json';

/** 空間の図の見る向き．既定(方位角60度，仰角20度)より，少し上から見る． */
const SPACE_AZIMUTH = 60;
const SPACE_ELEVATION = 30;
const SPACE_UNIT = '1cm';

/** 空間の図の座標軸と，原点Oの名前．軸は，`range`の範囲に引く． */
function spaceAxes(
  horizontal: readonly [number, number],
  vertical: readonly [number, number],
): JsonObject[] {
  return [
    { id: 'x_axis', type: 'axis', direction: 'x', range: [...horizontal], label: 'x' },
    { id: 'y_axis', type: 'axis', direction: 'y', range: [...horizontal], label: 'y' },
    { id: 'z_axis', type: 'axis', direction: 'z', range: [...vertical], label: 'z' },
    { id: 'origin_label', type: 'label', at: [0, 0, 0], anchor: 'north east', tex: '$O$' },
  ];
}

/** 丸めた値の有効数字．0.6000000000000001のような浮動小数点の誤差を消す． */
const PRECISION = 12;

/** `step`の1倍から`count`倍まで． */
function multiples(step: number, count: number): number[] {
  return Array.from({ length: count }, (_, k) => Number(((k + 1) * step).toPrecision(PRECISION)));
}

/** `step`の，-`count`倍から`count`倍まで(0を含む)． */
function symmetricMultiples(step: number, count: number): number[] {
  const positive = multiples(step, count);
  return [...positive.map((value) => -value).toReversed(), 0, ...positive];
}

export { SPACE_AZIMUTH, SPACE_ELEVATION, SPACE_UNIT, multiples, spaceAxes, symmetricMultiples };
