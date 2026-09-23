import { describe, expect, it } from 'vitest';

import { emptyDraft } from './draft';
import { insertObjects } from './insert-template';

describe('テンプレートのオブジェクトの挿入', () => {
  it('識別子を，今の図と重ならないものへ付け替える', () => {
    const draft = {
      ...emptyDraft('plane'),
      objects: [{ id: 'point1', type: 'point', at: [0, 0] }],
    };
    const inserted = insertObjects(draft, 'plane', [
      { id: 'p1', type: 'point', at: [1, 0] },
      { id: 'p2', type: 'point', at: [0, 1] },
    ]);
    const ids = inserted.objects.map((object) => object.id);
    expect(ids).toEqual(['point1', 'point2', 'point3']);
  });

  it('グループの中でのお互いの参照(線分の両端など)を，付け替えた先に合わせて書き換える', () => {
    const draft = emptyDraft('plane');
    const inserted = insertObjects(draft, 'plane', [
      { id: 'p1', type: 'point', at: [0, 0] },
      { id: 'p2', type: 'point', at: [1, 0] },
      { id: 's1', type: 'segment', from: 'p1', to: 'p2' },
    ]);
    const segment = inserted.objects.find((object) => object.type === 'segment');
    const points = inserted.objects.filter((object) => object.type === 'point');
    expect(segment?.from).toBe(points[0]?.id);
    expect(segment?.to).toBe(points[1]?.id);
  });

  it('グループの外にある既存の点への参照は，そのまま残す', () => {
    const draft = {
      ...emptyDraft('plane'),
      objects: [{ id: 'anchor', type: 'point', at: [0, 0] }],
    };
    const inserted = insertObjects(draft, 'plane', [
      { id: 'p1', type: 'point', at: [1, 1] },
      { id: 's1', type: 'segment', from: 'anchor', to: 'p1' },
    ]);
    const segment = inserted.objects.find((object) => object.type === 'segment');
    expect(segment?.from).toBe('anchor');
  });

  it('2回挿入しても，識別子は重ならない', () => {
    const draft = emptyDraft('plane');
    const once = insertObjects(draft, 'plane', [{ id: 'p1', type: 'point', at: [0, 0] }]);
    const twice = insertObjects(once, 'plane', [{ id: 'p1', type: 'point', at: [1, 1] }]);
    const ids = twice.objects.map((object) => object.id);
    expect(new Set(ids).size).toBe(ids.length);
  });
});
