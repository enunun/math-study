import { describe, expect, it } from 'vitest';

import { isJapanese, isLatin, segmentAutospace } from './autospace';

describe('isJapanese，isLatin', () => {
  it('漢字，ひらがな，カタカナ，長音符を和文とし，句読点と括弧は，どちらでもない', () => {
    expect([...'点あアー'].every((char) => isJapanese(char))).toBe(true);
    expect([...'，．（）'].some((char) => isJapanese(char) || isLatin(char))).toBe(false);
  });

  it('欧文の文字と数字を欧文とし，全角の英数字，ローマ数字，丸数字は，含めない', () => {
    expect([...'Aa1αé'].every((char) => isLatin(char))).toBe(true);
    expect([...'Ａ１Ⅲ①あ'].some((char) => isLatin(char))).toBe(false);
  });
});

describe('segmentAutospace', () => {
  it('和文に挟まれた欧文の連なりに，前後の隙間を付ける', () => {
    expect(segmentAutospace('は1.5倍')).toEqual([
      { text: 'は', before: false, after: false },
      { text: '1.5', before: true, after: true },
      { text: '倍', before: false, after: false },
    ]);
  });

  it('句読点や括弧の側には，付けない', () => {
    expect(segmentAutospace('の（A）と，B')).toEqual([
      { text: 'の（A）と，B', before: false, after: false },
    ]);
    expect(segmentAutospace('とA，')).toEqual([
      { text: 'と', before: false, after: false },
      { text: 'A，', before: true, after: false },
    ]);
  });

  it('文字列の外の，前後の文字を見る', () => {
    expect(segmentAutospace('MDX', 'の', 'を')).toEqual([
      { text: 'MDX', before: true, after: true },
    ]);
    expect(segmentAutospace('の', 'A', 'B')).toEqual([{ text: 'の', before: false, after: false }]);
  });

  it('欧文だけ，和文だけの文字列は，1つのまま返す', () => {
    expect(segmentAutospace('abc')).toEqual([{ text: 'abc', before: false, after: false }]);
    expect(segmentAutospace('定理')).toEqual([{ text: '定理', before: false, after: false }]);
    expect(segmentAutospace('')).toEqual([]);
  });

  it('サロゲートペアの漢字を，1文字として扱う', () => {
    expect(segmentAutospace('𠮷A')).toEqual([
      { text: '𠮷', before: false, after: false },
      { text: 'A', before: true, after: false },
    ]);
  });
});
