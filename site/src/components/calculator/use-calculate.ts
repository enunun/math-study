import { useEffect, useState } from 'react';

import { loadCalculator } from '@/calculator/wasm';
import type { Calculate } from '@/calculator/wasm';

interface CalculatorState {
  /** 読み込み済みの計算する関数．読み込み中，または失敗したときは，未定義である． */
  calculate: Calculate | undefined;
  /** 読み込みに失敗したかどうか． */
  loadFailed: boolean;
}

/** RustのWasmを，最初に表示されたときに読み込み，計算する関数を返す． */
function useCalculate(): CalculatorState {
  const [calculate, setCalculate] = useState<Calculate>();
  const [loadFailed, setLoadFailed] = useState(false);

  useEffect(() => {
    let active = true;
    const load = async (): Promise<void> => {
      try {
        const loaded = await loadCalculator();
        if (active) {
          // 関数を状態に入れるため，関数の形で渡す．
          setCalculate(() => loaded);
        }
      } catch {
        if (active) {
          setLoadFailed(true);
        }
      }
    };
    void load();
    return (): void => {
      active = false;
    };
  }, []);

  return { calculate, loadFailed };
}

export { useCalculate };
