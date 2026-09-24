import { readFile } from 'node:fs/promises';

import { beforeAll, describe, expect, it } from 'vitest';

import { initSync, renderScene } from '@/wasm/figure';

import { createObject, OBJECT_TYPES, TRANSFORMABLE } from './create';
import { addObject, emptyDraft, stringifyDraft } from './draft';
import type { SceneDraft, ViewKind } from './draft';
import { fieldsFor } from './fields';
import { stringOf } from './json';
import type { JsonObject } from './json';

beforeAll(async () => {
  const module = await readFile(new URL('../wasm/figure_bg.wasm', import.meta.url));
  initSync({ module });
});

const PLANE_TYPES = [
  'axis',
  'axis',
  'parameter',
  'graph',
  'curve',
  'bezierCurve',
  'splineCurve',
  'tangent_line',
  'grid',
  'fractal',
  'point',
  'point',
  'label',
  'vector',
  'segment',
  'region',
  'polygon',
  'vertexPolygon',
  'taylor',
  'function',
  'map',
  'image',
];

const SPACE_TYPES = [
  'axis',
  'parameter',
  'sphere',
  'surface',
  'bezier',
  'cut',
  'intersection',
  'tangent_plane',
  'complex',
  'point',
  'point',
  'label',
  'vector',
  'segment',
  'curve',
  'bezierCurve',
  'splineCurve',
  'grid',
  'polyhedron',
  'function',
  'map',
  'image',
];

/** 種類を順に，初期値で追加した図． */
function draftOf(kind: ViewKind, types: readonly string[]): SceneDraft {
  let draft = emptyDraft(kind);
  for (const type of types) {
    draft = addObject(draft, createObject(type, draft, kind));
  }
  return draft;
}

function build(kind: ViewKind, types: readonly string[]): string {
  return stringifyDraft(draftOf(kind, types));
}

describe('新しいオブジェクトの初期値を，エンジンが読める', () => {
  it.each([
    ['plane', PLANE_TYPES],
    ['space', SPACE_TYPES],
  ] as const)('%sの図で，使える種類をすべて足しても，描ける', (kind, types) => {
    const outcome = renderScene(build(kind, types));
    if (outcome.status !== 'ok') {
      throw new Error(`${outcome.message}(${outcome.object ?? ''})`);
    }
    expect(outcome.tikz).toContain(String.raw`\begin{tikzpicture}`);
    expect(outcome.figure.items.length).toBeGreaterThan(0);
  });

  it('参照する相手がまだないベクトルは，エンジンが，原因のオブジェクトを示して断る', () => {
    const outcome = renderScene(build('plane', ['vector']));
    expect(outcome.status).toBe('error');
    expect(outcome.status === 'error' && outcome.object).toBe('vector1');
  });
});

describe('変換は，座標軸・媒介変数・関数・写像のほかのすべてに使える', () => {
  const untransformable = new Set(['axis', 'parameter', 'function', 'map']);

  it.each(OBJECT_TYPES)('$labelの変換の項目', ({ type, kinds }) => {
    const [kind] = kinds;
    const object = createObject(type, emptyDraft(kind ?? 'plane'), kind ?? 'plane');
    const count = fieldsFor(object, kind ?? 'plane').filter(
      (spec) => spec.kind === 'transform',
    ).length;
    expect(count).toBe(untransformable.has(type) ? 0 : 1);
  });

  it.each([
    ['plane', PLANE_TYPES, [2, 1]],
    ['space', SPACE_TYPES, [0, 0, 1]],
  ] as const)('%sの図で，変換できるものをすべて平行移動しても，描ける', (kind, types, offset) => {
    const draft = draftOf(kind, types);
    // 像は自分の変換を持ち，グラフは領域とテイラー展開が参照するので，動かさない．
    const fixed = new Set(['image', 'graph']);
    const objects: JsonObject[] = [];
    for (const object of draft.objects) {
      const type = stringOf(object, 'type');
      const movable = TRANSFORMABLE.includes(type) && !fixed.has(type);
      objects.push(movable ? { ...object, transform: [{ translate: [...offset] }] } : object);
    }
    const moved = { ...draft, objects };
    const outcome = renderScene(stringifyDraft(moved));
    if (outcome.status !== 'ok') {
      throw new Error(`${outcome.message}(${outcome.object ?? ''})`);
    }
  });
});
