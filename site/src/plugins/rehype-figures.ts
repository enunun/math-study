import { readFile } from 'node:fs/promises';
import path from 'node:path';

import type { Element, ElementContent, Root } from 'hast';
import type { Plugin } from 'unified';

import { splitOnDollars } from '../figure/label';
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
/** `$`で区切った節の，偶奇．数式は奇数番目，文字は偶数番目である． */
const PARITY = 2;

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

/**
 * 図の説明文を，文字と行内の数式の並びにする．`$`で囲んだ部分は，行内の数式として置き，後段のMathJaxが描画する．
 * 節の数が偶数(`$`が閉じていない)ときは，全体を文字として扱う．
 */
function captionContent(caption: string): ElementContent[] {
  const parts = splitOnDollars(caption);
  if (parts.length % PARITY === 0) {
    return [{ type: 'text', value: caption }];
  }
  return parts.flatMap((part, index): ElementContent[] => {
    if (part === '') {
      return [];
    }
    if (index % PARITY === 0) {
      return [{ type: 'text', value: part }];
    }
    return [
      {
        type: 'element',
        tagName: 'code',
        properties: { className: ['language-math', 'math-inline'] },
        children: [{ type: 'text', value: part }],
      },
    ];
  });
}

/**
 * 番号を付けた図(`label`と`anchor`の属性がある)に，飛び先のidと，番号と説明文の見出し(`<figcaption>`)を加える．
 * 番号は，参照(`<Ref>`)のリンクの文字と同じである．
 */
function labelFigure(figure: Element, node: JsxElement): void {
  const label = readStringAttribute(node, 'label');
  const anchor = readStringAttribute(node, 'anchor');
  if (label === undefined || anchor === undefined) {
    return;
  }
  figure.properties.id = anchor;
  const caption = readStringAttribute(node, 'caption');
  figure.children.push({
    type: 'element',
    tagName: 'figcaption',
    properties: { className: ['figure-caption'] },
    children: [
      {
        type: 'element',
        tagName: 'strong',
        properties: {},
        children: [{ type: 'text', value: label }],
      },
      ...(caption === undefined
        ? []
        : [{ type: 'text' as const, value: ' ' }, ...captionContent(caption)]),
    ],
  });
}

/** 名前の図を描く．シーンの誤りは，図の名前を付けて，文書の誤りにする． */
async function drawFigure(name: string, node: JsxElement, options: Options): Promise<Element> {
  const json = await readScene(name, node, options);
  await prepareWasm(options.wasmPath);
  const outcome = renderScene(json);
  if (outcome.status === 'error') {
    throw new DocumentError(`図「${name}」：${outcome.message}`, node.position?.start);
  }
  const figure = figureToHast(outcome.figure);
  labelFigure(figure, node);
  return figure;
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
 * `<Figure src="図の名前" />`を，シーン(JSON)から描いた図に置き換える．`id`を書いた図は，番号(rehypeStatementsが付ける)と，`caption`の説明文を，図の下に置く．
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
