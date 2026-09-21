import init, { parseScene } from '@/wasm/figure';
import type { SceneOutcome } from '@/wasm/figure';
import wasmUrl from '@/wasm/figure_bg.wasm?url';

/** シーンのJSONを読み，検査する．シーンの誤りは，例外ではなく，`status`が`"error"`の値で返す． */
type ParseScene = (json: string) => SceneOutcome;

const state: { loading?: Promise<ParseScene> } = {};

async function initialize(): Promise<ParseScene> {
  await init({ module_or_path: wasmUrl });
  return parseScene;
}

/**
 * RustのWasmを読み込み，シーンを読む関数を返す．
 * 最初の呼び出しで読み込みを始め，以降は，同じ読み込みの結果を返す．
 */
function loadSceneParser(): Promise<ParseScene> {
  state.loading ??= initialize();
  return state.loading;
}

export { loadSceneParser };
export type { ParseScene };
export type { SceneObject, SceneOutcome } from '@/wasm/figure';
