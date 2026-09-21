import { readFile } from 'node:fs/promises';

import { beforeAll, describe, expect, it } from 'vitest';

import { initSync, renderScene } from '@/wasm/figure';

import { camera, dot, screenOf } from './camera';
import type { Vector3 } from './camera';

beforeAll(async () => {
  const module = await readFile(new URL('../wasm/figure_bg.wasm', import.meta.url));
  initSync({ module });
});

const DIGITS = 6;

function expectClose(
  actual: readonly number[],
  expected: readonly number[],
  digits = DIGITS,
): void {
  expect(actual).toHaveLength(expected.length);
  for (const [index, value] of actual.entries()) {
    expect(value).toBeCloseTo(expected[index] ?? Number.NaN, digits);
  }
}

describe('カメラの基底', () => {
  it('方位角60度，仰角20度で，解説の数値になる', () => {
    const { r, u, d } = camera(60, 20);
    const coarse = 3;
    expectClose(r, [-0.866, 0.5, 0], coarse);
    expectClose(u, [-0.171, -0.296, 0.94], coarse);
    expectClose(d, [0.47, 0.814, 0.342], coarse);
  });

  it.each([
    [0, 0],
    [60, 20],
    [-135, 55],
    [200, -30],
  ])('方位角%d度，仰角%d度で，正規直交の右手系になる', (azimuth, elevation) => {
    const { r, u, d } = camera(azimuth, elevation);
    expect(dot(r, r)).toBeCloseTo(1, DIGITS);
    expect(dot(u, u)).toBeCloseTo(1, DIGITS);
    expect(dot(d, d)).toBeCloseTo(1, DIGITS);
    expect(dot(r, u)).toBeCloseTo(0, DIGITS);
    expect(dot(u, d)).toBeCloseTo(0, DIGITS);
    expect(dot(d, r)).toBeCloseTo(0, DIGITS);
    // r×u = d
    const cross: Vector3 = [
      r[1] * u[2] - r[2] * u[1],
      r[2] * u[0] - r[0] * u[2],
      r[0] * u[1] - r[1] * u[0],
    ];
    expectClose(cross, d);
  });
});

describe('点の写り方', () => {
  it('点(3,1,2)は，画面の(-2.098, 1.070)に，奥行き2.907で写る', () => {
    const { x, y, depth } = screenOf(camera(60, 20), [3, 1, 2]);
    expect(x).toBeCloseTo(-2.098, 3);
    expect(y).toBeCloseTo(1.07, 3);
    expect(depth).toBeCloseTo(2.9074, 3);
  });

  it.each([
    [60, 20],
    [-30, 45],
    [140, -10],
  ])('図のエンジンが描く点の位置と，方位角%d度，仰角%d度で一致する', (azimuth, elevation) => {
    const scene = JSON.stringify({
      version: '0.1.0',
      description: '点の写り方の確認',
      view: { azimuth, elevation, unit: '1cm' },
      objects: [
        { id: 'O', type: 'point', at: [0, 0, 0] },
        { id: 'P', type: 'point', at: [3, 1, 2] },
        { id: 'v', type: 'vector', from: 'O', to: 'P' },
      ],
    });
    const outcome = renderScene(scene);
    if (outcome.status !== 'ok') {
      throw new Error(outcome.message);
    }
    const path = outcome.figure.items.find((item) => item.type === 'path');
    const end = path?.type === 'path' ? path.points.at(-1) : undefined;
    const { x, y } = screenOf(camera(azimuth, elevation), [3, 1, 2]);
    expect(end?.[0]).toBeCloseTo(x, 4);
    expect(end?.[1]).toBeCloseTo(y, 4);
  });
});
