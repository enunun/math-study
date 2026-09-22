import { readFile } from 'node:fs/promises';

import { beforeAll, describe, expect, it } from 'vitest';

import { initSync, renderScene } from '@/wasm/figure';

import { createObject } from './create';
import { addObject, emptyDraft, stringifyDraft } from './draft';
import type { ViewKind } from './draft';

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
  'point',
  'point',
  'label',
  'vector',
  'segment',
  'curve',
  'bezierCurve',
  'splineCurve',
];

/** 種類を順に，初期値で追加した図． */
function build(kind: ViewKind, types: readonly string[]): string {
  let draft = emptyDraft(kind);
  for (const type of types) {
    draft = addObject(draft, createObject(type, draft, kind));
  }
  return stringifyDraft(draft);
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
