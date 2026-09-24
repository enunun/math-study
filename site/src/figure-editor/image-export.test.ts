import { describe, expect, it } from 'vitest';

import type { Figure } from '@/wasm/figure';

import { figureSvg } from './image-export';

const FIGURE: Figure = {
  description: '試験の図',
  bounds: { min: [0, 0], max: [2, 1] },
  items: [
    { type: 'label', at: [0, 0], anchor: 'north east', tex: '$O$' },
    { type: 'label', at: [2, 1], anchor: 'south', tex: String.raw`原点と$\RealNumbers$` },
  ],
};

describe('figureSvg', () => {
  it('ラベルを，MathJaxのSVGで組み，図のSVGの中に置く', async () => {
    const file = await figureSvg(FIGURE, true);
    expect(file.svg.match(/data-mml-node="math"/gu)).toHaveLength(2);
    expect(file.svg).toContain('原点と');
    expect(file.svg).not.toContain('<rect');
    // 左下と上にはみ出したラベルの分，図の範囲より大きい．
    expect(file.width).toBeGreaterThan(2);
    expect(file.height).toBeGreaterThan(1);
  });
});
