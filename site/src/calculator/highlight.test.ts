import { describe, expect, it } from 'vitest';

import { splitAt } from './highlight';

describe('splitAt', () => {
  it('誤りのある範囲で，入力を3つに分ける', () => {
    expect(splitAt('1 + * 2', 4, 5)).toEqual({ before: '1 + ', hit: '*', after: ' 2' });
  });

  it('範囲が空(式の終わり)のときは，前だけに入力が入る', () => {
    expect(splitAt('1 +', 3, 3)).toEqual({ before: '1 +', hit: '', after: '' });
  });

  it('サロゲートペアの文字を，1文字として数える', () => {
    expect(splitAt('x + 𠮷 + y', 4, 5)).toEqual({ before: 'x + ', hit: '𠮷', after: ' + y' });
  });
});
