import { FONT_DIRECTORY } from '@/math/constants';
import { environments, macros } from '@/math/macros';

/** ブラウザで動くMathJaxの，使う部分だけの型． */
interface BrowserMathJax {
  startup: { promise: Promise<unknown> };
  tex2chtmlPromise: (
    tex: string,
    options: { display: boolean; em: number; ex: number; containerWidth: number },
  ) => Promise<HTMLElement>;
  chtmlStylesheet: () => HTMLElement;
}

const EM = 16;
const EX = 8;
const CONTAINER_WIDTH = 1280;

const state: { loading?: Promise<BrowserMathJax>; queue: Promise<unknown> } = {
  queue: Promise.resolve(),
};

/**
 * ビルド時の描画(`worker.ts`)と，同じマクロと，同じ設定を使う．
 * 読み上げの生成，メニュー，意味づけは，ブラウザでは使わない．結果の読み上げには，呼び出し側が，平文のラベルを付ける．
 */
function configure(): void {
  Reflect.set(globalThis, 'MathJax', {
    tex: {
      packages: { '[-]': ['noundefined'] },
      macros,
      environments,
      formatError: (_jax: unknown, error: Error) => {
        throw error;
      },
    },
    chtml: { fontURL: `${import.meta.env.BASE_URL}${FONT_DIRECTORY}`, displayOverflow: 'scroll' },
    startup: { typeset: false },
    options: {
      enableMenu: false,
      // メニューの設定が，個別のenable*を上書きするため，メニューの設定で，強調，読み上げ，点字を切る．
      menuOptions: {
        settings: { enrich: false, speech: false, braille: false, assistiveMml: false },
      },
    },
  });
}

async function initialize(): Promise<BrowserMathJax> {
  configure();
  // MathJaxは大きい(約1MB)ため，最初に数式が要るときまで，読み込まない．
  await import('@mathjax/src/bundle/tex-chtml.js');
  const mathJax = Reflect.get(globalThis, 'MathJax') as BrowserMathJax;
  await mathJax.startup.promise;
  return mathJax;
}

function loadMathJax(): Promise<BrowserMathJax> {
  state.loading ??= initialize();
  return state.loading;
}

async function convert(tex: string): Promise<HTMLElement> {
  const mathJax = await loadMathJax();
  const node = await mathJax.tex2chtmlPromise(tex, {
    display: true,
    em: EM,
    ex: EX,
    containerWidth: CONTAINER_WIDTH,
  });
  // 描画した文字の分のCSSを，ページに反映する．
  const stylesheet = mathJax.chtmlStylesheet();
  if (!stylesheet.isConnected) {
    document.head.append(stylesheet);
  }
  return node;
}

async function runAfter<T>(previous: Promise<unknown>, task: () => Promise<T>): Promise<T> {
  await previous;
  return task();
}

/**
 * TeXの式を，別行立ての数式の要素にする．失敗したときは，例外を投げる．
 * MathJaxは，複数の式を同時に描画する使い方を想定していないため，1つずつ処理する．
 */
function renderMath(tex: string): Promise<HTMLElement> {
  const run = runAfter(state.queue, () => convert(tex));
  state.queue = Promise.allSettled([run]);
  return run;
}

export { renderMath };
