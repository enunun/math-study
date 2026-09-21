import { useCallback, useEffect, useState } from 'react';

import { emptyDraft, parseDraft, stringifyDraft } from '@/figure-editor/draft';
import type { SceneDraft } from '@/figure-editor/draft';

const STORAGE_KEY = 'math-study:figure-editor';

interface DraftState {
  draft: SceneDraft;
  /** 外から図を読み込むたびに増える．入力欄の作り直しに使う． */
  revision: number;
  setDraft: (draft: SceneDraft) => void;
  /** 見本や，読み込んだファイルの図に置き換える． */
  load: (draft: SceneDraft) => void;
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
 * 編集中の図．ブラウザに保存して，開き直したときに戻す．保存できない環境では，保存せずに使える．
 * サーバーでの描画と揃えるため，保存した図は，最初の描画のあとで読む．
 */
function useDraft(): DraftState {
  const [draft, setDraft] = useState<SceneDraft>(() => emptyDraft());
  const [revision, setRevision] = useState(0);
  const [restored, setRestored] = useState(false);

  useEffect(() => {
    const saved = readSaved();
    if (saved !== undefined) {
      setDraft(saved);
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

  const load = useCallback((next: SceneDraft) => {
    setDraft(next);
    setRevision((value) => value + 1);
  }, []);

  return { draft, revision, setDraft, load };
}

export { useDraft };
