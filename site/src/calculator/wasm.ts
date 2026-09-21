import init, { calculate } from '@/wasm/polynomial';
import type { Outcome } from '@/wasm/polynomial';
import wasmUrl from '@/wasm/polynomial_bg.wasm?url';

/** 式を展開し，変数ごとに偏微分する．式の誤りは，例外ではなく，`status`が`"error"`の値で返す． */
type Calculate = (source: string) => Outcome;

const state: { loading?: Promise<Calculate> } = {};

async function initialize(): Promise<Calculate> {
  await init({ module_or_path: wasmUrl });
  return calculate;
}

/**
 * RustのWasmを読み込み，計算する関数を返す．
 * 最初の呼び出しで読み込みを始め，以降は，同じ読み込みの結果を返す．
 */
function loadCalculator(): Promise<Calculate> {
  state.loading ??= initialize();
  return state.loading;
}

export { loadCalculator };
export type { Calculate };
export type { Outcome } from '@/wasm/polynomial';
