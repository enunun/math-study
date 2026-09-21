import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

/** 描画した図．`tikz`は，Wasmが出したTikZで，`bounds`は，描く範囲(cm)である． */
interface Rendered {
  name: string;
  bounds: { min: [number, number]; max: [number, number] };
  tikz: string;
}

/** 描画する図の入力．`name`は，出力のファイル名に使い，`file`は，シーンのJSONである． */
interface SceneSource {
  name: string;
  file: string;
}

/** Wasmの窓口のうち，ここで使う部分．生成されたファイルは，実行時に読み込む． */
interface FigureWasm {
  initSync: (options: { module: Buffer }) => void;
  renderScene: (
    json: string,
  ) =>
    | { status: 'ok'; figure: { bounds: Rendered['bounds'] }; tikz: string }
    | { status: 'error'; message: string };
}

const FIGURES_DIR = 'site/src/figures';

/** MDXに書かれた，`<Figure src="…" />`の名前を，書かれた順に返す． */
async function figureNames(mdx: string): Promise<string[]> {
  const source = await readFile(mdx, 'utf8');
  return [...source.matchAll(/<Figure src="(?<name>[a-z0-9-]+)"/gu)].flatMap((found) =>
    found.groups?.name === undefined ? [] : [found.groups.name],
  );
}

/** 図の名前(`site/src/figures/`のJSON)か，JSONのパスから，描画する図の入力を作る． */
function sceneSource(argument: string): SceneSource {
  if (argument.endsWith('.json') || argument.includes('/')) {
    return { name: path.basename(argument, '.json'), file: argument };
  }
  return { name: argument, file: path.join(FIGURES_DIR, `${argument}.json`) };
}

/** 読み込んだものが，使うWasmの窓口を持っているか． */
function isFigureWasm(value: unknown): value is FigureWasm {
  return (
    typeof value === 'object' &&
    value !== null &&
    'initSync' in value &&
    typeof value.initSync === 'function' &&
    'renderScene' in value &&
    typeof value.renderScene === 'function'
  );
}

async function loadWasm(): Promise<FigureWasm> {
  const wasm: unknown = await import(pathToFileURL(path.resolve('site/src/wasm/figure.js')).href);
  if (!isFigureWasm(wasm)) {
    throw new Error(
      'site/src/wasm/figure.jsに，initSyncとrenderSceneがない．mise run wasmを実行する',
    );
  }
  wasm.initSync({ module: await readFile('site/src/wasm/figure_bg.wasm') });
  return wasm;
}

/** シーンのJSONを，Wasmで描画して，TikZの文字列と描く範囲を得る． */
async function render(wasm: FigureWasm, source: SceneSource): Promise<Rendered> {
  const outcome = wasm.renderScene(await readFile(source.file, 'utf8'));
  if (outcome.status !== 'ok') {
    throw new Error(`${source.name}を描画できない：${outcome.message}`);
  }
  return { name: source.name, bounds: outcome.figure.bounds, tikz: outcome.tikz };
}

export { figureNames, loadWasm, render, sceneSource };
export type { FigureWasm, Rendered, SceneSource };
