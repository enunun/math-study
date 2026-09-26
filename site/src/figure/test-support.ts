import { readFileSync } from 'node:fs';

import type { Element, ElementContent } from 'hast';
import { expect } from 'vitest';

import { initSync, renderScene } from '@/wasm/figure';
import type { AnchorName, LabelItem } from '@/wasm/figure';

import { CM_PER_PT, figureToHast, svgPoint } from './svg';

/** 木の中から，指定した名前の要素を，文書の順にすべて集める． */
function all(root: Element, tagName: string): Element[] {
  const found: Element[] = [];
  const walk = (node: ElementContent): void => {
    if (node.type !== 'element') {
      return;
    }
    if (node.tagName === tagName) {
      found.push(node);
    }
    for (const child of node.children) {
      walk(child);
    }
  };
  for (const child of root.children) {
    walk(child);
  }
  return found;
}

function classes(element: Element): string[] {
  const { className } = element.properties;
  return Array.isArray(className) ? className.map(String) : [];
}

// 図を描く処理を，テストを集める段階で呼ぶため，beforeAllではなく，読み込みの時点で初期化する．
initSync({ module: readFileSync(new URL('../wasm/figure_bg.wasm', import.meta.url)) });

type Point = [number, number];

interface Box {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface Drawn {
  svg: Element;
  paths: Element[];
  polygons: Element[];
  circles: Element[];
  labels: LabelItem[];
}

/** 位置の許容の誤差(cm)．0.05cmは，画面の約2pxである． */
const TOLERANCE = 0.05;

function draw(name: string): Drawn {
  const json = readFileSync(new URL(`../figures/${name}.json`, import.meta.url), 'utf8');
  const outcome = renderScene(json);
  if (outcome.status !== 'ok') {
    throw new Error(`図「${name}」を描けない：${outcome.message}`);
  }
  const root = figureToHast(outcome.figure);
  const [svg] = all(root, 'svg');
  if (svg === undefined) {
    throw new Error(`図「${name}」にSVGがない`);
  }
  return {
    svg,
    paths: all(root, 'path'),
    polygons: all(root, 'polygon'),
    circles: all(root, 'circle'),
    labels: outcome.figure.items.filter((item): item is LabelItem => item.type === 'label'),
  };
}

function nth<T>(items: T[], index: number): T {
  const found = items[index];
  if (found === undefined) {
    throw new Error(`${index}番目の要素がない`);
  }
  return found;
}

/** パスの頂点．`M x yL x y…`の形を読む． */
function points(path: Element): Point[] {
  const numbers = [...String(path.properties.d).matchAll(/-?[\d.]+(?:e-?\d+)?/gu)].map(Number);
  const found: Point[] = [];
  for (let index = 0; index + 1 < numbers.length; index += 2) {
    found.push([nth(numbers, index), nth(numbers, index + 1)]);
  }
  return found;
}

/** 線幅を含まない，パスの外接する箱． */
function bounds(path: Element): Box {
  const xs = points(path).map(([x]) => x);
  const ys = points(path).map(([, y]) => y);
  const x = Math.min(...xs);
  const y = Math.min(...ys);
  return { x, y, width: Math.max(...xs) - x, height: Math.max(...ys) - y };
}

function center({ x, y, width, height }: Box): Point {
  return [x + width / 2, y + height / 2];
}

function dashed(path: Element): boolean {
  return path.properties.strokeDasharray !== undefined;
}

/** 線の太さ(pt)． */
function widthPt(path: Element): number {
  return Number(path.properties.strokeWidth) / CM_PER_PT;
}

/** 矢じりの先端．多角形の最初の点である． */
function tip(polygon: Element): Point {
  const [first = ''] = String(polygon.properties.points).split(' ');
  const [x = Number.NaN, y = Number.NaN] = first.split(',').map(Number);
  return [x, y];
}

function circleCenter(circle: Element): Point {
  return [Number(circle.properties.cx), Number(circle.properties.cy)];
}

function distance([ax, ay]: Point, [bx, by]: Point): number {
  return Math.hypot(ax - bx, ay - by);
}

/** 名前の文字が`tex`のラベルの，SVGの座標での位置と，アンカー． */
function label(drawn: Drawn, tex: string): { at: Point; anchor: AnchorName } {
  const found = drawn.labels.find((item) => item.tex === tex);
  if (found === undefined) {
    throw new Error(`ラベル「${tex}」がない`);
  }
  return { at: svgPoint(found.at), anchor: found.anchor };
}

function expectNear(actual: Point, expected: Point): void {
  expect(distance(actual, expected), `${actual.join(',')}と${expected.join(',')}`).toBeLessThan(
    TOLERANCE,
  );
}

/** パスの両端のうち，`pick`が最初の点を選べば最初の点，そうでなければ最後の点． */
function end(path: Element, pick: (first: Point, last: Point) => boolean): Point {
  const vertices = points(path);
  const [first] = vertices;
  const last = vertices.at(-1);
  if (first === undefined || last === undefined) {
    throw new Error('端を持たないパス');
  }
  return pick(first, last) ? first : last;
}

const leftEnd = (path: Element): Point => end(path, ([ax], [bx]) => ax < bx);
const bottomEnd = (path: Element): Point => end(path, ([, ay], [, by]) => ay > by);

export type { Box, Drawn, Point };
export {
  TOLERANCE,
  all,
  bottomEnd,
  bounds,
  center,
  circleCenter,
  classes,
  dashed,
  distance,
  draw,
  expectNear,
  label,
  leftEnd,
  nth,
  points,
  tip,
  widthPt,
};
