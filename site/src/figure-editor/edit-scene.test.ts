import { readFile } from 'node:fs/promises';

import { beforeAll, describe, expect, it } from 'vitest';

import { initSync, renderScene } from '@/wasm/figure';

import { addObject, emptyDraft, parseDraft, stringifyDraft } from './draft';
import { buildEditScene } from './edit-scene';
import { stringOf } from './json';
import type { JsonObject } from './json';
import { EDITOR_SAMPLES } from './samples';

beforeAll(async () => {
  const module = await readFile(new URL('../wasm/figure_bg.wasm', import.meta.url));
  initSync({ module });
});

const AXIS: JsonObject = { id: 'x_axis', type: 'axis', direction: 'x', label: 'x' };
const GRID: JsonObject = { id: 'grid1', type: 'grid', x_step: 1, y_step: 1 };
const CURVE: JsonObject = {
  id: 'curve1',
  type: 'curve',
  var: 'x',
  expr: ['x', 'x'],
  domain: [-1, 1],
};
const FORMULA_SURFACE: JsonObject = {
  id: 'surface1',
  type: 'surface',
  vars: ['u', 'v'],
  expr: ['u', 'v', 'u^2 + v^2'],
  domain: [
    [-2, 2],
    [-2, 2],
  ],
};
const BEZIER_SURFACE: JsonObject = {
  id: 'bezier1',
  type: 'surface',
  bezier: [
    [
      [0, 0, 0],
      [0, 2, 0],
    ],
    [
      [2, 0, 0],
      [2, 2, 1],
    ],
  ],
};

describe('編集の補助を加えた図', () => {
  it('格子と座標軸を，補助のスタイルにする．ほかのオブジェクトは変えない', () => {
    const draft = addObject(addObject(emptyDraft(), AXIS), CURVE);
    const withGrid = addObject(draft, GRID);
    const edited = buildEditScene(withGrid, { wireframe: false });
    const axis = edited.objects.find((object) => stringOf(object, 'id') === 'x_axis');
    const grid = edited.objects.find((object) => stringOf(object, 'id') === 'grid1');
    const curve = edited.objects.find((object) => stringOf(object, 'id') === 'curve1');
    expect(axis?.style).toEqual({ line: 'dotted', color: 'gray', width: '0.4pt' });
    expect(grid?.style).toEqual({ line: 'dotted', color: 'gray', width: '0.4pt' });
    expect(curve).toEqual(CURVE);
  });

  it('ワイヤーフレームを求めないときは，オブジェクトの数が変わらない', () => {
    const draft = addObject(emptyDraft('space'), FORMULA_SURFACE);
    const edited = buildEditScene(draft, { wireframe: false });
    expect(edited.objects).toHaveLength(1);
  });

  it('式の曲面には，u一定・v一定の断面を，エンジンが描ける形で足す', () => {
    const draft = addObject(emptyDraft('space'), FORMULA_SURFACE);
    const edited = buildEditScene(draft, { wireframe: true });
    const wires = edited.objects.filter((object) => object !== FORMULA_SURFACE);
    expect(wires.length).toBeGreaterThan(0);
    expect(wires.every((object) => stringOf(object, 'type') === 'curve')).toBe(true);
    const outcome = renderScene(stringifyDraft(edited));
    if (outcome.status !== 'ok') {
      throw new Error(`${outcome.message}(${outcome.object ?? ''})`);
    }
  });

  it('ベジエ曲面には，制御点の網を，エンジンが描ける形で足す', () => {
    const draft = addObject(emptyDraft('space'), BEZIER_SURFACE);
    const edited = buildEditScene(draft, { wireframe: true });
    const wires = edited.objects.filter((object) => object !== BEZIER_SURFACE);
    expect(wires.some((object) => stringOf(object, 'type') === 'point')).toBe(true);
    expect(wires.some((object) => stringOf(object, 'type') === 'segment')).toBe(true);
    const outcome = renderScene(stringifyDraft(edited));
    if (outcome.status !== 'ok') {
      throw new Error(`${outcome.message}(${outcome.object ?? ''})`);
    }
  });

  it('範囲が式で書かれた曲面は，補間できないので，ワイヤーフレームを足さない', () => {
    const expressed: JsonObject = {
      ...FORMULA_SURFACE,
      domain: [
        ['-r', 'r'],
        [-2, 2],
      ],
    };
    const draft = addObject(emptyDraft('space'), expressed);
    const edited = buildEditScene(draft, { wireframe: true });
    expect(edited.objects).toHaveLength(1);
  });

  it('新しく足すオブジェクトの識別子は，既にある識別子と重ならない', () => {
    const draft = addObject(
      addObject(emptyDraft('space'), { id: 'wire1', type: 'point', at: [0, 0, 0] }),
      FORMULA_SURFACE,
    );
    const edited = buildEditScene(draft, { wireframe: true });
    const ids = edited.objects.map((object) => stringOf(object, 'id'));
    expect(new Set(ids).size).toBe(ids.length);
  });
});

describe('見本の図に，編集の補助を加える', () => {
  it.each(EDITOR_SAMPLES)(
    '「$label」に，ワイヤーフレームを加えても，エンジンが描ける',
    ({ json }) => {
      const parsed = parseDraft(json);
      if (!parsed.ok) {
        throw new Error(parsed.message);
      }
      const edited = buildEditScene(parsed.draft, { wireframe: true });
      const outcome = renderScene(stringifyDraft(edited));
      if (outcome.status !== 'ok') {
        throw new Error(`${outcome.message}(${outcome.object ?? ''})`);
      }
    },
  );
});
