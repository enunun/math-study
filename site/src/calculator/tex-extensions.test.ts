import { readdirSync } from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';

import { describe, expect, it } from 'vitest';

import { loadTexExtension, TEX_EXTENSIONS } from './tex-extensions';

describe('TEX_EXTENSIONS', () => {
  it('TeXの拡張のファイルを，すべて持つ', () => {
    const file = createRequire(import.meta.url).resolve(
      '@mathjax/src/bundle/input/tex/extensions/boldsymbol.js',
    );
    const names = readdirSync(path.dirname(file))
      .filter((name) => name.endsWith('.js'))
      .map((name) => name.replace(/\.js$/u, ''));
    expect(Object.keys(TEX_EXTENSIONS).toSorted()).toEqual(names.toSorted());
  });
});

describe('loadTexExtension', () => {
  it('TeXの拡張でないファイルは，失敗にする', async () => {
    await expect(loadTexExtension('@mathjax/src/bundle/ui/menu.js')).rejects.toThrow('ui/menu.js');
  });
});
