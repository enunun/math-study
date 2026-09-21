import { readFile } from 'node:fs/promises';

import { beforeAll, describe, expect, it } from 'vitest';

import { calculate, initSync } from '@/wasm/polynomial';

/**
 * Rustで書いた計算機を，Wasmにしたものを，そのまま呼んで，JavaScriptとの約束を確かめる．
 * 式の意味の網羅は，Rust側のテストが担う．ここでは，値の形と，位置，文字の扱いを確かめる．
 */
beforeAll(async () => {
  const module = await readFile(new URL('../wasm/polynomial_bg.wasm', import.meta.url));
  initSync({ module });
});

describe('Wasmの計算機', () => {
  it('式を展開し，変数ごとに偏微分する', () => {
    expect(calculate('(x+1)^2')).toEqual({
      status: 'ok',
      expanded: { tex: 'x^2 + 2x + 1', text: 'x^2 + 2*x + 1' },
      variables: ['x'],
      derivatives: [{ variable: 'x', tex: '2x + 2', text: '2*x + 2' }],
    });
  });

  it('複数の変数を，名前の順に偏微分する', () => {
    const outcome = calculate('(a+b)^2');
    expect(outcome.status).toBe('ok');
    if (outcome.status === 'ok') {
      expect(outcome.variables).toEqual(['a', 'b']);
      expect(outcome.derivatives.map(({ text }) => text)).toEqual(['2*a + 2*b', '2*a + 2*b']);
    }
  });

  it('全角の記号と数字で書いた式を読む', () => {
    const outcome = calculate('（ｘ＋１）＾２');
    expect(outcome.status === 'ok' && outcome.expanded.text).toBe('x^2 + 2*x + 1');
  });

  it('誤りを，例外ではなく，位置つきの値で返す', () => {
    const outcome = calculate('1 + * 2');
    expect(outcome.status).toBe('error');
    if (outcome.status === 'error') {
      expect([outcome.start, outcome.end]).toEqual([4, 5]);
      expect(outcome.message).toContain('*');
    }
  });

  it('位置は，UTF-16ではなく，文字の番号で数える', () => {
    // 「𠮷」は，UTF-16で2つ分だが，1文字として数える．
    const outcome = calculate('x + 𠮷');
    expect(outcome.status === 'error' && [outcome.start, outcome.end]).toEqual([4, 5]);
  });

  it('空の入力は，式が途中で終わる誤りになる', () => {
    expect(calculate('').status).toBe('error');
  });
});
