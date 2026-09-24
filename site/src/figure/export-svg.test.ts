import { readFileSync } from 'node:fs';

import { describe, expect, it } from 'vitest';

import type { Figure } from '@/wasm/figure';

import { LIGHT_COLORS, standaloneSvg } from './export-svg';
import type { LabelSvg } from './label-svg';

const FIGURE: Figure = {
  description: '試験の図',
  bounds: { min: [-1, -1], max: [3, 2] },
  items: [
    {
      type: 'path',
      points: [
        [0, 0],
        [1, 1],
      ],
      stroke: { line: 'solid', width: 0.8, color: 'red' },
      arrow: null,
    },
    { type: 'label', at: [3, 0], anchor: 'west', tex: '$x$' },
  ],
};

/** 幅1em，高さ1em(基準線の上0.75em，下0.25em)のラベル． */
const LABEL: LabelSvg = {
  content: '<g data-mml-node="math"><path d="M0 0L1000 0"></path></g>',
  viewBox: '0 -750 1000 1000',
  width: 1,
  height: 1,
};

/** 10ptの1emの長さ(cm)． */
const EM = (10 * 2.54) / 72.27;
/** ラベルの箱の内側の余白(em)． */
const PADDING = 0.3333;

/** SVGに書く数(小数点以下5桁，末尾の0を除く)． */
function written(value: number): string {
  return String(Number(value.toFixed(5)));
}

describe('standaloneSvg', () => {
  const file = standaloneSvg(FIGURE, [LABEL], { transparent: false });

  it('XMLの宣言と，名前空間を持つ，単独のSVGにする', () => {
    expect(file.svg).toMatch(/^<\?xml version="1\.0" encoding="UTF-8"\?>\n<svg /u);
    expect(file.svg).toContain('xmlns="http://www.w3.org/2000/svg"');
    expect(file.svg).toContain('aria-label="試験の図"');
  });

  it('色の変数を，明るい背景の色にし，数式の色の規則を持つ', () => {
    expect(file.svg).not.toContain('var(');
    expect(file.svg).toContain('stroke="#c62828"');
    expect(file.svg).toContain('.math-color-red{color:#c62828}');
  });

  it('右にはみ出したラベルの箱を含めた範囲を，実寸(cm)の大きさにする', () => {
    // 箱の幅は，1emと，両側の余白0.3333emずつである．左端は，位置のx=3である．
    const right = 3 + (1 + 2 * PADDING) * EM;
    expect(file.width).toBeCloseTo(right + 1, 4);
    expect(file.height).toBeCloseTo(3, 4);
    expect(file.svg).toContain(`width="${written(right + 1)}cm"`);
  });

  it('ラベルは，余白の内側に，MathJaxの中身と範囲のまま置く', () => {
    expect(file.svg).toContain('viewBox="0 -750 1000 1000"');
    expect(file.svg).toContain(LABEL.content);
    // 西のアンカーなので，箱の左端がx=3，縦の真ん中がy=0である．
    expect(file.svg).toContain(`x="${written(3 + PADDING * EM)}"`);
  });

  it('透過しないときは背景を白く塗り，透過するときは塗らない', () => {
    expect(file.svg).toContain('fill="#ffffff"');
    const clear = standaloneSvg(FIGURE, [LABEL], { transparent: true });
    expect(clear.svg).not.toContain('<rect');
  });

  it('ラベルの数が図と合わなければ，失敗にする', () => {
    expect(() => standaloneSvg(FIGURE, [], { transparent: true })).toThrow();
  });
});

describe('LIGHT_COLORS', () => {
  it('figure.cssの明るい背景の色と同じである', () => {
    const css = readFileSync(new URL('../styles/figure.css', import.meta.url), 'utf8');
    const root = css.slice(css.indexOf(':root {'), css.indexOf('}'));
    for (const [name, color] of Object.entries(LIGHT_COLORS)) {
      expect(root).toContain(`--figure-${name}: ${color};`);
    }
  });
});
