import { describe, expect, it } from 'vitest';

import { slashFractions } from './inline-fraction';

describe('slashFractions', () => {
  it('分子と分母が1つの文字か数のときは，括弧を付けない', () => {
    expect(slashFractions(String.raw`\frac{1}{2}`)).toBe('1/2');
    expect(slashFractions(String.raw`\frac{\pi}{2}`)).toBe(String.raw`\pi/2`);
    expect(slashFractions(String.raw`\frac12`)).toBe('1/2');
    expect(slashFractions(String.raw`\frac {a} {b}`)).toBe('a/b');
    expect(slashFractions(String.raw`\frac{1.5}{10}`)).toBe('1.5/10');
  });

  it('分子は，掛け合わせた因子の並びなら，括弧を付けない', () => {
    expect(slashFractions(String.raw`\frac{3\pi}{2}`)).toBe(String.raw`3\pi/2`);
    expect(slashFractions(String.raw`\frac{x^{n+1}}{n}`)).toBe('x^{n+1}/n');
    expect(slashFractions(String.raw`\frac{(a+b)^2}{4}`)).toBe('(a+b)^2/4');
  });

  it('分子に演算子や関数があれば，括弧で囲む', () => {
    expect(slashFractions(String.raw`\frac{a+b}{2}`)).toBe('(a+b)/2');
    expect(slashFractions(String.raw`\frac{\sin x}{x}`)).toBe(String.raw`(\sin x)/x`);
  });

  it('分母は，1つの因子でなければ，括弧で囲む', () => {
    expect(slashFractions(String.raw`\frac{1}{2x}`)).toBe('1/(2x)');
    expect(slashFractions(String.raw`\frac{m}{\sqrt{1 + m^2}}`)).toBe(String.raw`m/\sqrt{1 + m^2}`);
    expect(slashFractions(String.raw`\frac{1}{x^2}`)).toBe('1/x^2');
  });

  it('入れ子の分数は，内側を括弧で囲む', () => {
    expect(slashFractions(String.raw`\frac{\frac{a}{b}}{c}`)).toBe('(a/b)/c');
    expect(slashFractions(String.raw`\sqrt{\frac12}`)).toBe(String.raw`\sqrt{1/2}`);
  });

  it('後ろに因子や添字が続くときは，分数全体を括弧で囲む', () => {
    expect(slashFractions(String.raw`\frac{1}{2}x`)).toBe('(1/2)x');
    expect(slashFractions(String.raw`\frac{d}{dx} f(x)`)).toBe('(d/(dx)) f(x)');
    expect(slashFractions(String.raw`\frac{a}{b}^2`)).toBe('(a/b)^2');
    expect(slashFractions(String.raw`\frac{\pi}{2}\Integers`)).toBe(String.raw`(\pi/2)\Integers`);
  });

  it('後ろが演算子，関係，閉じ括弧なら，囲まない', () => {
    expect(slashFractions(String.raw`\frac{\pi}{2} < x`)).toBe(String.raw`\pi/2 < x`);
    expect(slashFractions(String.raw`\left(\frac{\pi}{2}\right)`)).toBe(
      String.raw`\left(\pi/2\right)`,
    );
    expect(slashFractions(String.raw`-\frac{\pi}{2}, \frac{\pi}{2}]`)).toBe(
      String.raw`-\pi/2, \pi/2]`,
    );
    expect(slashFractions(String.raw`\frac{a}{b} \leq c`)).toBe(String.raw`a/b \leq c`);
  });

  it('frac以外の命令と，エスケープした波括弧は，そのまま残す', () => {
    expect(slashFractions(String.raw`\dfrac{a}{b} + \set{x}`)).toBe(
      String.raw`\dfrac{a}{b} + \set{x}`,
    );
    expect(slashFractions(String.raw`\{\frac{1}{n}\}`)).toBe(String.raw`\{1/n\}`);
    expect(slashFractions(String.raw`\fracture`)).toBe(String.raw`\fracture`);
  });
});
