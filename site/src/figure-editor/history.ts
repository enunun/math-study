/**
 * 編集の履歴(元に戻す・やり直す)．図は書き換えるたびに新しい値になるので，値をそのまま積む．
 * 入力欄への打ち込み，スライダー，点のドラッグ，再生のような続けての書き換えは，間隔が短ければ1つの編集に
 * まとめる．そうしないと，1文字や1コマごとに戻すことになる．
 */

/** 続けての書き換えを1つの編集にまとめる，記録の間隔の上限(ミリ秒)． */
const MERGE_INTERVAL = 400;
/** 戻せる編集の数の上限．古いものから捨てる． */
const HISTORY_LIMIT = 100;

interface History<T> {
  /** 戻せる図．古い順． */
  past: readonly T[];
  present: T;
  /** やり直せる図．次に来る順． */
  future: readonly T[];
  /** 最後に記録した時刻．戻す・やり直す・区切りのあとは，まとめないように`-Infinity`にする． */
  recordedAt: number;
}

function startHistory<T>(present: T): History<T> {
  return { past: [], present, future: [], recordedAt: -Infinity };
}

/**
 * 次の図を記録する．前の記録から`MERGE_INTERVAL`より短ければ，今の図を置き換えるだけにする．
 * `now`は記録の時刻(ミリ秒)．`separate`は，見本の読み込みのような，まとめてはいけない書き換えに使う．
 */
function record<T>(
  history: History<T>,
  next: T,
  { now, separate = false }: { now: number; separate?: boolean },
): History<T> {
  if (next === history.present) {
    return history;
  }
  const merge = !separate && now - history.recordedAt < MERGE_INTERVAL;
  const past = merge ? history.past : [...history.past, history.present].slice(-HISTORY_LIMIT);
  return { past, present: next, future: [], recordedAt: separate ? -Infinity : now };
}

function canUndo<T>(history: History<T>): boolean {
  return history.past.length > 0;
}

function canRedo<T>(history: History<T>): boolean {
  return history.future.length > 0;
}

function undo<T>(history: History<T>): History<T> {
  const previous = history.past.at(-1);
  if (previous === undefined) {
    return history;
  }
  return {
    past: history.past.slice(0, -1),
    present: previous,
    future: [history.present, ...history.future],
    recordedAt: -Infinity,
  };
}

function redo<T>(history: History<T>): History<T> {
  const [next, ...future] = history.future;
  if (next === undefined) {
    return history;
  }
  return {
    past: [...history.past, history.present],
    present: next,
    future,
    recordedAt: -Infinity,
  };
}

export { HISTORY_LIMIT, MERGE_INTERVAL, canRedo, canUndo, record, redo, startHistory, undo };
export type { History };
