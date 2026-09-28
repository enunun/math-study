import { readFile } from 'node:fs/promises';

import { beforeAll, describe, expect, it } from 'vitest';

import { initSync } from '@/wasm/figure';

import { SPACE_SYMMETRY_SAMPLES } from './space-symmetry-sample-scenes';
import { SYMMETRY_SAMPLES } from './symmetry-sample-scenes';
import { coincide, dotsByColor, sample, withValues } from './symmetry-test-support';
import type { SceneTemplate } from './template-types';

beforeAll(async () => {
  const module = await readFile(new URL('../wasm/figure_bg.wasm', import.meta.url));
  initSync({ module });
});

interface Case {
  samples: readonly SceneTemplate[];
  id: string;
  /** 対称操作になる媒介変数の値(2つの格子が重なる)． */
  symmetric: Readonly<Record<string, number>>;
  /** 対称操作にならない値(重ならない)． */
  other: Readonly<Record<string, number>>;
}

const HEXAGONAL = { b: 1, angle_gamma: 60 };

const CASES: readonly Case[] = [
  {
    samples: SYMMETRY_SAMPLES,
    id: 'latticeRotation',
    symmetric: { theta: 90 },
    other: { theta: 45 },
  },
  {
    samples: SYMMETRY_SAMPLES,
    id: 'latticeRotation',
    symmetric: { ...HEXAGONAL, theta: 60 },
    other: { ...HEXAGONAL, theta: 90 },
  },
  {
    samples: SYMMETRY_SAMPLES,
    id: 'latticeMirror',
    symmetric: { phi: 90, t: 1 },
    other: { phi: 30, t: 1 },
  },
  {
    samples: SYMMETRY_SAMPLES,
    id: 'latticeTranslation',
    symmetric: { s: 1, t: -1 },
    other: { s: 0.5, t: 0 },
  },
  {
    samples: SPACE_SYMMETRY_SAMPLES,
    id: 'cubicRotation',
    symmetric: { k: 0, theta: 90 },
    other: { k: 0, theta: 45 },
  },
  {
    samples: SPACE_SYMMETRY_SAMPLES,
    id: 'cubicRotation',
    symmetric: { k: 1, theta: 120 },
    other: { k: 1, theta: 90 },
  },
  {
    samples: SPACE_SYMMETRY_SAMPLES,
    id: 'cubicRotation',
    symmetric: { k: 2, theta: 180 },
    other: { k: 2, theta: 90 },
  },
  {
    samples: SPACE_SYMMETRY_SAMPLES,
    id: 'cubicMirror',
    symmetric: { psi: 45, t: 1 },
    other: { psi: 30, t: 1 },
  },
  {
    samples: SPACE_SYMMETRY_SAMPLES,
    id: 'latticeInversion',
    symmetric: { t: 1 },
    other: { t: 0.3 },
  },
];

describe('格子の対称性の見本は，対称操作の値でだけ，2つの格子が重なる', () => {
  it.each(CASES)('$id：$symmetric', ({ samples, id, symmetric, other }) => {
    const scene = sample(samples, id);
    expect(coincide(withValues(scene, symmetric))).toBe(true);
    expect(coincide(withValues(scene, other))).toBe(false);
  });

  it.each([
    [SYMMETRY_SAMPLES, 'latticeMirror'],
    [SPACE_SYMMETRY_SAMPLES, 'cubicMirror'],
    [SPACE_SYMMETRY_SAMPLES, 'latticeInversion'],
  ] as const)('縮んで潰れる途中の値(t = 0.5)でも描ける：%#', (samples, id) => {
    const scene = withValues(sample(samples, id), { t: 0.5 });
    expect(dotsByColor(scene).size).toBe(2);
  });
});
