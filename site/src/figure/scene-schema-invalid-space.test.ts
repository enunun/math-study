import { describe, expect, it } from 'vitest';

import { minimalObject, validate } from './scene-schema-support';

function invalid(scene: unknown): boolean {
  return !validate(scene);
}

describe('シーンのJSON Schemaは，空間のオブジェクトの構造の誤りを断る', () => {
  const plane = minimalObject('plane');
  const space = minimalObject('space');

  it('交線が，同じ曲面を2つ指せば断る', () => {
    expect(
      invalid({
        ...space,
        objects: [{ id: 'i', type: 'intersection', surfaces: ['s', 's'] }],
      }),
    ).toBe(true);
  });

  it('接線に，ofかatがなければ断る', () => {
    expect(invalid({ ...plane, objects: [{ id: 't', type: 'tangent_line', at: 1 }] })).toBe(true);
    expect(invalid({ ...plane, objects: [{ id: 't', type: 'tangent_line', of: 'f' }] })).toBe(true);
  });

  it('接平面のsizeが0以下なら断る', () => {
    expect(
      invalid({
        ...space,
        objects: [{ id: 'p', type: 'tangent_plane', of: 's', at: [0, 0], size: 0 }],
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

  it('曲面のワイヤーフレームの本数が0以下なら断る', () => {
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
            wireframe: {},
            wireframe_lines: 0,
          },
        ],
      }),
    ).toBe(true);
  });

  it('複体の頂点が3個未満なら断る', () => {
    expect(
      invalid({
        ...space,
        objects: [
          {
            id: 't',
            type: 'complex',
            vertices: [
              [0, 0, 0],
              [1, 0, 0],
            ],
            faces: [[0, 1]],
          },
        ],
      }),
    ).toBe(true);
  });

  it('複体の面の頂点が3個未満なら断る', () => {
    expect(
      invalid({
        ...space,
        objects: [
          {
            id: 't',
            type: 'complex',
            vertices: [
              [0, 0, 0],
              [1, 0, 0],
              [0, 1, 0],
            ],
            faces: [[0, 1]],
          },
        ],
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
