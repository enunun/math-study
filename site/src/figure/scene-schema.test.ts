import { readFileSync } from 'node:fs';
import { readFile, readdir } from 'node:fs/promises';
import path from 'node:path';

import type { ValidateFunction } from 'ajv';
import Ajv2020 from 'ajv/dist/2020.js';
import { describe, expect, it } from 'vitest';

import { createObject } from '@/figure-editor/create';
import { addObject, emptyDraft, stringifyDraft } from '@/figure-editor/draft';
import type { SceneDraft, ViewKind } from '@/figure-editor/draft';

const SCHEMA_PATH = path.resolve(import.meta.dirname, '../../public/schema/scene.schema.json');
const FIGURES_DIR = path.resolve(import.meta.dirname, '../figures');

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

/** オブジェクトなら，そのオブジェクト．そうでなければ，空のオブジェクト． */
function objectOf(value: unknown): Record<string, unknown> {
  return isRecord(value) ? value : {};
}

/** 信頼できる自分のJSONを，オブジェクトとして読む(`JSON.parse`は`any`を返すので，一度`unknown`を経由する)． */
function parsedJson(text: string): Record<string, unknown> {
  const value: unknown = JSON.parse(text);
  if (!isRecord(value)) {
    throw new Error('シーンのJSONは，オブジェクトで書く．');
  }
  return value;
}

const schema = parsedJson(readFileSync(SCHEMA_PATH, 'utf8'));
// strictRequired: `grid`と`surface`は，anyOf/oneOfの枝の中で，親のpropertiesにある項目をrequiredにする
// (「x_stepかy_stepの少なくとも一方」のような，選べる必須項目を書く，一般的な書き方)．厳格さは落とさない．
const ajv = new Ajv2020({ allErrors: true, strict: true, strictRequired: false });
const validate: ValidateFunction = ajv.compile(schema);

/** 最小限の，平面か空間のシーン．型ごとの検査は，オブジェクトを1つ足して書き換える． */
function minimalScene(kind: ViewKind): SceneDraft {
  return emptyDraft(kind);
}

function minimalObject(kind: ViewKind): Record<string, unknown> {
  return parsedJson(stringifyDraft(minimalScene(kind)));
}

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
              'point',
              'point',
              'label',
              'vector',
              'segment',
              'curve',
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

describe('シーンのJSON Schemaは，構造の誤りを断る', () => {
  const plane = minimalObject('plane');
  const space = minimalObject('space');

  function invalid(scene: unknown): boolean {
    return !validate(scene);
  }

  it('シーンに要る項目がなければ断る', () => {
    const { version: _version, ...rest } = plane;
    expect(invalid(rest)).toBe(true);
  });

  it('シーンに知らない項目があれば断る', () => {
    expect(invalid({ ...plane, extra: 1 })).toBe(true);
  });

  it('viewが，平面の図の形にも空間の図の形にも合わなければ断る', () => {
    expect(invalid({ ...plane, view: { foo: 1 } })).toBe(true);
  });

  it('viewが，平面の図と空間の図，両方の項目を持てば断る', () => {
    const mixed = { ...objectOf(plane.view), ...objectOf(space.view) };
    expect(invalid({ ...plane, view: mixed })).toBe(true);
  });

  it('仰角が90度を超えれば断る', () => {
    expect(invalid({ ...space, view: { ...objectOf(space.view), elevation: 95 } })).toBe(true);
  });

  it('知らないtypeのオブジェクトは断る', () => {
    expect(invalid({ ...plane, objects: [{ id: 'a', type: 'triangle' }] })).toBe(true);
  });

  it('座標軸に向き(direction)がなければ断る', () => {
    expect(invalid({ ...plane, objects: [{ id: 'a', type: 'axis' }] })).toBe(true);
  });

  it('曲線の式が1個だけなら断る(平面は2個，空間は3個)', () => {
    expect(
      invalid({
        ...plane,
        objects: [{ id: 'c', type: 'curve', var: 't', expr: ['cos(t)'], domain: [0, 1] }],
      }),
    ).toBe(true);
  });

  it('曲面が，式とベジエ曲面の両方を持てば断る', () => {
    expect(
      invalid({
        ...space,
        objects: [
          {
            id: 's',
            type: 'surface',
            vars: ['u', 'v'],
            expr: ['u', 'v', '0'],
            domain: [
              [0, 1],
              [0, 1],
            ],
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
          },
        ],
      }),
    ).toBe(true);
  });

  it('曲面が，式もベジエ曲面も持たなければ断る', () => {
    expect(invalid({ ...space, objects: [{ id: 's', type: 'surface' }] })).toBe(true);
  });

  it('制御点の網(control_net)を，ベジエ曲面でない曲面に指定すれば断る', () => {
    expect(
      invalid({
        ...space,
        objects: [
          {
            id: 's',
            type: 'surface',
            vars: ['u', 'v'],
            expr: ['u', 'v', '0'],
            domain: [
              [0, 1],
              [0, 1],
            ],
            control_net: {},
          },
        ],
      }),
    ).toBe(true);
  });

  it('格子が，x_stepもy_stepも持たなければ断る', () => {
    expect(invalid({ ...plane, objects: [{ id: 'g', type: 'grid' }] })).toBe(true);
  });

  it('交線が，同じ曲面を2つ指せば断る', () => {
    expect(
      invalid({
        ...space,
        objects: [{ id: 'i', type: 'intersection', surfaces: ['s', 's'] }],
      }),
    ).toBe(true);
  });

  it('球の半径が0以下なら断る', () => {
    expect(
      invalid({
        ...space,
        objects: [{ id: 'b', type: 'sphere', center: [0, 0, 0], radius: 0 }],
      }),
    ).toBe(true);
  });

  it('スタイルに，知らない項目(色の綴りの誤りなど)があれば断る', () => {
    expect(
      invalid({
        ...plane,
        objects: [{ id: 'g', type: 'grid', x_step: 1, style: { colour: 'gray' } }],
      }),
    ).toBe(true);
  });
});
