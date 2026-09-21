import { describe, expect, it } from 'vitest';

import { environments, macros } from './index';
import { mergeMacroModules } from './merge';

describe('mergeMacroModules', () => {
  it('複数のモジュールのマクロと環境を，1つにまとめる', () => {
    const merged = mergeMacroModules({
      './a.ts': { macros: { first: 'x' }, environments: { one: [String.raw`\a`, ''] } },
      './b.ts': { macros: { second: [String.raw`\y{#1}`, 1] } },
    });
    expect(Object.keys(merged.macros)).toEqual(['first', 'second']);
    expect(Object.keys(merged.environments)).toEqual(['one']);
  });

  it('モジュールをまたいで重なるマクロの名前を，持ち主を添えて誤りにする', () => {
    expect(() =>
      mergeMacroModules({
        './a.ts': { macros: { same: 'x' } },
        './b.ts': { macros: { same: 'y' } },
      }),
    ).toThrow(/マクロ「same」が，\.\/a\.tsと\.\/b\.tsで重複している/u);
  });

  it('重なる環境の名前も，誤りにする', () => {
    expect(() =>
      mergeMacroModules({
        './a.ts': { macros: {}, environments: { same: [String.raw`\a`, ''] } },
        './b.ts': { macros: {}, environments: { same: [String.raw`\b`, ''] } },
      }),
    ).toThrow(/環境「same」が/u);
  });

  it('モジュールの並びによらず，結果が同じになる', () => {
    const a = { macros: { alpha: 'a' } };
    const b = { macros: { beta: 'b' } };
    const forward = mergeMacroModules({ './a.ts': a, './b.ts': b });
    const backward = mergeMacroModules({ './b.ts': b, './a.ts': a });
    expect(Object.keys(forward.macros)).toEqual(Object.keys(backward.macros));
  });
});

describe('マクロの一覧', () => {
  it('modules/の下のすべてのファイルから，マクロを集める', () => {
    for (const name of ['RealNumbers', 'NumberSet', 'abs', 'norm', 'set', 'rank']) {
      expect(macros).toHaveProperty(name);
    }
    for (const name of ['openball', 'closure', 'preimage']) {
      expect(macros).toHaveProperty(name);
    }
  });

  it('数の集合のスタイルの環境を，集める', () => {
    expect(Object.keys(environments)).toEqual([
      'numbersetstylebb',
      'numbersetstylebfup',
      'numbersetstylebfit',
    ]);
  });
});
