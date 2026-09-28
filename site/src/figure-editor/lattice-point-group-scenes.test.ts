import { readFile } from 'node:fs/promises';

import { beforeAll, describe, expect, it } from 'vitest';

import { initSync } from '@/wasm/figure';

import { PLANE_POINT_GROUP_SAMPLES } from './plane-point-group-scenes';
import { PLANE_SYMMETRY_OPERATION_SAMPLES } from './plane-symmetry-operation-scenes';
import { SPACE_POINT_GROUP_SAMPLES } from './space-point-group-scenes';
import { SPACE_SYMMETRY_OPERATION_SAMPLES } from './space-symmetry-operation-scenes';
import { coincide, matchedFraction, sample, withValues } from './symmetry-test-support';

beforeAll(async () => {
  const module = await readFile(new URL('../wasm/figure_bg.wasm', import.meta.url));
  initSync({ module });
});

const ANGLES = Array.from({ length: 24 }, (_, index) => index * 15);

describe('平面の格子の点群の見本は，θとtの組を全部試すと，点群の元の数だけ重なる', () => {
  it.each([
    ['planeObliquePointGroup', 2],
    ['planeRectanglePointGroup', 4],
    ['planeCenteredRectanglePointGroup', 4],
    ['planeSquarePointGroup', 8],
    ['planeHexagonalPointGroup', 12],
  ] as const)('%s：%i個', (id, order) => {
    const scene = sample(PLANE_POINT_GROUP_SAMPLES, id);
    const found = ANGLES.flatMap((theta) =>
      [0, 1].filter((t) => coincide(withValues(scene, { theta, t }))).map((t) => [theta, t]),
    );
    expect(found).toHaveLength(order);
  });
});

const SQUARE = { b: 1, angle_gamma: 90 };
const HEXAGONAL = { b: 1, angle_gamma: 60 };
const OBLIQUE = { b: 1.2, angle_gamma: 70 };

interface Case {
  id: string;
  symmetric: readonly Readonly<Record<string, number>>[];
  other: readonly Readonly<Record<string, number>>[];
}

const PLANE_CASES: readonly Case[] = [
  {
    id: 'latticeRotationCenter',
    symmetric: [
      { ...SQUARE, p: 3, q: 3, theta: 90 },
      { ...SQUARE, p: 3, q: 0, theta: 180 },
      { ...HEXAGONAL, p: 2, q: 2, theta: 120 },
      { ...HEXAGONAL, p: 4, q: 4, theta: 240 },
      { ...HEXAGONAL, p: 0, q: 0, theta: 60 },
      { ...OBLIQUE, p: 3, q: 3, theta: 180 },
    ],
    other: [
      { ...SQUARE, p: 3, q: 3, theta: 45 },
      { ...SQUARE, p: 3, q: 0, theta: 90 },
      { ...HEXAGONAL, p: 2, q: 2, theta: 60 },
      { ...OBLIQUE, p: 3, q: 3, theta: 90 },
    ],
  },
  {
    id: 'crystallographicRestriction',
    symmetric: [
      { ...OBLIQUE, n: 1 },
      { ...OBLIQUE, n: 2 },
      { ...SQUARE, n: 4 },
      { ...HEXAGONAL, n: 3 },
      { ...HEXAGONAL, n: 6 },
    ],
    other: [5, 7, 8].flatMap((n) => [
      { ...SQUARE, n },
      { ...HEXAGONAL, n },
      { ...OBLIQUE, n },
    ]),
  },
  {
    id: 'latticeGlide',
    symmetric: [
      { t: 1, h: 0, s: 0 },
      { t: 1, h: 0.5, s: 0 },
      { t: 1, h: 0.25, s: 0.5 },
      { t: 1, h: 0.75, s: 0.5 },
      { t: 1, h: 0, s: 1 },
    ],
    other: [
      { t: 1, h: 0.25, s: 0 },
      { t: 1, h: 0, s: 0.5 },
      { t: 1, h: 0.25, s: 1 },
    ],
  },
];

describe('平面の格子の対称操作の見本は，対称操作の値でだけ重なる', () => {
  it.each(PLANE_CASES)('$id', ({ id, symmetric, other }) => {
    const scene = sample(PLANE_SYMMETRY_OPERATION_SAMPLES, id);
    for (const values of symmetric) {
      expect(coincide(withValues(scene, values)), JSON.stringify(values)).toBe(true);
    }
    for (const values of other) {
      expect(coincide(withValues(scene, values)), JSON.stringify(values)).toBe(false);
    }
  });
});

/** 晶系ごとの，回転軸(媒介変数kの順)の回転の回数． */
const SPACE_AXES: readonly (readonly [string, readonly number[]])[] = [
  ['triclinicPointGroup', []],
  ['monoclinicPointGroup', [2]],
  ['orthorhombicPointGroup', [2, 2, 2]],
  ['tetragonalPointGroup', [4, 2, 2, 2, 2]],
  ['trigonalPointGroup', [3, 2, 2, 2]],
  ['hexagonalPointGroup', [6, 2, 2, 2, 2, 2, 2]],
  ['cubicPointGroup', [4, 4, 4, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2]],
];

describe('空間の格子の点群の見本', () => {
  it.each(SPACE_AXES)('%s：軸ごとの回転の回数と，反転', (id, orders) => {
    const scene = sample(SPACE_POINT_GROUP_SAMPLES, id);
    expect(scene.objects.some((object) => object.id === 'k')).toBe(orders.length > 1);
    for (const [k, order] of orders.entries()) {
      const at = (theta: number): boolean => coincide(withValues(scene, { k, theta, t: 0 }));
      expect(at(360 / order), `k = ${k}`).toBe(true);
      // 半分の角では重ならないので，軸の回転の回数はちょうどorderである．
      expect(at(180 / order), `k = ${k}`).toBe(false);
    }
    expect(coincide(withValues(scene, { t: 1 }))).toBe(true);
    expect(coincide(withValues(scene, { t: 0.3 }))).toBe(false);
  });

  it('回反：正方晶系の4̄は重なり，三方晶系の6̄にあたる操作は重ならない', () => {
    const tetragonal = sample(SPACE_POINT_GROUP_SAMPLES, 'tetragonalPointGroup');
    expect(coincide(withValues(tetragonal, { k: 0, theta: 90, t: 1 }))).toBe(true);
    const trigonal = sample(SPACE_POINT_GROUP_SAMPLES, 'trigonalPointGroup');
    expect(coincide(withValues(trigonal, { k: 0, theta: 120, t: 1 }))).toBe(true);
    expect(coincide(withValues(trigonal, { k: 0, theta: 60, t: 1 }))).toBe(false);
  });
});

describe('らせん軸と映進面の見本は，並進を合わせたときだけ，内側の格子点が重なる', () => {
  it('らせん軸4_2', () => {
    const scene = sample(SPACE_SYMMETRY_OPERATION_SAMPLES, 'latticeScrew');
    expect(matchedFraction(withValues(scene, { theta: 180, s: 0 }))).toBe(1);
    expect(matchedFraction(withValues(scene, { theta: 90, s: 0.5 }))).toBeGreaterThanOrEqual(0.75);
    expect(matchedFraction(withValues(scene, { theta: 90, s: 0 }))).toBe(0);
    expect(matchedFraction(withValues(scene, { theta: 0, s: 0.5 }))).toBe(0);
  });

  it('映進面', () => {
    const scene = sample(SPACE_SYMMETRY_OPERATION_SAMPLES, 'latticeGlidePlane');
    expect(matchedFraction(withValues(scene, { t: 1, s: 0.5 }))).toBeGreaterThanOrEqual(0.6);
    expect(matchedFraction(withValues(scene, { t: 1, s: 0 }))).toBe(0);
    expect(matchedFraction(withValues(scene, { t: 0, s: 0.5 }))).toBe(0);
  });
});
