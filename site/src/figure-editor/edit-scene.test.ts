import { readFile } from 'node:fs/promises';

import { beforeAll, describe, expect, it } from 'vitest';

import { initSync, renderScene } from '@/wasm/figure';

import { addObject, emptyDraft, parseDraft, stringifyDraft } from './draft';
import type { SceneDraft } from './draft';
import { buildEditScene } from './edit-scene';
import { stringOf } from './json';
import type { Json, JsonObject } from './json';
import { EDITOR_SAMPLES } from './samples';

beforeAll(async () => {
  const module = await readFile(new URL('../wasm/figure_bg.wasm', import.meta.url));
  initSync({ module });
});

const AXIS: JsonObject = { id: 'x_axis', type: 'axis', direction: 'x', label: 'x' };
const SPACE_AXIS: JsonObject = { id: 'x_axis', type: 'axis', direction: 'x', range: [-5, 5] };
const GRID: JsonObject = { id: 'grid1', type: 'grid', x_step: 1, y_step: 1 };
const CURVE: JsonObject = {
  id: 'curve1',
  type: 'curve',
  var: 'x',
  expr: ['x', 'x'],
  domain: [-1, 1],
};

/** 1つのオブジェクトだけを持つ図に，編集の補助を加えた，そのオブジェクト． */
function editedObjectOf(object: JsonObject, kind: 'plane' | 'space' = 'plane'): JsonObject {
  const [edited] = buildEditScene(addObject(emptyDraft(kind), object)).objects;
  return edited ?? {};
}

function numberAt(value: Json): number {
  return typeof value === 'object' && value !== null && !Array.isArray(value) && 'at' in value
    ? Number(value.at)
    : Number.NaN;
}

describe('編集の補助を加えた図', () => {
  it('格子と座標軸を，補助のスタイルにする．ほかのオブジェクトは変えない', () => {
    let draft: SceneDraft = emptyDraft();
    draft = addObject(draft, AXIS);
    draft = addObject(draft, CURVE);
    draft = addObject(draft, GRID);
    const edited = buildEditScene(draft);
    const axis = edited.objects.find((object) => stringOf(object, 'id') === 'x_axis');
    const grid = edited.objects.find((object) => stringOf(object, 'id') === 'grid1');
    const curve = edited.objects.find((object) => stringOf(object, 'id') === 'curve1');
    expect(axis?.style).toEqual({ line: 'dotted', color: 'gray', width: '0.4pt' });
    expect(grid?.style).toEqual({ line: 'dotted', color: 'gray', width: '0.4pt' });
    expect(curve).toEqual(CURVE);
  });

  it('平面の座標軸に，読みやすい目盛を足す(既に目盛があれば，そのままにする)', () => {
    const edited = editedObjectOf(AXIS);
    expect(Array.isArray(edited.ticks) ? edited.ticks.length : 0).toBeGreaterThan(0);

    const kept = editedObjectOf({ ...AXIS, ticks: [{ at: 1 }] });
    expect(kept.ticks).toEqual([{ at: 1 }]);
  });

  it('目盛の範囲は，軸自身のrangeがあればそれを，なければ図の見える範囲を使う', () => {
    const edited = editedObjectOf({ ...AXIS, range: [0, 100] });
    const ticks = Array.isArray(edited.ticks) ? edited.ticks : [];
    expect(ticks.length).toBeGreaterThan(0);
    for (const tick of ticks) {
      const at = numberAt(tick);
      expect(at).toBeGreaterThanOrEqual(0);
      expect(at).toBeLessThanOrEqual(100);
    }
  });

  it('空間の図の座標軸には，目盛を足さない(エンジンがまだ受け付けない)', () => {
    const edited = editedObjectOf(SPACE_AXIS, 'space');
    expect(edited.ticks).toBeUndefined();
    expect(edited.style).toEqual({ line: 'dotted', color: 'gray', width: '0.4pt' });
  });

  it('曲面のワイヤーフレームは，シーン自身の項目なので，ここでは変えない', () => {
    const surface: JsonObject = {
      id: 's',
      type: 'surface',
      vars: ['u', 'v'],
      expr: ['u', 'v', '0'],
      domain: [
        [-1, 1],
        [-1, 1],
      ],
      wireframe: { color: 'red' },
    };
    expect(editedObjectOf(surface, 'space')).toEqual(surface);
  });
});

describe('見本の図に，編集の補助を加える', () => {
  it.each(EDITOR_SAMPLES)('「$label」の補助を加えても，エンジンが描ける', ({ json }) => {
    const parsed = parseDraft(json);
    if (!parsed.ok) {
      throw new Error(parsed.message);
    }
    const edited = buildEditScene(parsed.draft);
    const outcome = renderScene(stringifyDraft(edited));
    if (outcome.status !== 'ok') {
      throw new Error(`${outcome.message}(${outcome.object ?? ''})`);
    }
  });
});
