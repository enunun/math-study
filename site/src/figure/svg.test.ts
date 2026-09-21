import type { Element, ElementContent } from 'hast';
import { describe, expect, it } from 'vitest';

import type { Figure } from '@/wasm/figure';

import { figureToHast } from './svg';

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
      stroke: { line: 'solid', width: 0.8, color: null },
      arrow: null,
    },
    {
      type: 'path',
      points: [
        [0, 0],
        [1, 0],
        [2, 1],
      ],
      stroke: { line: 'dotted', width: 0.8, color: null },
      arrow: null,
    },
    {
      type: 'path',
      points: [
        [0, 0],
        [2, 0],
      ],
      stroke: { line: 'dashed', width: 0.6, color: null },
      arrow: {
        kind: 'stealth',
        polygon: [
          [2, 0],
          [1.8, 0.1],
          [1.85, 0],
          [1.8, -0.1],
        ],
        line_width: 0.4,
        line_end: [1.85, 0],
      },
    },
    { type: 'label', at: [1, 1], anchor: 'west', tex: String.raw`Graph of $y=\sin x$` },
  ],
};

/** 木の中から，指定した名前の要素をすべて集める． */
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

function textOf(element: Element): string {
  return element.children.map((child) => (child.type === 'text' ? child.value : '')).join('');
}

describe('figureToHast', () => {
  const root = figureToHast(FIGURE);
  const [svg] = all(root, 'svg');
  const paths = all(root, 'path');

  it('figure要素の中に，枠とSVGを持ち，Starlightの余白を除外する', () => {
    expect(root.tagName).toBe('figure');
    expect(classes(root)).toEqual(expect.arrayContaining(['figure', 'not-content']));
    expect(root.properties.style).toBe('--figure-width: 4cm');
    expect(all(root, 'div')).toHaveLength(1);
  });

  it('SVGは，描く範囲を表示範囲とし，説明を名前として持つ', () => {
    // yは上向きなので，表示範囲の上端は，範囲の最大のyの符号を変えた値である．
    expect(svg?.properties.viewBox).toBe('-1 -2 4 3');
    expect(svg?.properties.role).toBe('img');
    expect(svg?.properties.ariaLabel).toBe('試験の図');
  });

  it('折れ線は，yの向きを反転した，パスにする', () => {
    expect(paths[0]?.properties.d).toBe('M0 0L1 -1');
    expect(paths[0]?.properties.fill).toBe('none');
    expect(paths[0]?.properties.stroke).toBe('currentColor');
  });

  it('線幅は，ptからcmに直し，実線には破線の指定を付けない', () => {
    // 0.8pt = 0.8 * 2.54 / 72.27 cm．
    expect(paths[0]?.properties.strokeWidth).toBe('0.02812');
    expect(paths[0]?.properties.strokeDasharray).toBeUndefined();
  });

  it('点線と破線は，TikZと同じ寸法の，破線の指定にする', () => {
    // dotted：線幅の長さの点と，2ptの間．dashed：3ptと3pt．
    expect(paths[1]?.properties.strokeDasharray).toBe('0.02812 0.07029');
    expect(paths[2]?.properties.strokeDasharray).toBe('0.10544 0.10544');
  });

  it('矢じりのある線は，矢じりの手前で止め，輪郭を塗って縁取る', () => {
    expect(paths[2]?.properties.d).toBe('M0 0L1.85 0');
    const [polygon] = all(root, 'polygon');
    expect(polygon?.properties.points).toBe('2,0 1.8,-0.1 1.85,0 1.8,0.1');
    expect(polygon?.properties.fill).toBe('currentColor');
    expect(polygon?.properties.stroke).toBe('currentColor');
    // 0.4pt = 0.01406cm．角は，とがらせる．
    expect(polygon?.properties.strokeWidth).toBe('0.01406');
    expect(polygon?.properties.strokeLinejoin).toBe('miter');
    expect(polygon?.properties.strokeMiterlimit).toBe('10');
  });

  it('ラベルは，位置を割合で置き，アンカーの分だけ，箱をずらす', () => {
    const [label] = all(root, 'span');
    expect(classes(label)).toContain('figure-label');
    // 位置(1, 1)は，範囲(-1から3，-1から2)の，横50%，上から33.3333%である．
    expect(label?.properties.style).toBe(
      'left: 50%; top: 33.3333%; transform: translate(0%, -50%)',
    );
    // 図の絵の一部なので，支援技術には，SVGの説明だけを伝える．
    expect(label?.properties.ariaHidden).toBe('true');
  });

  it('ラベルの文字列は，数式に直して，行内の数式として置く', () => {
    const [code] = all(root, 'code');
    expect(classes(code)).toEqual(['language-math', 'math-inline']);
    expect(textOf(code)).toBe(String.raw`\text{Graph of }y=\sin x`);
  });

  it('色の名前は，CSSの変数に置き換える．色がなければ，文字の色に従う', () => {
    const colored = figureToHast({
      ...FIGURE,
      items: [
        {
          type: 'path',
          points: [
            [0, 0],
            [2, 0],
          ],
          stroke: { line: 'solid', width: 0.6, color: 'red' },
          arrow: {
            kind: 'stealth',
            polygon: [
              [2, 0],
              [1.8, 0.1],
              [1.85, 0],
              [1.8, -0.1],
            ],
            line_width: 0.4,
            line_end: [1.85, 0],
          },
        },
      ],
    });
    const [path] = all(colored, 'path');
    const [polygon] = all(colored, 'polygon');
    expect(path?.properties.stroke).toBe('var(--figure-red)');
    // 矢じりは，線と同じ色で，塗って縁取る．
    expect(polygon?.properties.fill).toBe('var(--figure-red)');
    expect(polygon?.properties.stroke).toBe('var(--figure-red)');
    const [plain] = all(root, 'path');
    expect(plain?.properties.stroke).toBe('currentColor');
  });

  it('何も描かない図は，空のSVGになる', () => {
    const empty = figureToHast({ ...FIGURE, items: [] });
    expect(all(empty, 'svg')).toHaveLength(1);
    expect(all(empty, 'path')).toHaveLength(0);
  });
});
