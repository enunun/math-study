import { loadCalculator } from '@/calculator/wasm';
import type { Calculate } from '@/calculator/wasm';
import { useLoaded } from '@/components/use-loaded';

interface CalculatorState {
  /** 読み込み済みの計算する関数．読み込み中，または失敗したときは，未定義である． */
  calculate: Calculate | undefined;
  /** 読み込みに失敗したかどうか． */
  loadFailed: boolean;
}

/** RustのWasmを，最初に表示されたときに読み込み，計算する関数を返す． */
function useCalculate(): CalculatorState {
  const { value, failed } = useLoaded(loadCalculator);
  return { calculate: value, loadFailed: failed };
}

export { useCalculate };
