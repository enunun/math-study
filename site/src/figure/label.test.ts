import { describe, expect, it } from 'vitest';

import { anchorShift, labelToMath } from './label';

describe('labelToMath', () => {
  it('数式を含まない文字列は，文字として扱う', () => {
    expect(labelToMath('O')).toBe(String.raw`\text{O}`);
    expect(labelToMath('図の説明')).toBe(String.raw`\text{図の説明}`);
  });

  it('$で囲んだところは数式として，そのまま渡す', () => {
    expect(labelToMath('$x$')).toBe('x');
    expect(labelToMath(String.raw`$y=\sin x$`)).toBe(String.raw`y=\sin x`);
  });

  it('文字と数式が混ざった文字列は，文字の部分をtext命令で包んでつなぐ', () => {
    expect(labelToMath(String.raw`Graph of $y=\sin x$`)).toBe(String.raw`\text{Graph of }y=\sin x`);
    expect(labelToMath('$a$ and $b$')).toBe(String.raw`a\text{ and }b`);
    expect(labelToMath('点$P$の座標')).toBe(String.raw`\text{点}P\text{の座標}`);
  });

  it('空の部分は出力しない', () => {
    expect(labelToMath('$a$$b$')).toBe('ab');
    expect(labelToMath('')).toBe('');
  });

  it('バックスラッシュを付けたドル記号は，区切りではなく，ドル記号として扱う', () => {
    expect(labelToMath(String.raw`price \$5`)).toBe(String.raw`\text{price \$5}`);
  });

  it('$の数が奇数で，閉じていないときは，全体を文字として扱う', () => {
    expect(labelToMath('a $b')).toBe(String.raw`\text{a \$b}`);
  });
});

describe('anchorShift', () => {
  it('アンカーが，箱のどの部分を位置に合わせるかを，割合で返す', () => {
    // 左端をx，上端をyの0とした割合．
    expect(anchorShift('center')).toEqual({ x: 0.5, y: 0.5 });
    expect(anchorShift('west')).toEqual({ x: 0, y: 0.5 });
    expect(anchorShift('east')).toEqual({ x: 1, y: 0.5 });
    expect(anchorShift('north')).toEqual({ x: 0.5, y: 0 });
    expect(anchorShift('south')).toEqual({ x: 0.5, y: 1 });
    expect(anchorShift('north west')).toEqual({ x: 0, y: 0 });
    expect(anchorShift('north east')).toEqual({ x: 1, y: 0 });
    expect(anchorShift('south west')).toEqual({ x: 0, y: 1 });
    expect(anchorShift('south east')).toEqual({ x: 1, y: 1 });
  });
});
