import { describe, expect, it } from 'vitest';

import { stepsToText } from './values';

describe('変換の手順の文字列', () => {
  it('1行に1つの手順を書く', () => {
    expect(stepsToText([{ rotate: 90 }, { translate: [1, 0] }])).toBe(
      '[\n  {"rotate":90},\n  {"translate":[1,0]}\n]',
    );
  });

  it('手順がなければ，空の文字列にする', () => {
    expect(stepsToText([])).toBe('');
  });
});
