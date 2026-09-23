import { describe, expect, it } from 'vitest';

import { stringOf } from './json';
import type { JsonObject } from './json';
import { regularPolygonObjects } from './regular-shapes';

function isNumber(value: unknown): value is number {
  return typeof value === 'number';
}

/** オブジェクトの項目のうち，数だけでできた配列のもの．なければ空の配列． */
function numbersOf(object: JsonObject, key: string): number[] {
  const value = object[key];
  return Array.isArray(value) ? value.filter((item): item is number => isNumber(item)) : [];
}

function distance(a: readonly number[], b: readonly number[]): number {
  return Math.sqrt(a.reduce((sum, value, index) => sum + (value - (b[index] ?? 0)) ** 2, 0));
}

describe('正多角形の頂点と辺', () => {
  it.each([3, 4, 5, 6, 8, 10, 12])('正%d角形は，n個の点とn本の辺でできている', (n) => {
    const objects = regularPolygonObjects(n);
    const points = objects.filter((object) => object.type === 'point');
    const segments = objects.filter((object) => object.type === 'segment');
    expect(points).toHaveLength(n);
    expect(segments).toHaveLength(n);
  });

  it('辺の長さは，すべて等しい(正多角形になっている)', () => {
    const objects = regularPolygonObjects(6);
    const points = new Map(
      objects
        .filter((object) => object.type === 'point')
        .map((object) => [stringOf(object, 'id'), numbersOf(object, 'at')]),
    );
    const lengths = objects
      .filter((object) => object.type === 'segment')
      .map((segment) => {
        const from = points.get(stringOf(segment, 'from'));
        const to = points.get(stringOf(segment, 'to'));
        if (from === undefined || to === undefined) {
          throw new Error('参照する点がない');
        }
        return distance(from, to);
      });
    const [first] = lengths;
    for (const value of lengths) {
      expect(value).toBeCloseTo(first ?? 0, 6);
    }
  });

  it('辺は，隣り合う点を順につなぎ，最後の点は最初の点へ戻る', () => {
    const objects = regularPolygonObjects(4);
    const segments = objects.filter((object) => object.type === 'segment');
    expect(segments.map((segment) => [segment.from, segment.to])).toEqual([
      ['p1', 'p2'],
      ['p2', 'p3'],
      ['p3', 'p4'],
      ['p4', 'p1'],
    ]);
  });
});
