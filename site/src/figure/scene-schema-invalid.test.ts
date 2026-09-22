import { describe, expect, it } from 'vitest';

import { minimalObject, objectOf, validate } from './scene-schema-support';

function invalid(scene: unknown): boolean {
  return !validate(scene);
}

describe('シーンのJSON Schemaは，構造の誤りを断る', () => {
  const plane = minimalObject('plane');
  const space = minimalObject('space');

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

  it('曲線が，式とベジエ曲線の両方を持てば断る', () => {
    expect(
      invalid({
        ...plane,
        objects: [
          {
            id: 'c',
            type: 'curve',
            var: 't',
            expr: ['cos(t)', 'sin(t)'],
            domain: [0, 1],
            bezier: [
              [0, 0],
              [1, 1],
            ],
          },
        ],
      }),
    ).toBe(true);
  });

  it('曲線が，式もベジエ曲線も持たなければ断る', () => {
    expect(invalid({ ...plane, objects: [{ id: 'c', type: 'curve' }] })).toBe(true);
  });

  it('曲線が，ベジエ曲線とスプライン曲線の両方を持てば断る', () => {
    expect(
      invalid({
        ...plane,
        objects: [
          {
            id: 'c',
            type: 'curve',
            bezier: [
              [0, 0],
              [1, 1],
            ],
            spline: [
              [0, 0],
              [1, 1],
            ],
          },
        ],
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

  it('スタイルに，知らない項目(色の綴りの誤りなど)があれば断る', () => {
    expect(
      invalid({
        ...plane,
        objects: [{ id: 'g', type: 'grid', x_step: 1, style: { colour: 'gray' } }],
      }),
    ).toBe(true);
  });
});
