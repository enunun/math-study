import { useEffect } from 'react';
import type { ReactElement } from 'react';

interface Props {
  undo: () => void;
  redo: () => void;
  canUndo: boolean;
  canRedo: boolean;
}

/** 文字を打ち込む欄．そこでのCtrl+Zは，欄の中の文字の取り消しとしてブラウザに任せる． */
function isTextEntry(target: EventTarget | null): boolean {
  if (target instanceof HTMLTextAreaElement) {
    return true;
  }
  return (
    target instanceof HTMLInputElement &&
    !['range', 'checkbox', 'button', 'file'].includes(target.type)
  );
}

/**
 * Ctrl+Z(macOSでは⌘Z)で元に戻し，Ctrl+Shift+ZかCtrl+Yでやり直す．文字を打ち込む欄の中では，ブラウザの
 * 取り消しを優先する．
 */
function useHistoryKeys({ undo, redo }: Pick<Props, 'undo' | 'redo'>): void {
  useEffect(() => {
    const onKey = (event: KeyboardEvent): void => {
      if (!(event.ctrlKey || event.metaKey) || event.altKey || isTextEntry(event.target)) {
        return;
      }
      const key = event.key.toLowerCase();
      const redoKey = key === 'y' || (key === 'z' && event.shiftKey);
      if (key !== 'z' && !redoKey) {
        return;
      }
      event.preventDefault();
      if (redoKey) {
        redo();
      } else {
        undo();
      }
    };
    document.addEventListener('keydown', onKey);
    return (): void => {
      document.removeEventListener('keydown', onKey);
    };
  }, [undo, redo]);
}

/** 元に戻す・やり直すのボタン．キーボードの操作も受け付ける． */
function HistoryControls({ undo, redo, canUndo, canRedo }: Props): ReactElement {
  useHistoryKeys({ undo, redo });
  return (
    <div className="fe-buttons" role="group" aria-label="編集の履歴">
      <button type="button" disabled={!canUndo} onClick={undo} title="Ctrl+Z">
        元に戻す
      </button>
      <button type="button" disabled={!canRedo} onClick={redo} title="Ctrl+Shift+Z">
        やり直す
      </button>
    </div>
  );
}

export { HistoryControls };
