import { describe, expect, it } from 'vitest';

import { emptyDraft } from './draft';
import type { SceneDraft } from './draft';
import {
  draggablePoints,
  lengthToCm,
  mathFromSvg,
  pointNear,
  svgFromMath,
  withPointAt,
} from './point-drag';

function planeDraft(objects: SceneDraft['objects']): SceneDraft {
  return {
    ...emptyDraft('plane'),
    view: { x: [-5, 5], y: [-5, 5], unit: { x: '2cm', y: '10mm' } },
    objects,
  };
}

describe('点のドラッグ', () => {
  it('長さの単位をcmにする', () => {
    expect(lengthToCm('2cm')).toBe(2);
    expect(lengthToCm('10mm')).toBe(1);
    expect(lengthToCm('72.27pt')).toBeCloseTo(2.54);
    expect(lengthToCm('2in')).toBeUndefined();
  });

  it('動かせるのは，位置を2つの数で書き，変換を持たない点だけである', () => {
    const draft = planeDraft([
      { id: 'A', type: 'point', at: [1, 2] },
      { id: 'B', type: 'point', at: 'A + A' },
      { id: 'C', type: 'point', at: ['t', 0] },
      { id: 'D', type: 'point', at: [0, 0], transform: [{ translate: [1, 0] }] },
      { id: 'E', type: 'point', at: [3, 4], transform: [] },
      { id: 'l', type: 'label', at: [0, 0], tex: 'x' },
    ]);
    expect(draggablePoints(draft)).toEqual([
      { id: 'A', at: [1, 2] },
      { id: 'E', at: [3, 4] },
    ]);
  });

  it('空間の図の点は動かさない', () => {
    const draft = { ...emptyDraft('space'), objects: [{ id: 'A', type: 'point', at: [1, 2] }] };
    expect(draggablePoints(draft)).toEqual([]);
  });

  it('SVGの座標(cm，yは下向き)と数学の座標を行き来する', () => {
    const draft = planeDraft([]);
    expect(svgFromMath(draft, [1, 2])).toEqual([2, -2]);
    expect(mathFromSvg(draft, [2, -2])).toEqual([1, 2]);
  });

  it('近くの点のうち，いちばん近いものを選ぶ', () => {
    const draft = planeDraft([
      { id: 'A', type: 'point', at: [0, 0] },
      { id: 'B', type: 'point', at: [0.2, 0] },
    ]);
    // Bは，SVGの(0.4, 0)にある．
    expect(pointNear(draft, [0.35, 0], 0.3)?.id).toBe('B');
    expect(pointNear(draft, [0.1, 0], 0.3)?.id).toBe('A');
    expect(pointNear(draft, [2, 2], 0.3)).toBeUndefined();
  });

  it('動かした位置は，見える範囲の幅の1000分の1に丸めて，その点にだけ書き込む', () => {
    const draft = planeDraft([
      { id: 'A', type: 'point', at: [0, 0], dot: true },
      { id: 'B', type: 'point', at: 'A + A' },
    ]);
    const moved = withPointAt(draft, 'A', [1.23456, -0.30001]);
    expect(moved.objects[0]).toEqual({ id: 'A', type: 'point', at: [1.23, -0.3], dot: true });
    expect(moved.objects[1]).toBe(draft.objects[1]);
    // 式で決まる点は，名前を指しても書き換えない．
    expect(withPointAt(draft, 'B', [1, 1]).objects[1]).toBe(draft.objects[1]);
  });
});
