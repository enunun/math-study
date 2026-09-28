import { describe, expect, it } from 'vitest';

import {
  HISTORY_LIMIT,
  MERGE_INTERVAL,
  canRedo,
  canUndo,
  record,
  redo,
  startHistory,
  undo,
} from './history';

/** まとめる間隔より十分あとの時刻． */
const LATER = MERGE_INTERVAL * 2;

describe('編集の履歴', () => {
  it('記録した図を，元に戻し，やり直せる', () => {
    let history = startHistory('a');
    history = record(history, 'b', { now: 0 });
    history = record(history, 'c', { now: LATER });
    expect(history.present).toBe('c');
    history = undo(history);
    expect(history.present).toBe('b');
    history = undo(history);
    expect(history.present).toBe('a');
    expect(canUndo(history)).toBe(false);
    history = redo(history);
    expect(history.present).toBe('b');
    expect(canRedo(history)).toBe(true);
  });

  it('戻せないとき，やり直せないときは，そのままである', () => {
    const history = startHistory('a');
    expect(undo(history)).toBe(history);
    expect(redo(history)).toBe(history);
  });

  it('続けての書き換え(入力やドラッグ)は，1つの編集にまとめる', () => {
    let history = startHistory('a');
    history = record(history, 'b', { now: 0 });
    history = record(history, 'bc', { now: MERGE_INTERVAL / 2 });
    history = record(history, 'bcd', { now: MERGE_INTERVAL });
    expect(history.present).toBe('bcd');
    expect(undo(history).present).toBe('a');
  });

  it('区切りを付けた記録(見本の読み込み)は，続けてでもまとめない', () => {
    let history = startHistory('a');
    history = record(history, 'b', { now: 0 });
    history = record(history, 'sample', { now: 1, separate: true });
    expect(undo(history).present).toBe('b');
  });

  it('戻したあとの編集は，やり直しの図を捨て，前の書き換えとまとめない', () => {
    let history = startHistory('a');
    history = record(history, 'b', { now: 0 });
    history = undo(history);
    history = record(history, 'x', { now: 1 });
    expect(canRedo(history)).toBe(false);
    expect(undo(history).present).toBe('a');
  });

  it('同じ図の記録は，履歴を変えない', () => {
    const history = record(startHistory('a'), 'b', { now: 0 });
    expect(record(history, 'b', { now: LATER })).toBe(history);
  });

  it('戻せる図は，上限の数だけ残し，古いものから捨てる', () => {
    let history = startHistory(0);
    const count = HISTORY_LIMIT + 10;
    for (let step = 1; step <= count; step += 1) {
      history = record(history, step, { now: step * LATER });
    }
    expect(history.past).toHaveLength(HISTORY_LIMIT);
    expect(history.past[0]).toBe(count - HISTORY_LIMIT);
  });
});
