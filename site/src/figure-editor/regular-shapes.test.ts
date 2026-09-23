import { describe, expect, it } from 'vitest';

import { convexHullFaces } from './convex-hull';
import { arrayOf, stringOf } from './json';
import type { JsonObject } from './json';
import {
  CUBE_VERTICES,
  DODECAHEDRON_VERTICES,
  ICOSAHEDRON_VERTICES,
  OCTAHEDRON_VERTICES,
  TETRAHEDRON_VERTICES,
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

/** 面の集まりから，重ならない稜(頂点の番号の組)の数を数える． */
function edgeCountOf(faces: readonly (readonly number[])[]): number {
  const edges = new Set<string>();
  for (const face of faces) {
    for (let i = 0; i < face.length; i += 1) {
      const a = face[i];
      const b = face[(i + 1) % face.length];
      edges.add(a < b ? `${a}-${b}` : `${b}-${a}`);
    }
  }
  return edges.size;
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

describe('正多面体の凸包の面', () => {
  const POLYHEDRA = [
    { name: '正4面体', vertices: TETRAHEDRON_VERTICES, vertexCount: 4, edgeCount: 6, faceSize: 3 },
    { name: '正6面体', vertices: CUBE_VERTICES, vertexCount: 8, edgeCount: 12, faceSize: 4 },
    { name: '正8面体', vertices: OCTAHEDRON_VERTICES, vertexCount: 6, edgeCount: 12, faceSize: 3 },
    {
      name: '正20面体',
      vertices: ICOSAHEDRON_VERTICES,
      vertexCount: 12,
      edgeCount: 30,
      faceSize: 3,
    },
    {
      name: '正12面体',
      vertices: DODECAHEDRON_VERTICES,
      vertexCount: 20,
      edgeCount: 30,
      faceSize: 5,
    },
  ];

  it.each(POLYHEDRA)(
    '$nameは，オイラーの公式(頂点-辺+面=2)を満たす面が求まる',
    ({ vertices, vertexCount, edgeCount }) => {
      const faces = convexHullFaces(vertices);
      expect(vertices).toHaveLength(vertexCount);
      expect(edgeCountOf(faces)).toBe(edgeCount);
      expect(vertexCount - edgeCount + faces.length).toBe(2);
    },
  );

  it.each(POLYHEDRA)(
    '$nameは，すべての面が同じ頂点数(正多面体)である',
    ({ vertices, faceSize }) => {
      const faces = convexHullFaces(vertices);
      for (const face of faces) {
        expect(face).toHaveLength(faceSize);
      }
    },
  );

  it('どの稜も，ちょうど2つの面に属する(閉じた立体になっている)', () => {
    const faces = convexHullFaces(ICOSAHEDRON_VERTICES);
    const counts = new Map<string, number>();
    for (const face of faces) {
      for (let i = 0; i < face.length; i += 1) {
        const a = face[i];
        const b = face[(i + 1) % face.length];
        const key = a < b ? `${a}-${b}` : `${b}-${a}`;
        counts.set(key, (counts.get(key) ?? 0) + 1);
      }
    }
    expect(new Set(counts.values())).toEqual(new Set([2]));
  });

  it('正4面体は，1個の複体(complex)オブジェクトにまとまる', () => {
    const objects = polyhedronObjects(TETRAHEDRON_VERTICES);
    expect(objects).toHaveLength(1);
    const [complex] = objects;
    expect(stringOf(complex, 'type')).toBe('complex');
    expect(arrayOf(complex, 'vertices')).toHaveLength(4);
    expect(arrayOf(complex, 'faces')).toHaveLength(4);
  });
});
