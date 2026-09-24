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

describe('像と写像を含むテンプレートの挿入', () => {
  it('像の元(of)と，変換の手順の写像(map)を，付け替えた識別子に書き換える', () => {
    const draft = {
      ...emptyDraft('plane'),
      objects: [{ id: 'map1', type: 'map', vars: ['x', 'y'], expr: ['x', 'y'] }],
    };
    const inserted = insertObjects(draft, 'plane', [
      { id: 'F', type: 'map', vars: ['x', 'y'], expr: ['2*x', 'y'] },
      { id: 'c', type: 'curve', var: 't', expr: ['t', '0'], domain: [0, 1] },
      { id: 'd', type: 'image', of: 'c', transform: [{ rotate: 90 }, { map: 'F' }] },
    ]);
    const [, map, curve, image] = inserted.objects;
    expect(map?.id).toBe('map2');
    expect(image?.of).toBe(curve?.id);
    expect(image?.transform).toEqual([{ rotate: 90 }, { map: 'map2' }]);
  });

  it('変換を持たないオブジェクトに，空の変換を足さない', () => {
    const inserted = insertObjects(emptyDraft('plane'), 'plane', [
      { id: 'p', type: 'point', at: [0, 0] },
    ]);
    expect(inserted.objects[0]).not.toHaveProperty('transform');
  });
});
