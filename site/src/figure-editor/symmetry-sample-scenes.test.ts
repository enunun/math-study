import { readFile } from 'node:fs/promises';

import { beforeAll, describe, expect, it } from 'vitest';

import { initSync, renderScene } from '@/wasm/figure';

import { stringifyDraft } from './draft';
import type { SceneDraft } from './draft';
import type { JsonObject } from './json';
import { SPACE_SYMMETRY_SAMPLES } from './space-symmetry-sample-scenes';
import { SYMMETRY_SAMPLES } from './symmetry-sample-scenes';
import type { SceneTemplate } from './template-types';

beforeAll(async () => {
  const module = await readFile(new URL('../wasm/figure_bg.wasm', import.meta.url));
  initSync({ module });
});

function sample(samples: readonly SceneTemplate[], id: string): SceneDraft {
  const found = samples.find((candidate) => candidate.id === id);
  if (found === undefined) {
    throw new Error(`見本「${id}」がない`);
  }
  return found.scene;
}

/** 媒介変数の値を書き換えた図． */
function withValues(scene: SceneDraft, values: Readonly<Record<string, number>>): SceneDraft {
  const objects: JsonObject[] = [];
  for (const object of scene.objects) {
    const value = typeof object.id === 'string' ? values[object.id] : undefined;
    objects.push(value === undefined ? object : { ...object, value });
  }
  return { ...scene, objects };
}

/** 丸めた点の印の位置を，色ごとに集めたもの．灰色は元の組，青は変換した組である． */
function dotsByColor(scene: SceneDraft): Map<string, string[]> {
  const outcome = renderScene(stringifyDraft(scene));
  if (outcome.status !== 'ok') {
    throw new Error(JSON.stringify(outcome));
  }
  const groups = new Map<string, string[]>();
  for (const item of outcome.figure.items) {
    if (item.type === 'dot') {
      const key = item.color ?? 'none';
      const at = item.at.map((value) => (Math.abs(value) < 1e-6 ? 0 : value).toFixed(4)).join(',');
      groups.set(key, [...(groups.get(key) ?? []), at]);
    }
  }
  return groups;
}

/** 元の組と変換した組の，見えている格子点が一致するか． */
function coincide(scene: SceneDraft): boolean {
  const groups = dotsByColor(scene);
  const fixed = [...(groups.get('gray') ?? [])].toSorted();
  const moved = [...(groups.get('blue') ?? [])].toSorted();
  expect(fixed.length).toBeGreaterThan(0);
  return fixed.length === moved.length && fixed.every((at, index) => at === moved[index]);
}

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
