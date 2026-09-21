import type { Element, ElementContent, Properties } from 'hast';

import type {
  ArrowHead,
  ColorName,
  DotItem,
  FillItem,
  Figure,
  LabelItem,
  PathItem,
} from '@/wasm/figure';

import { anchorShift, labelToMath } from './label';

/** 1インチの長さ(cm)． */
const CM_PER_INCH = 2.54;
/** 1インチの長さ(pt)．TeXのptで，72.27である． */
const PT_PER_INCH = 72.27;
/** 1ptの長さ(cm)． */
const CM_PER_PT = CM_PER_INCH / PT_PER_INCH;
/** TikZの`dotted`の，点の間隔(pt)．点の長さは，線幅である． */
const DOTTED_GAP = 2;
/** TikZの`dashed`の，線と間隔の長さ(pt)． */
const DASH_LENGTH = 3;
/** 数を書く小数点以下の桁数． */
const DIGITS = 5;
/** 割合(%)を書く小数点以下の桁数． */
const PERCENT_DIGITS = 4;
/** 割合を，百分率にする倍率． */
const PERCENT = 100;

/** 末尾の0を省いた小数にする．-0は0にする． */
function format(value: number, digits = DIGITS): string {
  const text = value.toFixed(digits).replace(/\.?0+$/u, '');
  return text === '-0' || text === '' ? '0' : text;
}

function element(
  tagName: string,
  properties: Properties = {},
  children: ElementContent[] = [],
): Element {
  return { type: 'element', tagName, properties, children };
}

/** ptの長さを，cmの文字列にする． */
function cm(points: number): string {
  return format(points * CM_PER_PT);
}

/** yの向きを反転して，SVGの座標にした点． */
function svgPoint([x, y]: [number, number]): [number, number] {
  return [x, -y];
}

/** 線の色．色の名前は，CSSの変数(`figure.css`)に置き換え，明るい背景と暗い背景で色を変える．色がなければ，文字の色に従う． */
function strokeColor(color: ColorName | null): string {
  return color === null ? 'currentColor' : `var(--figure-${color})`;
}

/** 線の種類ごとの，破線の指定．実線には，指定がない． */
function dashArray({ line, width }: PathItem['stroke']): string | undefined {
  if (line === 'dotted') {
    return `${cm(width)} ${cm(DOTTED_GAP)}`;
  }
  return line === 'dashed' ? `${cm(DASH_LENGTH)} ${cm(DASH_LENGTH)}` : undefined;
}

/** 折れ線のパスのデータ．矢じりのある線は，矢じりの手前で止める． */
function pathData(item: PathItem): string {
  const points = item.points.map((point) => svgPoint(point));
  if (item.arrow !== null) {
    points.splice(-1, 1, svgPoint(item.arrow.line_end));
  }
  return points
    .map(([x, y], index) => `${index === 0 ? 'M' : 'L'}${format(x)} ${format(y)}`)
    .join('');
}

/** 矢じりの輪郭．塗って縁取る．角は，TikZと同じく，とがらせる． */
function arrowElement(arrow: ArrowHead, color: ColorName | null): Element {
  const paint = strokeColor(color);
  return element('polygon', {
    points: arrow.polygon
      .map((point) => svgPoint(point))
      .map(([x, y]) => `${format(x)},${format(y)}`)
      .join(' '),
    fill: paint,
    stroke: paint,
    strokeWidth: cm(arrow.line_width),
    strokeLinejoin: 'miter',
    strokeMiterlimit: '10',
  });
}

/** 折れ線と，矢じりがあれば，その輪郭． */
function pathElements(item: PathItem): Element[] {
  const properties: Properties = {
    d: pathData(item),
    fill: 'none',
    stroke: strokeColor(item.stroke.color),
    strokeWidth: cm(item.stroke.width),
  };
  const dashes = dashArray(item.stroke);
  if (dashes !== undefined) {
    properties.strokeDasharray = dashes;
  }
  return [
    element('path', properties),
    ...(item.arrow === null ? [] : [arrowElement(item.arrow, item.stroke.color)]),
  ];
}

/** 塗った多角形．線は引かず，色の変数と不透明度で塗る． */
function fillElement(item: FillItem): Element {
  const commands = item.points
    .map((point) => svgPoint(point))
    .map(([x, y], index) => `${index === 0 ? 'M' : 'L'}${format(x)} ${format(y)}`)
    .join('');
  return element('path', {
    d: `${commands}Z`,
    fill: strokeColor(item.color),
    fillOpacity: format(item.opacity),
    stroke: 'none',
  });
}

/** 点の印．塗った丸である． */
function dotElement(item: DotItem): Element {
  const [x, y] = svgPoint(item.at);
  return element('circle', {
    cx: format(x),
    cy: format(y),
    r: cm(item.radius),
    fill: strokeColor(item.color),
  });
}

/** 位置を割合で置き，アンカーの分だけ箱をずらした，ラベル．数式は，後段のMathJaxが描画する． */
function labelElement(item: LabelItem, figure: Figure): Element {
  const { min, max } = figure.bounds;
  const left = ((item.at[0] - min[0]) / (max[0] - min[0])) * PERCENT;
  const top = ((max[1] - item.at[1]) / (max[1] - min[1])) * PERCENT;
  const shift = anchorShift(item.anchor);
  const style = [
    `left: ${format(left, PERCENT_DIGITS)}%`,
    `top: ${format(top, PERCENT_DIGITS)}%`,
    `transform: translate(${format(-shift.x * PERCENT, PERCENT_DIGITS)}%, ${format(-shift.y * PERCENT, PERCENT_DIGITS)}%)`,
  ].join('; ');
  return element('span', { className: ['figure-label'], style, ariaHidden: 'true' }, [
    element('code', { className: ['language-math', 'math-inline'] }, [
      { type: 'text', value: labelToMath(item.tex) },
    ]),
  ]);
}

/**
 * 中間表現を，HTMLの木にする．`figure`の中に，SVG(線と矢じり)と，その上に重ねるラベルを置く．
 * ラベルの数式は，行内の数式(`code.language-math`)として置き，MathJaxのプラグインが描画する．
 */
function figureToHast(figure: Figure): Element {
  const { min, max } = figure.bounds;
  const width = max[0] - min[0];
  const height = max[1] - min[1];
  const drawn = figure.items.flatMap((item) => {
    if (item.type === 'path') {
      return pathElements(item);
    }
    if (item.type === 'fill') {
      return [fillElement(item)];
    }
    return item.type === 'dot' ? [dotElement(item)] : [];
  });
  const labels = figure.items.flatMap((item) =>
    item.type === 'label' ? [labelElement(item, figure)] : [],
  );
  const svg = element(
    'svg',
    {
      viewBox: `${format(min[0])} ${format(-max[1])} ${format(width)} ${format(height)}`,
      role: 'img',
      ariaLabel: figure.description,
    },
    drawn,
  );
  return element(
    'figure',
    { className: ['figure', 'not-content'], style: `--figure-width: ${format(width)}cm` },
    [element('div', { className: ['figure-frame'] }, [svg, ...labels])],
  );
}

export { figureToHast };
