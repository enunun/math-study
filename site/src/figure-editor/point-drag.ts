import type { SceneDraft } from './draft';
import { arrayOf, objectOf, stringOf } from './json';
import type { JsonObject } from './json';

/**
 * 編集中の図で，点をドラッグして動かすための計算．動かせるのは，平面の図の点のうち，位置を2つの数で書き，
 * 変換を持たない点である．式で決まる点(点の式や，媒介変数の式)は動かさず，描き直すと，動かした点に従う．
 */

/** 動かせる点．識別子と，数学の座標． */
interface DraggablePoint {
  id: string;
  at: readonly [number, number];
}

const CM_PER_INCH = 2.54;
const PT_PER_INCH = 72.27;
const MM_PER_CM = 10;
/** 書き込む座標の細かさ．見える範囲の幅をこの数で割った刻みに丸める． */
const POSITION_DIVISIONS = 1000;
/** 丸めた値の有効数字．浮動小数点の誤差(0.30000000000000004など)を消す． */
const POSITION_PRECISION = 12;
/** 平面の点の座標の数． */
const PLANE_DIMENSION = 2;

/** 単位つきの長さ(`"1cm"`，`"5mm"`，`"28.45pt"`)を，cmにする．読めなければ`undefined`． */
function lengthToCm(text: string): number | undefined {
  const groups = /^(?<value>\d+\.\d+|\d+|\.\d+)(?<unit>cm|mm|pt)$/u.exec(text)?.groups;
  if (groups === undefined) {
    return undefined;
  }
  const value = Number(groups.value);
  switch (groups.unit) {
    case 'mm': {
      return value / MM_PER_CM;
    }
    case 'pt': {
      return (value / PT_PER_INCH) * CM_PER_INCH;
    }
    default: {
      return value;
    }
  }
}

/** 平面の図の，数学の座標の1あたりのcm．平面の図でなければ`undefined`． */
function planeUnits(draft: SceneDraft): readonly [number, number] | undefined {
  const unit = objectOf(draft.view, 'unit');
  const x = lengthToCm(stringOf(unit, 'x'));
  const y = lengthToCm(stringOf(unit, 'y'));
  return x === undefined || y === undefined ? undefined : [x, y];
}

function isPlainPoint(object: JsonObject): boolean {
  const { at, transform } = object;
  return (
    object.type === 'point' &&
    Array.isArray(at) &&
    at.length === PLANE_DIMENSION &&
    at.every((value) => typeof value === 'number' && Number.isFinite(value)) &&
    (transform === undefined || (Array.isArray(transform) && transform.length === 0))
  );
}

/** 動かせる点(並びの順)． */
function draggablePoints(draft: SceneDraft): DraggablePoint[] {
  if (planeUnits(draft) === undefined) {
    return [];
  }
  return draft.objects.filter(isPlainPoint).map((object) => {
    const [x = 0, y = 0] = arrayOf(object, 'at').filter((value) => typeof value === 'number');
    return { id: stringOf(object, 'id'), at: [x, y] };
  });
}

/**
 * 図のSVGの座標(cm．SVGはyが下向きなので，yの符号は数学と逆)を，数学の座標にする．平面の図でなければ
 * `undefined`．
 */
function mathFromSvg(
  draft: SceneDraft,
  [x, y]: readonly [number, number],
): [number, number] | undefined {
  const units = planeUnits(draft);
  return units === undefined ? undefined : [x / units[0], -y / units[1]];
}

/** 数学の座標を，図のSVGの座標(cm)にする． */
function svgFromMath(
  draft: SceneDraft,
  [x, y]: readonly [number, number],
): [number, number] | undefined {
  const units = planeUnits(draft);
  return units === undefined ? undefined : [x * units[0], -y * units[1]];
}

/** SVGの座標`target`から`radius`(cm)の中で，いちばん近い動かせる点．なければ`undefined`． */
function pointNear(
  draft: SceneDraft,
  target: readonly [number, number],
  radius: number,
): DraggablePoint | undefined {
  const distance = (point: DraggablePoint): number => {
    const at = svgFromMath(draft, point.at);
    return at === undefined ? Infinity : Math.hypot(at[0] - target[0], at[1] - target[1]);
  };
  return draggablePoints(draft)
    .filter((point) => distance(point) <= radius)
    .toSorted((first, second) => distance(first) - distance(second))[0];
}

/** 見える範囲の幅．丸めの刻みに使う．読めなければ1． */
function viewWidth(draft: SceneDraft): number {
  const [low, high] = arrayOf(draft.view, 'x');
  return typeof low === 'number' && typeof high === 'number' && high > low ? high - low : 1;
}

/** 点`id`の位置を`at`(数学の座標)にした図．座標は，見える範囲の幅の`POSITION_DIVISIONS`分の1に丸める． */
function withPointAt(draft: SceneDraft, id: string, at: readonly [number, number]): SceneDraft {
  const step = viewWidth(draft) / POSITION_DIVISIONS;
  const rounded = at.map((value) =>
    Number((Math.round(value / step) * step).toPrecision(POSITION_PRECISION)),
  );
  const objects: JsonObject[] = [];
  for (const object of draft.objects) {
    objects.push(object.id === id && isPlainPoint(object) ? { ...object, at: rounded } : object);
  }
  return { ...draft, objects };
}

export { draggablePoints, lengthToCm, mathFromSvg, pointNear, svgFromMath, withPointAt };
export type { DraggablePoint };
