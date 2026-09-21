import type { AnchorName } from '@/wasm/figure';

interface Shift {
  x: number;
  y: number;
}

/** 箱の真ん中の割合． */
const MIDDLE = 0.5;
/** `$`で区切った節の，数式は奇数番目，文字は偶数番目． */
const PARITY = 2;

/** アンカーが，`low`側と`high`側のどちらの名前を含むかで，割合を決める．どちらもなければ，真ん中である． */
function fraction(anchor: AnchorName, low: string, high: string): number {
  if (anchor.includes(low)) {
    return 0;
  }
  return anchor.includes(high) ? 1 : MIDDLE;
}

/** ラベルの箱の，位置に合わせる点の，箱の中での場所．左端をxの0，上端をyの0とした割合． */
function anchorShift(anchor: AnchorName): Shift {
  return { x: fraction(anchor, 'west', 'east'), y: fraction(anchor, 'north', 'south') };
}

/** `$`で分けた，文字の部分と数式の部分．`\$`は，区切りにしない． */
function splitOnDollars(tex: string): string[] {
  const parts = [''];
  for (let index = 0; index < tex.length; index += 1) {
    const char = tex.charAt(index);
    const last = parts.length - 1;
    if (char === '\\' && tex.charAt(index + 1) === '$') {
      parts[last] = `${parts[last] ?? ''}\\$`;
      index += 1;
    } else if (char === '$') {
      parts.push('');
    } else {
      parts[last] = `${parts[last] ?? ''}${char}`;
    }
  }
  return parts;
}

/**
 * ラベルの文字列(TikZの節点に書く，`$…$`で数式を含む文字列)を，MathJaxが描画できる，1つの数式にする．
 * 文字の部分は`\text{…}`で包み，数式の部分は，そのままつなぐ．
 * 節の数が偶数(`$`が奇数個)で閉じていないときは，全体を文字として扱う．
 */
function labelToMath(tex: string): string {
  const parts = splitOnDollars(tex);
  if (parts.length % PARITY === 0) {
    return `\\text{${tex.replaceAll(/(?<!\\)\$/gu, String.raw`\$`)}}`;
  }
  return parts
    .map((part, index) => {
      if (part === '') {
        return '';
      }
      // 偶数番目が文字，奇数番目が数式である．
      return index % PARITY === 0 ? `\\text{${part}}` : part;
    })
    .join('');
}

export { anchorShift, labelToMath };
export type { Shift };
