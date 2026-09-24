import { LiteElement } from '@mathjax/src/js/adaptors/lite/Element.js';
import { liteAdaptor } from '@mathjax/src/js/adaptors/liteAdaptor.js';
import { RegisterHTMLHandler as registerHtmlHandler } from '@mathjax/src/js/handlers/html.js';
import { TeX } from '@mathjax/src/js/input/tex.js';
import { mathjax } from '@mathjax/src/js/mathjax.js';
import { SVG } from '@mathjax/src/js/output/svg.js';

import { environments, macros } from '@/math/macros';

import { FONT_FILES } from './label-font-files';
import { LABEL_TEX_PACKAGES } from './label-tex-packages';

/**
 * 図のラベルを，MathJaxのSVG出力で組む．画像として書き出す図に埋め込むため，文字の形を，パスとして持つ(`fontCache: 'none'`)．
 * ページの数式(CHTML，`calculator/mathjax.ts`)とは別の，MathJaxの文書を作る．マクロは，同じものを使う．
 */

/** 組んだラベル．MathJaxが出したSVGの中身と，その座標の範囲．大きさは，`em`単位で持つ． */
interface LabelSvg {
  /** SVGの要素の中身(文字の形のパスなど)の文字列． */
  content: string;
  /** 中身の座標の範囲(`viewBox`の値)．1emが1000で，基準線がyの0である． */
  viewBox: string;
  /** 幅(em)． */
  width: number;
  /** 高さ(em)．基準線の上と下の合計である． */
  height: number;
}

/** MathJaxのSVGの`viewBox`の単位．1emが，1000である． */
const UNITS_PER_EM = 1000;
/** `viewBox`の数の並び(左，上，幅，高さ)． */
const VIEW_BOX_LENGTH = 4;
const WIDTH_INDEX = 2;
const HEIGHT_INDEX = 3;

/** MathJaxが求める，`@mathjax/mathjax-newcm-font/js/svg/dynamic/latin.js`のような名前のファイルを，一覧から探して読み込む． */
function loadDynamic(name: string): Promise<unknown> {
  const file = name.slice(name.lastIndexOf('/') + 1).replace(/\.js$/u, '');
  const load = Object.hasOwn(FONT_FILES, file) ? FONT_FILES[file] : undefined;
  if (load === undefined) {
    return Promise.reject(new Error(`MathJaxのファイルがない：${name}`));
  }
  return load();
}

interface Converter {
  convert: (tex: string) => Promise<LabelSvg>;
}

const state: { converter?: Converter; queue: Promise<unknown> } = { queue: Promise.resolve() };

/** `viewBox`の数を読む．4つの有限の数でなければ，失敗にする． */
function readViewBox(text: unknown, source: string): number[] {
  const numbers = String(text).split(' ').map(Number);
  if (numbers.length !== VIEW_BOX_LENGTH || numbers.some((value) => !Number.isFinite(value))) {
    throw new Error(`MathJaxのSVGの大きさを読めない：${source}`);
  }
  return numbers;
}

function createConverter(): Converter {
  mathjax.asyncLoad = loadDynamic;
  const adaptor = liteAdaptor();
  registerHtmlHandler(adaptor);
  const tex = new TeX({
    packages: LABEL_TEX_PACKAGES,
    macros,
    environments,
    formatError: (_jax: unknown, error: Error): never => {
      throw error;
    },
  });
  // MathJax 4は，行内の式を演算子の位置で分け，SVGを複数に分けて出す．ラベルは1つのSVGとして置くので，分けない．
  const document = mathjax.document('', {
    InputJax: tex,
    OutputJax: new SVG({ fontCache: 'none', linebreaks: { inline: false } }),
  });
  return {
    convert: async (source) => {
      const container: unknown = await document.convertPromise(source, { display: false });
      const svg = container instanceof LiteElement ? adaptor.firstChild(container) : undefined;
      if (!(svg instanceof LiteElement)) {
        throw new TypeError(`MathJaxのSVGがない：${source}`);
      }
      const viewBox = readViewBox(adaptor.getAttribute(svg, 'viewBox'), source);
      return {
        content: adaptor.innerHTML(svg),
        viewBox: viewBox.join(' '),
        width: (viewBox[WIDTH_INDEX] ?? 0) / UNITS_PER_EM,
        height: (viewBox[HEIGHT_INDEX] ?? 0) / UNITS_PER_EM,
      };
    },
  };
}

async function runAfter<T>(previous: Promise<unknown>, task: () => Promise<T>): Promise<T> {
  await previous;
  return task();
}

/**
 * TeXの式(`labelToMath`の結果)を，SVGにする．失敗したときは，例外を投げる．
 * MathJaxは，複数の式を同時に組む使い方を想定していないため，1つずつ処理する．
 */
function labelSvg(tex: string): Promise<LabelSvg> {
  const run = runAfter(state.queue, () => {
    state.converter ??= createConverter();
    return state.converter.convert(tex);
  });
  state.queue = Promise.allSettled([run]);
  return run;
}

export { labelSvg };
export type { LabelSvg };
