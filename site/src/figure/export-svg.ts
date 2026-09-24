import type { Element, ElementContent, RootContent } from 'hast';
import { fromHtml } from 'hast-util-from-html';
import { toHtml } from 'hast-util-to-html';

import type { ColorName, Figure, LabelItem } from '@/wasm/figure';

import { anchorShift } from './label';
import type { LabelSvg } from './label-svg';
import { CM_PER_PT, drawnElements, element, format, svgPoint } from './svg';

/**
 * 図を，単独で開けるSVGのファイルにする．ページの図(`svg.ts`)は，色をCSSの変数で持ち，ラベルをHTMLで重ねるが，
 * ファイルでは，色を明るい背景の色に決め，ラベルを，MathJaxのSVG(`label-svg.ts`)として，同じSVGの中に置く．
 * PNGやJPEGは，このSVGを，ブラウザで画素にして作る．
 */

/** 線の色の名前ごとの，明るい背景での色．`figure.css`の`:root`と同じである(`export-svg.test.ts`が確かめる)． */
const LIGHT_COLORS: Record<ColorName, string> = {
  gray: '#8a8f98',
  red: '#c62828',
  blue: '#1565c0',
  green: '#2e7d32',
  orange: '#e65100',
  purple: '#6a1b9a',
};
/** 色の名前がない線と，ラベルの文字の色． */
const TEXT_COLOR = '#000000';
/** 透過しないときの背景の色． */
const BACKGROUND_COLOR = '#ffffff';
/** ラベルの文字の大きさ(pt)．ページの図(`figure.css`)とTikZの節点と同じ，10ptである． */
const LABEL_FONT_PT = 10;
/** ラベルの箱の内側の余白(em)．TikZの節点の内側の余白(`inner sep`の既定，0.3333em)と同じである． */
const LABEL_PADDING_EM = 0.3333;
/** 余白は，箱の両側にある． */
const BOTH_SIDES = 2;

interface SvgOptions {
  /** 背景を塗らずに，透明のままにするか． */
  transparent: boolean;
}

/** 長方形．左上の点と，幅と高さ(cm，yは下向き)． */
interface Box {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** ラベルの箱．アンカーの点が，位置に来るように置く． */
function labelBox(item: LabelItem, label: LabelSvg): Box {
  const em = LABEL_FONT_PT * CM_PER_PT;
  const width = (label.width + BOTH_SIDES * LABEL_PADDING_EM) * em;
  const height = (label.height + BOTH_SIDES * LABEL_PADDING_EM) * em;
  const [x, y] = svgPoint(item.at);
  const shift = anchorShift(item.anchor);
  return { x: x - shift.x * width, y: y - shift.y * height, width, height };
}

function isElementContent(node: RootContent): node is ElementContent {
  return node.type !== 'doctype';
}

/** 箱の中に，余白を除いて，MathJaxのSVGを置く．中身は，SVGとして読み直し，XMLとして正しく書き出す． */
function labelElement(box: Box, label: LabelSvg): Element {
  const padding = LABEL_PADDING_EM * LABEL_FONT_PT * CM_PER_PT;
  const content = fromHtml(label.content, { fragment: true, space: 'svg' });
  return element(
    'svg',
    {
      x: format(box.x + padding),
      y: format(box.y + padding),
      width: format(box.width - BOTH_SIDES * padding),
      height: format(box.height - BOTH_SIDES * padding),
      viewBox: label.viewBox,
      overflow: 'visible',
    },
    content.children.filter((node) => isElementContent(node)),
  );
}

/** 図の範囲に，はみ出したラベルの箱を加えた範囲． */
function extent(figure: Figure, boxes: readonly Box[]): Box {
  const { min, max } = figure.bounds;
  const left = Math.min(min[0], ...boxes.map((box) => box.x));
  const top = Math.min(-max[1], ...boxes.map((box) => box.y));
  const right = Math.max(max[0], ...boxes.map((box) => box.x + box.width));
  const bottom = Math.max(-min[1], ...boxes.map((box) => box.y + box.height));
  return { x: left, y: top, width: right - left, height: bottom - top };
}

/** 数式の一部の色(`\colored`)の規則．ページでは`figure.css`が決める色を，ファイルの中に書く． */
function colorStyle(): string {
  return Object.entries(LIGHT_COLORS)
    .map(([name, color]) => `.math-color-${name}{color:${color}}`)
    .join('');
}

/** CSSの変数で書いた色(`var(--figure-red)`)を，色の値にする． */
function resolveColors(svg: string): string {
  const colors = new Map(Object.entries(LIGHT_COLORS));
  return svg.replaceAll(
    /var\(--figure-(?<name>[a-z]+)\)/gu,
    (text, name: string) => colors.get(name) ?? text,
  );
}

/** 透過しないときの，範囲いっぱいの白い長方形． */
function backgroundElements(area: Box, transparent: boolean): Element[] {
  if (transparent) {
    return [];
  }
  return [
    element('rect', {
      x: format(area.x),
      y: format(area.y),
      width: format(area.width),
      height: format(area.height),
      fill: BACKGROUND_COLOR,
    }),
  ];
}

/** 画像の大きさ(cm)．画素にするときの大きさを決めるのに使う． */
interface SvgFile {
  svg: string;
  width: number;
  height: number;
}

/**
 * 中間表現と，組んだラベル(図の中のラベルの順)から，SVGのファイルの中身を作る．大きさは，図の実寸(cm)である．
 */
function standaloneSvg(
  figure: Figure,
  labels: readonly LabelSvg[],
  { transparent }: SvgOptions,
): SvgFile {
  const labelItems = figure.items.filter((item) => item.type === 'label');
  if (labelItems.length !== labels.length) {
    throw new Error(`ラベルの数が合わない：${labelItems.length}個と${labels.length}個`);
  }
  const placed = labelItems.flatMap((item, index) => {
    const label = labels[index];
    return label === undefined ? [] : [{ box: labelBox(item, label), label }];
  });
  const area = extent(
    figure,
    placed.map(({ box }) => box),
  );
  const root = element(
    'svg',
    {
      xmlns: 'http://www.w3.org/2000/svg',
      width: `${format(area.width)}cm`,
      height: `${format(area.height)}cm`,
      viewBox: [area.x, area.y, area.width, area.height].map((value) => format(value)).join(' '),
      role: 'img',
      ariaLabel: figure.description,
      color: TEXT_COLOR,
    },
    [
      element('style', {}, [{ type: 'text', value: colorStyle() }]),
      ...backgroundElements(area, transparent),
      ...drawnElements(figure),
      ...placed.map(({ box, label }) => labelElement(box, label)),
    ],
  );
  const svg = toHtml(root, { space: 'svg' });
  return {
    svg: `<?xml version="1.0" encoding="UTF-8"?>\n${resolveColors(svg)}\n`,
    width: area.width,
    height: area.height,
  };
}

export { BACKGROUND_COLOR, LIGHT_COLORS, standaloneSvg };
export type { SvgFile, SvgOptions };
