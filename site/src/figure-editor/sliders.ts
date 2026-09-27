import type { SceneDraft } from './draft';
import { arrayOf, stringOf } from './json';
import type { JsonObject } from './json';

/** 範囲のある媒介変数．編集画面は，これをスライダーにする． */
interface Slider {
  id: string;
  value: number;
  min: number;
  max: number;
}

/** 書き込む値の細かさ．範囲の幅をこの数で割った刻みに丸め，JSONに長い小数が入らないようにする． */
const VALUE_DIVISIONS = 10_000;
/** 丸めた値の有効数字．浮動小数点の誤差(0.30000000000000004など)を消す． */
const VALUE_PRECISION = 12;
/** 範囲は，下端と上端の2つの数である． */
const RANGE_LENGTH = 2;

function sliderOf(object: JsonObject): Slider | undefined {
  const range = arrayOf(object, 'range');
  const [min, max] = range;
  const { value } = object;
  if (
    stringOf(object, 'type') !== 'parameter' ||
    range.length !== RANGE_LENGTH ||
    typeof min !== 'number' ||
    typeof max !== 'number' ||
    typeof value !== 'number' ||
    !(min < max)
  ) {
    return undefined;
  }
  return { id: stringOf(object, 'id'), value, min, max };
}

/** 図の中の，範囲のある媒介変数(並びの順)．範囲の形が崩れているものは除く(エンジンが誤りを示す)． */
function slidersOf(draft: SceneDraft): Slider[] {
  return draft.objects.flatMap((object) => {
    const slider = sliderOf(object);
    return slider === undefined ? [] : [slider];
  });
}

/** 媒介変数`id`の値を書き換えた図．値は，範囲の幅の`VALUE_DIVISIONS`分の1に丸める． */
function withParameterValue(draft: SceneDraft, id: string, value: number): SceneDraft {
  const objects = draft.objects.map((object) => {
    const slider = sliderOf(object);
    if (slider === undefined || slider.id !== id) {
      return object;
    }
    const step = (slider.max - slider.min) / VALUE_DIVISIONS;
    const rounded = Number((Math.round(value / step) * step).toPrecision(VALUE_PRECISION));
    return { ...object, value: rounded };
  });
  return { ...draft, objects };
}

/**
 * 再生を始めてから`elapsed`ミリ秒後の値．始めの値から増え，`period`ミリ秒で範囲を1周し，
 * 上端を越えると下端に戻る．
 */
function playedValue(slider: Slider, elapsed: number, period: number): number {
  const width = slider.max - slider.min;
  const phase = (slider.value - slider.min) / width + elapsed / period;
  return slider.min + (phase - Math.floor(phase)) * width;
}

export { playedValue, slidersOf, withParameterValue };
export type { Slider };
