import { useCallback, useEffect, useState } from 'react';

import { emptyDraft, parseDraft, stringifyDraft, viewKind } from '@/figure-editor/draft';
import type { SceneDraft } from '@/figure-editor/draft';
import { canRedo, canUndo, record, redo, startHistory, undo } from '@/figure-editor/history';
import type { History } from '@/figure-editor/history';
import { insertObjects } from '@/figure-editor/insert-template';
import type { JsonObject } from '@/figure-editor/json';

const STORAGE_KEY = 'math-study:figure-editor';

interface DraftState {
  draft: SceneDraft;
  /** 外から図を読み込むたび，戻す・やり直すたびに増える．入力欄の作り直しに使う． */
  revision: number;
  setDraft: (draft: SceneDraft) => void;
  /** 今の図から次の図を作る．スライダーの再生のように，続けて書き換えるときに使う． */
  update: (change: (draft: SceneDraft) => SceneDraft) => void;
  /** 見本や，読み込んだファイルの図に置き換える． */
  load: (draft: SceneDraft) => void;
  /** テンプレートのオブジェクトを，重ならない識別子に付け替えて，今の図に加える． */
  insert: (objects: readonly JsonObject[]) => void;
  /** 元に戻す・やり直す．できないときは何もしない． */
  undo: () => void;
  redo: () => void;
  canUndo: boolean;
  canRedo: boolean;
}

function readSaved(): SceneDraft | undefined {
  try {
    const text = localStorage.getItem(STORAGE_KEY);
    const parsed = text === null ? undefined : parseDraft(text);
    return parsed?.ok === true ? parsed.draft : undefined;
  } catch {
    return undefined;
  }
}

/**
 * 保存した図を，最初の描画のあとで読み(履歴はそこから始める)，以後は図が変わるたびに保存する．
 */
function useSaved(
  draft: SceneDraft,
  setHistory: (history: History<SceneDraft>) => void,
  setRevision: (change: (value: number) => number) => void,
): void {
  const [restored, setRestored] = useState(false);
  useEffect(() => {
    const saved = readSaved();
    if (saved !== undefined) {
      setHistory(startHistory(saved));
      setRevision((value) => value + 1);
    }
    setRestored(true);
  }, []);

  useEffect(() => {
    if (!restored) {
      return;
    }
    try {
      localStorage.setItem(STORAGE_KEY, stringifyDraft(draft));
    } catch {
      // 保存できなくても，編集は続けられる．
    }
  }, [draft, restored]);
}

/** 履歴を書き換え，入力欄を作り直す．戻す・やり直すに使う． */
function useStep(
  setHistory: (change: (history: History<SceneDraft>) => History<SceneDraft>) => void,
  setRevision: (change: (value: number) => number) => void,
  step: (history: History<SceneDraft>) => History<SceneDraft>,
): () => void {
  return useCallback(() => {
    setHistory(step);
    setRevision((value) => value + 1);
  }, [setHistory, setRevision, step]);
}

/**
 * 編集中の図と，その履歴．ブラウザに保存して，開き直したときに戻す．保存できない環境では，保存せずに使える．
 * サーバーでの描画と揃えるため，保存した図は，最初の描画のあとで読む．履歴は保存しない．
 */
function useDraft(): DraftState {
  const [history, setHistory] = useState(() => startHistory(emptyDraft()));
  const [revision, setRevision] = useState(0);
  const draft = history.present;

  useSaved(draft, setHistory, setRevision);

  const update = useCallback((change: (current: SceneDraft) => SceneDraft) => {
    const now = performance.now();
    setHistory((current) => record(current, change(current.present), { now }));
  }, []);
  const setDraft = useCallback(
    (next: SceneDraft) => {
      update(() => next);
    },
    [update],
  );
  const load = useCallback((next: SceneDraft) => {
    setHistory((current) => record(current, next, { now: 0, separate: true }));
    setRevision((value) => value + 1);
  }, []);
  const insert = useCallback((objects: readonly JsonObject[]) => {
    setHistory((current) =>
      record(current, insertObjects(current.present, viewKind(current.present), objects), {
        now: 0,
        separate: true,
      }),
    );
  }, []);

  return {
    draft,
    revision,
    setDraft,
    update,
    load,
    insert,
    undo: useStep(setHistory, setRevision, undo),
    redo: useStep(setHistory, setRevision, redo),
    canUndo: canUndo(history),
    canRedo: canRedo(history),
  };
}

export { useDraft };
