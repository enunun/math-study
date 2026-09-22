import { describe, expect, it } from 'vitest';

import { evaluateConstant } from './constant-expr';

describe('定数だけの式を数にする', () => {
  it('数と，四則演算，累乗，丸括弧を読む', () => {
    expect(evaluateConstant('2')).toBe(2);
    expect(evaluateConstant('2 + 3 * 4')).toBe(14);
    expect(evaluateConstant('(2 + 3) * 4')).toBe(20);
    expect(evaluateConstant('2^3')).toBe(8);
    expect(evaluateConstant('2^-1')).toBe(0.5);
    expect(evaluateConstant('-2 - -3')).toBe(1);
  });

  it('piとeを定数として読む', () => {
    expect(evaluateConstant('2*pi')).toBeCloseTo(2 * Math.PI);
    expect(evaluateConstant('pi/2')).toBeCloseTo(Math.PI / 2);
    expect(evaluateConstant('e')).toBeCloseTo(Math.E);
  });

  it('媒介変数や関数など，piとe以外の名前は読めない', () => {
    expect(evaluateConstant('r')).toBeUndefined();
    expect(evaluateConstant('sin(1)')).toBeUndefined();
  });

  it('構文が壊れていれば読めない', () => {
    expect(evaluateConstant('')).toBeUndefined();
    expect(evaluateConstant('2 +')).toBeUndefined();
    expect(evaluateConstant('(2 + 3')).toBeUndefined();
    expect(evaluateConstant('2 3')).toBeUndefined();
    expect(evaluateConstant('2 @ 3')).toBeUndefined();
  });
});
