import { readdirSync } from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';

import { describe, expect, it } from 'vitest';

import { FONT_FILES } from './label-font-files';
import { labelSvg } from './label-svg';

describe('labelSvg', () => {
  it('ラベルの式を，文字の形をパスで持つSVGにする', async () => {
    const label = await labelSvg(String.raw`x^2`);
    expect(label.viewBox).toMatch(/^0 -/u);
    expect(label.content).toContain('<path');
    expect(label.content).not.toContain('<use');
    expect(label.width).toBeGreaterThan(0);
    expect(label.height).toBeGreaterThan(0);
  });

  it('演算子の後ろも，同じ1つのSVGに含める', async () => {
    const label = await labelSvg('x+y');
    expect(label.content).toContain('data-c="2B"');
    expect(label.content).toContain('data-c="1D466"');
    expect(label.width).toBeGreaterThan(2);
  });

  it('追加の字形(黒板太字)と，自作マクロを組める', async () => {
    const label = await labelSvg(String.raw`\RealNumbers \colored{red}{y}`);
    expect(label.content).toContain('math-color-red');
    expect(label.content).toContain('<path');
  });

  it('日本語の文字は，文字として残す', async () => {
    const label = await labelSvg(String.raw`\text{原点}`);
    expect(label.content).toContain('原点');
  });

  it('未定義のマクロは，失敗にする', async () => {
    await expect(labelSvg(String.raw`\undefinedmacro`)).rejects.toThrow();
  });
});

describe('FONT_FILES', () => {
  it('フォントの追加の字形のファイルを，すべて持つ', () => {
    const directory = createRequire(import.meta.url).resolve(
      '@mathjax/mathjax-newcm-font/js/svg/dynamic/latin.js',
    );
    const files = readdirSync(path.dirname(directory))
      .filter((file) => file.endsWith('.js'))
      .map((file) => file.replace(/\.js$/u, ''));
    expect(Object.keys(FONT_FILES).toSorted()).toEqual(files.toSorted());
  });
});
