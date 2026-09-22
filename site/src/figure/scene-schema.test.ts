import { readFile, readdir } from 'node:fs/promises';
import path from 'node:path';

import { describe, expect, it } from 'vitest';

import { createObject } from '@/figure-editor/create';
import { addObject, stringifyDraft } from '@/figure-editor/draft';

import { minimalObject, minimalScene, parsedJson, validate } from './scene-schema-support';

const FIGURES_DIR = path.resolve(import.meta.dirname, '../figures');

describe('シーンのJSON Schemaは，記事とサイトの見本のシーンを受け入れる', () => {
  it('site/src/figures/のすべてのシーンが，スキーマに合う', async () => {
    const entries = await readdir(FIGURES_DIR);
    const files = entries.filter((file) => file.endsWith('.json'));
    expect(files.length).toBeGreaterThan(0);
    for (const file of files) {
      // eslint-disable-next-line no-await-in-loop -- テストの読みやすさを優先し，フィクスチャを順に読む
      const scene = parsedJson(await readFile(path.join(FIGURES_DIR, file), 'utf8'));
      const ok = validate(scene);
      expect(ok, `${file}: ${JSON.stringify(validate.errors)}`).toBe(true);
    }
  });

  it('曲面と球のワイヤーフレーム，ベジエ曲面の制御点の網は，スキーマに合う', () => {
    const space = minimalObject('space');
    expect(
      validate({
        ...space,
        objects: [
          { id: 'b', type: 'sphere', center: [0, 0, 0], radius: 1, wireframe: {} },
          {
            id: 's',
            type: 'surface',
            vars: ['u', 'v'],
            expr: ['u', 'v', '0'],
            domain: [
              [0, 1],
              [0, 1],
            ],
            wireframe: { color: 'blue' },
          },
          {
            id: 'z',
            type: 'surface',
            bezier: [
              [
                [0, 0, 0],
                [0, 1, 0],
              ],
              [
                [1, 0, 0],
                [1, 1, 0],
              ],
            ],
            wireframe: {},
            control_net: { color: 'gray' },
          },
        ],
      }),
      JSON.stringify(validate.errors),
    ).toBe(true);
  });

  it('ベジエ曲線は，平面でも空間でも，スキーマに合う', () => {
    const plane = minimalObject('plane');
    const space = minimalObject('space');
    expect(
      validate({
        ...plane,
        objects: [
          {
            id: 'c',
            type: 'curve',
            bezier: [
              [0, 0],
              [1, 2],
              [2, 0],
            ],
          },
        ],
      }),
      JSON.stringify(validate.errors),
    ).toBe(true);
    expect(
      validate({
        ...space,
        objects: [
          {
            id: 'c',
            type: 'curve',
            bezier: [
              [0, 0, 0],
              [1, 2, 1],
              [2, 0, 0],
            ],
          },
        ],
      }),
      JSON.stringify(validate.errors),
    ).toBe(true);
  });

  it('スプライン曲線は，平面でも空間でも，スキーマに合う', () => {
    const plane = minimalObject('plane');
    const space = minimalObject('space');
    expect(
      validate({
        ...plane,
        objects: [
          {
            id: 'c',
            type: 'curve',
            spline: [
              [0, 0],
              [1, 2],
              [2, 0],
              [3, 1],
            ],
          },
        ],
      }),
      JSON.stringify(validate.errors),
    ).toBe(true);
    expect(
      validate({
        ...space,
        objects: [
          {
            id: 'c',
            type: 'curve',
            spline: [
              [0, 0, 0],
              [1, 2, 1],
              [2, 0, 0],
              [3, 1, 1],
            ],
          },
        ],
      }),
      JSON.stringify(validate.errors),
    ).toBe(true);
  });

  it('接線と接平面は，スキーマに合う', () => {
    const plane = minimalObject('plane');
    const space = minimalObject('space');
    expect(
      validate({
        ...plane,
        objects: [
          { id: 'f', type: 'graph', var: 'x', expr: 'x^2', domain: [-3, 3] },
          { id: 't', type: 'tangent_line', of: 'f', at: 1 },
        ],
      }),
      JSON.stringify(validate.errors),
    ).toBe(true);
    expect(
      validate({
        ...space,
        objects: [
          {
            id: 's',
            type: 'surface',
            vars: ['u', 'v'],
            expr: ['u', 'v', 'u^2 + v^2'],
            domain: [
              [-2, 2],
              [-2, 2],
            ],
          },
          { id: 'p', type: 'tangent_plane', of: 's', at: [0, 0], size: 1 },
        ],
      }),
      JSON.stringify(validate.errors),
    ).toBe(true);
  });

  it.each(['plane', 'space'] as const)(
    '図の作成ページが作る，%sの図の既定のオブジェクトが，スキーマに合う',
    (kind) => {
      let draft = minimalScene(kind);
      const types =
        kind === 'plane'
          ? [
              'axis',
              'axis',
              'parameter',
              'graph',
              'curve',
              'bezierCurve',
              'splineCurve',
              'tangent_line',
              'grid',
              'point',
              'point',
              'label',
              'vector',
              'segment',
              'region',
            ]
          : [
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
      for (const type of types) {
        draft = addObject(draft, createObject(type, draft, kind));
      }
      const scene = parsedJson(stringifyDraft(draft));
      const ok = validate(scene);
      expect(ok, JSON.stringify(validate.errors)).toBe(true);
    },
  );
});
