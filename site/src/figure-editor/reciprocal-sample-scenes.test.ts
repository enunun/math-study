import { readFile } from 'node:fs/promises';

import { beforeAll, describe, expect, it } from 'vitest';

import { initSync, renderScene } from '@/wasm/figure';

import { stringifyDraft } from './draft';
import type { SceneDraft } from './draft';
import { arrayOf, objectOf, stringOf } from './json';
import type { JsonObject } from './json';
import { RECIPROCAL_PLANE_SAMPLES, RECIPROCAL_SPACE_SAMPLES } from './reciprocal-sample-scenes';

beforeAll(async () => {
  const module = await readFile(new URL('../wasm/figure_bg.wasm', import.meta.url));
  initSync({ module });
});

/** 数の並び(数でない要素は除く)． */
function numbers(values: readonly unknown[]): number[] {
  return values.filter((value): value is number => typeof value === 'number');
}

/** スライダーの端と，刻みがあれば刻みごとの値．媒介変数の値の組み合わせを試すのに使う． */
function valuesOf(parameter: JsonObject): number[] {
  const [min = 0, max = 0] = numbers(arrayOf(parameter, 'range'));
  const { step } = parameter;
  if (typeof step !== 'number') {
    return [min, max];
  }
  const count = Math.round((max - min) / step);
  return Array.from({ length: count + 1 }, (_, n) => min + n * step);
}

function withValue(draft: SceneDraft, id: string, value: number): SceneDraft {
  const objects: JsonObject[] = [];
  for (const object of draft.objects) {
    objects.push(object.id === id ? { ...object, value } : object);
  }
  return { ...draft, objects };
}

/** 範囲のある媒介変数の値の，全部の組み合わせで作った図． */
function variants(scene: SceneDraft): SceneDraft[] {
  let drafts = [scene];
  for (const parameter of scene.objects) {
    if (parameter.type === 'parameter' && Array.isArray(parameter.range)) {
      const id = stringOf(parameter, 'id');
      drafts = drafts.flatMap((draft) =>
        valuesOf(parameter).map((value) => withValue(draft, id, value)),
      );
    }
  }
  return drafts;
}

/** 図の値の組み合わせを，媒介変数の名前と値で書いた文字列(失敗したときに示す)． */
function valuesText(draft: SceneDraft): string {
  return JSON.stringify(
    draft.objects
      .filter((object) => object.type === 'parameter')
      .map((object) => [object.id, object.value]),
  );
}

/** 位置の比較の許容(cm)． */
const TOLERANCE = 1e-9;

describe('逆格子の見本は，スライダーのどの値でも描ける', () => {
  it.each([...RECIPROCAL_PLANE_SAMPLES, ...RECIPROCAL_SPACE_SAMPLES])('$label', ({ scene }) => {
    for (const draft of variants(scene)) {
      const outcome = renderScene(stringifyDraft(draft));
      expect(outcome.status, `${valuesText(draft)}：${JSON.stringify(outcome)}`).toBe('ok');
    }
  });
});

describe('平面の逆格子の見本の点の印は，スライダーのどの値でも，見える範囲に収まる', () => {
  // 見える範囲の外の点は，SVGでは切れるが，TikZの図を広げてしまう．
  it.each(RECIPROCAL_PLANE_SAMPLES)('$label', ({ scene }) => {
    const unit = Number(stringOf(objectOf(scene.view, 'unit'), 'x').replace('cm', ''));
    const [xMin, xMax] = numbers(arrayOf(scene.view, 'x')).map((value) => value * unit);
    const [yMin, yMax] = numbers(arrayOf(scene.view, 'y')).map((value) => value * unit);
    for (const draft of variants(scene)) {
      const outcome = renderScene(stringifyDraft(draft));
      if (outcome.status !== 'ok') {
        throw new Error(valuesText(draft));
      }
      for (const item of outcome.figure.items) {
        if (item.type === 'dot') {
          const [x, y] = item.at;
          const inside =
            x >= (xMin ?? 0) - TOLERANCE &&
            x <= (xMax ?? 0) + TOLERANCE &&
            y >= (yMin ?? 0) - TOLERANCE &&
            y <= (yMax ?? 0) + TOLERANCE;
          expect(inside, `${valuesText(draft)}：(${x}, ${y})`).toBe(true);
        }
      }
    }
  });
});
