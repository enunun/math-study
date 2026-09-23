import { describe, expect, it } from 'vitest';

import { stringOf } from './json';
import type { JsonObject } from './json';
import {
  CUBE_VERTICES,
  DODECAHEDRON_VERTICES,
  ICOSAHEDRON_VERTICES,
  OCTAHEDRON_VERTICES,
  TETRAHEDRON_VERTICES,
  nearestNeighborEdges,
  polyhedronObjects,
  regularPolygonObjects,
} from './regular-shapes';

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
    for (const length of lengths) {
      expect(length).toBeCloseTo(first ?? 0, 6);
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

describe('正多面体の頂点と稜', () => {
  const POLYHEDRA = [
    { name: '正4面体', vertices: TETRAHEDRON_VERTICES, vertexCount: 4, edgeCount: 6 },
    { name: '正6面体', vertices: CUBE_VERTICES, vertexCount: 8, edgeCount: 12 },
    { name: '正8面体', vertices: OCTAHEDRON_VERTICES, vertexCount: 6, edgeCount: 12 },
    { name: '正20面体', vertices: ICOSAHEDRON_VERTICES, vertexCount: 12, edgeCount: 30 },
    { name: '正12面体', vertices: DODECAHEDRON_VERTICES, vertexCount: 20, edgeCount: 30 },
  ];

  it.each(POLYHEDRA)('$nameは，頂点$vertexCount個，稜$edgeCount本である', (spec) => {
    const objects = polyhedronObjects(spec.vertices);
    const points = objects.filter((object) => object.type === 'point');
    const segments = objects.filter((object) => object.type === 'segment');
    expect(points).toHaveLength(spec.vertexCount);
    expect(segments).toHaveLength(spec.edgeCount);
  });

  it('どの頂点も，つながる稜の本数(次数)が等しい(正多面体の対称性)', () => {
    const vertices = ICOSAHEDRON_VERTICES;
    const edges = nearestNeighborEdges(vertices);
    const degree = Array.from({ length: vertices.length }, () => 0);
    for (const [from, to] of edges) {
      degree[from] = (degree[from] ?? 0) + 1;
      degree[to] = (degree[to] ?? 0) + 1;
    }
    expect(new Set(degree)).toEqual(new Set([5]));
  });
});
