/**
 * 和文と欧文の間の隙間(TeXの`\xkanjiskip`)を入れる位置を決める．
 * 原稿にはスペースを書かず，組版のときに，欧文の側の要素に`autospace-before`と`autospace-after`のクラスを付ける．
 * 隙間の幅は，CSS(`typesetting.css`)が決める．
 */

/** 和文の文字．漢字，ひらがな，カタカナ，長音符．句読点や括弧は，含まない． */
const JAPANESE_LETTER = /[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}ー]/u;

/**
 * 欧文の文字と数字．和文の組版に合わせた文字は，TeXでも和文の隣に隙間が入らないので，含めない．
 * 全角の英数字(U+FF00からU+FFEF)，ローマ数字などの数字の形(Ⅲ，U+2150からU+218F)，丸数字などの囲み英数字(①，U+2460からU+24FF)である．
 */
const LATIN_LETTER = /[\p{L}\p{N}]/u;
const JAPANESE_FORM = /[\u2150-\u218F\u2460-\u24FF\uFF00-\uFFEF]/u;

const BEFORE_CLASS = 'autospace-before';
const AFTER_CLASS = 'autospace-after';

function isJapanese(char: string | undefined): boolean {
  return char !== undefined && JAPANESE_LETTER.test(char);
}

function isLatin(char: string | undefined): boolean {
  return (
    char !== undefined &&
    LATIN_LETTER.test(char) &&
    !JAPANESE_LETTER.test(char) &&
    !JAPANESE_FORM.test(char)
  );
}

/** 文字列の断片．`before`と`after`は，その側に隙間を入れるかどうか． */
interface Segment {
  text: string;
  before: boolean;
  after: boolean;
}

/** 文字列を，和文の文字の連なりと，それ以外の連なりに分ける．各連なりは，文字の配列． */
function splitRuns(text: string): string[][] {
  // サロゲートペアの文字(𠮷など)を，1文字として扱うため，コードポイントで分ける．
  const runs: string[][] = [];
  for (const char of text) {
    const last = runs.at(-1);
    if (last?.[0] !== undefined && isJapanese(last[0]) === isJapanese(char)) {
      last.push(char);
    } else {
      runs.push([char]);
    }
  }
  return runs;
}

/**
 * 文字列を，和文の文字の連なりと，それ以外の連なりに分け，和文の隣にある欧文の端に，隙間の印を付ける．
 * `previous`と`next`は，文字列の外の，前後の文字．隙間を付けない断片は，1つにまとめる．
 */
function segmentAutospace(text: string, previous?: string, next?: string): Segment[] {
  const runs = splitRuns(text);
  const segments: Segment[] = [];
  for (const [index, run] of runs.entries()) {
    const before =
      !isJapanese(run[0]) && isLatin(run[0]) && isJapanese(runs[index - 1]?.at(-1) ?? previous);
    const after =
      !isJapanese(run[0]) && isLatin(run.at(-1)) && isJapanese(runs[index + 1]?.[0] ?? next);
    const last = segments.at(-1);
    if (!before && !after && last !== undefined && !last.before && !last.after) {
      last.text += run.join('');
    } else {
      segments.push({ text: run.join(''), before, after });
    }
  }
  return segments;
}

/** 断片に付けるクラス． */
function autospaceClasses(segment: Pick<Segment, 'before' | 'after'>): string[] {
  return [...(segment.before ? [BEFORE_CLASS] : []), ...(segment.after ? [AFTER_CLASS] : [])];
}

export { autospaceClasses, isJapanese, isLatin, segmentAutospace };
export type { Segment };
