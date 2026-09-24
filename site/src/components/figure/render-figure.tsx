import type { ReactNode } from 'react';

import { hastToReact } from '@/figure/hast-react';
import { figureToHast } from '@/figure/svg';
import type { SceneEngine } from '@/figure/wasm';
import type { Figure } from '@/wasm/figure';

import { InlineMath } from './inline-math';

/** 描画の結果．描けたときは図の要素，描けなかったときは誤りを持つ． */
interface Rendered {
  figure: ReactNode;
  failure: Extract<ReturnType<SceneEngine['renderScene']>, { status: 'error' }> | undefined;
  tikz: string | undefined;
  /** エンジンが出した中間表現．画像の書き出しに使う． */
  ir: Figure | undefined;
}

/**
 * シーン(JSON)を，Rustのエンジンで描画して，図の要素にする．ラベルは，ブラウザのMathJaxで組む．
 * 描けないシーンは，エンジンの誤りを返す．
 */
function renderFigure(engine: SceneEngine, json: string): Rendered {
  const outcome = engine.renderScene(json);
  if (outcome.status !== 'ok') {
    return { figure: undefined, failure: outcome, tikz: undefined, ir: undefined };
  }
  const figure = hastToReact(figureToHast(outcome.figure), 'figure', (element, key) => {
    const [text] = element.children;
    if (element.tagName === 'code' && text?.type === 'text') {
      return <InlineMath key={key} tex={text.value} />;
    }
    return false;
  });
  return { figure, failure: undefined, tikz: outcome.tikz, ir: outcome.figure };
}

export { renderFigure };
export type { Rendered };
