import { readFile } from 'node:fs/promises';
import path from 'node:path';

import type { Element, Root } from 'hast';
import type { Plugin } from 'unified';

import { figureToHast } from '../figure/svg';
import init, { renderScene } from '../wasm/figure';
import { DocumentError, isJsxElement, readStringAttribute } from './statements/tree';
import type { JsxElement, TreeNode } from './statements/tree';

interface Options {
  /** 図のシーン(JSON)を置くディレクトリ． */
  figuresDirectory: string;
  /** シーンを描画するWasmのファイル． */
  wasmPath: string;
}

/** 図の名前．ファイル名になるので，小文字の英数字をハイフンでつないだものに限る． */
const FIGURE_NAME = /^[a-z0-9]+(?:-[a-z0-9]+)*$/u;

const state: { ready?: Promise<void> } = {};

async function startWasm(wasmPath: string): Promise<void> {
  await init({ module_or_path: await readFile(wasmPath) });
}

/** Wasmを，最初に使うときに初期化する．プラグインは，ビルドのたびに，何度も呼ばれる． */
async function prepareWasm(wasmPath: string): Promise<void> {
  state.ready ??= startWasm(wasmPath);
  await state.ready;
}

/** 図のシーンを読む．読めないときは，図の名前を付けて，文書の誤りにする． */
async function readScene(name: string, node: JsxElement, options: Options): Promise<string> {
  const file = path.join(options.figuresDirectory, `${name}.json`);
  try {
    return await readFile(file, 'utf8');
  } catch {
    throw new DocumentError(`図「${name}」のシーン(${file})を読めない．`, node.position?.start);
  }
}

/** 名前の図を描く．シーンの誤りは，図の名前を付けて，文書の誤りにする． */
async function drawFigure(name: string, node: JsxElement, options: Options): Promise<Element> {
  const json = await readScene(name, node, options);
  await prepareWasm(options.wasmPath);
  const outcome = renderScene(json);
  if (outcome.status === 'error') {
    throw new DocumentError(`図「${name}」：${outcome.message}`, node.position?.start);
  }
  return figureToHast(outcome.figure);
}

function figureName(node: JsxElement): string {
  const name = readStringAttribute(node, 'src');
  if (name === undefined) {
    throw new DocumentError('<Figure>には，図の名前(src="…")を書く．', node.position?.start);
  }
  if (!FIGURE_NAME.test(name)) {
    throw new DocumentError(
      `図の名前「${name}」は，小文字の英数字を，ハイフンでつないだものにする．`,
      node.position?.start,
    );
  }
  return name;
}

/** 子の中の`<Figure>`を，描いた図に置き換える． */
async function replaceFigures(node: TreeNode, options: Options): Promise<void> {
  // 葉の節に，空の`children`を作らないよう，子がある節だけを書き換える．
  if (node.children === undefined) {
    return;
  }
  node.children = await Promise.all(
    node.children.map(async (child) => {
      if (isJsxElement(child) && child.name === 'Figure') {
        return drawFigure(figureName(child), child, options);
      }
      await replaceFigures(child, options);
      return child;
    }),
  );
}

/**
 * `<Figure src="図の名前" />`を，シーン(JSON)から描いた図に置き換える．
 * シーンは，`figuresDirectory/図の名前.json`から読み，RustのWasmで描画して，SVGとラベルにする．
 * ラベルの数式は，行内の数式として置くので，後段のMathJaxのプラグインが描画する．
 * シーンの誤りは，図の名前と原因を付けて，ビルドの失敗にする．
 */
const rehypeFigures: Plugin<[Options], Root> = (options) => async (tree, file) => {
  try {
    await replaceFigures(tree, options);
  } catch (error) {
    if (error instanceof DocumentError) {
      file.fail(error.message, { place: error.place, ruleId: 'rehype-figures' });
    }
    throw error;
  }
};

export { rehypeFigures };
export type { Options };
