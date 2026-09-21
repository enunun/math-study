import init, { parseScene, renderScene } from '@/wasm/figure';
import type { RenderOutcome, SceneOutcome } from '@/wasm/figure';
import wasmUrl from '@/wasm/figure_bg.wasm?url';

/** シーンを読む関数と，描画する関数．シーンの誤りは，例外ではなく，`status`が`"error"`の値で返す． */
interface SceneEngine {
  /** シーンのJSONを読み，検査する． */
  parseScene: (json: string) => SceneOutcome;
  /** シーンのJSONを描画して，中間表現とTikZを返す． */
  renderScene: (json: string) => RenderOutcome;
}

const state: { loading?: Promise<SceneEngine> } = {};

async function initialize(): Promise<SceneEngine> {
  await init({ module_or_path: wasmUrl });
  return { parseScene, renderScene };
}

/**
 * RustのWasmを読み込み，シーンを読む関数と描画する関数を返す．
 * 最初の呼び出しで読み込みを始め，以降は，同じ読み込みの結果を返す．
 */
function loadSceneEngine(): Promise<SceneEngine> {
  state.loading ??= initialize();
  return state.loading;
}

export { loadSceneEngine };
export type { SceneEngine };
export type { RenderOutcome, SceneError, SceneObject, SceneOutcome } from '@/wasm/figure';
