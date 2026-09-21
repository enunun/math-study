import { useEffect, useRef } from 'react';
import type { ReactElement } from 'react';

import { renderMath } from '@/calculator/mathjax';

/**
 * TeXの式を，行内の数式として，ブラウザのMathJaxで描画する．図のラベルに使う．
 * 描画するまでは，式の文字列をそのまま出す(CSSが出す)．描画は非同期で，式が変わったときは，古い結果を捨てる．
 */
function InlineMath({ tex }: { tex: string }): ReactElement {
  const host = useRef<HTMLSpanElement>(null);
  useEffect(() => {
    let current = true;
    const draw = async (): Promise<void> => {
      try {
        const node = await renderMath(tex, false);
        if (current) {
          host.current?.replaceChildren(node);
        }
      } catch {
        // 描画に失敗したときは，式の文字列が，そのまま残る．
      }
    };
    void draw();
    return (): void => {
      current = false;
    };
  }, [tex]);
  // 中身は，MathJaxが差し替えるため，Reactの子は置かない．描画前の表示は，CSSが，data-fallbackから出す．
  return <span ref={host} className="inline-math" data-fallback={tex} />;
}

export { InlineMath };
