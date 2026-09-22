import { viewKind } from './draft';
import type { SceneDraft } from './draft';
import { arrayOf, stringOf, withField } from './json';
import type { Json, JsonObject } from './json';

/**
 * 編集の補助として示す，格子と座標軸の見た目(灰色の点線)．実際の出力(プレビューやTikZ)は，
 * 元のオブジェクトのスタイルのままで，この見た目は，編集画面だけに使う．
 */
const REFERENCE_STYLE: JsonObject = { line: 'dotted', color: 'gray', width: '0.4pt' };

const PAIR = 2;
/** 補助の目盛の，だいたいの本数．軸の範囲を，これくらいの数に分ける，きりのいい間隔を選ぶ． */
const TICK_TARGET_COUNT = 6;
const STEP_1 = 1;
const STEP_2 = 2;
const STEP_5 = 5;
const STEP_10 = 10;
/** きりのいい間隔として選ぶ，1桁の数．小さい方から，この間隔で範囲を分けたときの本数と比べる． */
const NICE_STEPS = [STEP_1, STEP_2, STEP_5, STEP_10];
/** 浮動小数の丸め誤差を吸収する，範囲に対する許容の割合． */
const FLOAT_TOLERANCE = 1e-9;

/** 格子と座標軸を，補助のスタイルで描く． */
function withReferenceStyle(object: JsonObject): JsonObject {
  return withField(object, 'style', REFERENCE_STYLE);
}

/** 2つの数の配列なら，その組．そうでなければ`undefined`． */
function numericPair(value: Json | undefined): [number, number] | undefined {
  if (!Array.isArray(value) || value.length !== PAIR) {
    return undefined;
  }
  const [low, high] = value;
  return typeof low === 'number' && typeof high === 'number' ? [low, high] : undefined;
}

/** 範囲をだいたい`TICK_TARGET_COUNT`本に分ける，1・2・5のきりのいい間隔． */
function niceStep(range: number): number {
  const magnitude = STEP_10 ** Math.floor(Math.log10(range / TICK_TARGET_COUNT));
  const step = NICE_STEPS.find((candidate) => range / (candidate * magnitude) <= TICK_TARGET_COUNT);
  return (step ?? NICE_STEPS.at(-1) ?? STEP_1) * magnitude;
}

/** `step`の倍数の中で，`value`に一番近いもの(浮動小数の誤差を丸める)． */
function roundToStep(value: number, step: number): number {
  return Math.round(value / step) * step;
}

/** きりのいい間隔で，範囲`[low, high]`に収まる位置． */
function niceTicks(low: number, high: number): number[] {
  if (!(high > low) || !Number.isFinite(low) || !Number.isFinite(high)) {
    return [];
  }
  const step = niceStep(high - low);
  const tolerance = step * FLOAT_TOLERANCE;
  const values: number[] = [];
  for (
    let n = Math.ceil(low / step - FLOAT_TOLERANCE);
    roundToStep(n * step, step) <= high + tolerance;
    n += 1
  ) {
    values.push(roundToStep(n * step, step));
  }
  return values;
}

/** 座標軸に，読みやすいように，補助の目盛を足す．すでに目盛があれば，そのままにする． */
function withReferenceTicks(axis: JsonObject, view: JsonObject): JsonObject {
  if (arrayOf(axis, 'ticks').length > 0) {
    return axis;
  }
  const direction = stringOf(axis, 'direction');
  const range = numericPair(axis.range) ?? numericPair(view[direction]);
  if (range === undefined) {
    return axis;
  }
  const ticks: Json[] = niceTicks(...range).map((value) => ({ at: value, label: String(value) }));
  return ticks.length === 0 ? axis : withField(axis, 'ticks', ticks);
}

/** オブジェクト1つに，編集の補助を加える．座標軸と格子だけを変え，ほかはそのままにする． */
function editObject(object: JsonObject, plane: boolean, view: JsonObject): JsonObject {
  const type = stringOf(object, 'type');
  if (type === 'axis') {
    const styled = withReferenceStyle(object);
    return plane ? withReferenceTicks(styled, view) : styled;
  }
  return type === 'grid' ? withReferenceStyle(object) : object;
}

/**
 * プレビューを，編集の補助を加えた図にする．格子と座標軸は，補助のスタイル(灰色の点線)で描き，
 * 実際に描かれるオブジェクトと見分けやすくする．平面の図の座標軸には，読みやすいように，目盛も足す
 * (空間の図の軸には，まだ目盛を付けられない)．どちらも，見た目だけを変える．実際の出力(プレビューや
 * TikZ)は，元のシーンのままである．曲面のワイヤーフレームは，編集の補助ではなく，シーン自身の項目
 * (`wireframe`，`control_net`)で持つので，ここでは扱わない．
 */
function buildEditScene(draft: SceneDraft): SceneDraft {
  const plane = viewKind(draft) === 'plane';
  const objects = draft.objects.map((object) => editObject(object, plane, draft.view));
  return { ...draft, objects };
}

export { buildEditScene };
