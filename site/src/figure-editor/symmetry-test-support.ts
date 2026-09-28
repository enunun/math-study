import { expect } from 'vitest';

import { renderScene } from '@/wasm/figure';

import { stringifyDraft } from './draft';
import type { SceneDraft } from './draft';
import type { JsonObject } from './json';
import type { SceneTemplate } from './template-types';

/** 同じ位置とみなす座標の誤差と，比べる桁数． */
const NEAR_ZERO = 1e-6;
const DIGITS = 4;

/**
 * 格子の対称性の見本のテストの道具．見本の媒介変数を書き換えて描き，灰色の組(元の格子)と青の組(変換した格子)の
 * 点の印が重なるかを調べる．Wasmは，使うテストの側で読み込んでおく．
 */

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
      const at = item.at
        .map((value) => (Math.abs(value) < NEAR_ZERO ? 0 : value).toFixed(DIGITS))
        .join(',');
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

/** 青の点の印のうち，灰色の点の印と重なるものの割合．有限個の格子点が並進ではみ出す見本に使う． */
function matchedFraction(scene: SceneDraft): number {
  const groups = dotsByColor(scene);
  const fixed = new Set(groups.get('gray'));
  const moved = groups.get('blue') ?? [];
  expect(moved.length).toBeGreaterThan(0);
  return moved.filter((at) => fixed.has(at)).length / moved.length;
}

export { coincide, dotsByColor, matchedFraction, sample, withValues };
