import { useEffect, useRef } from 'react';
import type { ReactElement } from 'react';

import { renderMath } from '@/calculator/mathjax';

interface Props {
  /** 描画するTeX． */
  tex: string;
  /** 数式を読めない人のための，平文の表記．読み上げと，描画前の表示に使う． */
  label: string;
}

/**
 * TeXの式を，ブラウザのMathJaxで，別行立ての数式として描画する．
 * 描画するまでは，平文の表記を表示する．描画は非同期で，式が変わったときは，古い描画の結果を捨てる．
 */
function MathView({ tex, label }: Props): ReactElement {
  const host = useRef<HTMLDivElement>(null);
  useEffect(() => {
    let current = true;
    const draw = async (): Promise<void> => {
      try {
        const node = await renderMath(tex);
        if (current) {
          host.current?.replaceChildren(node);
        }
      } catch {
        // 描画に失敗したときは，平文の表記が，そのまま残る．
      }
    };
    void draw();
    return (): void => {
      current = false;
    };
  }, [tex]);
  // 中身は，MathJaxが差し替えるため，Reactの子は置かない．描画前の表示は，CSSが，data-fallbackから出す．
  return (
    <div
      ref={host}
      className="math-view"
      role="img"
      aria-label={label}
      data-fallback={label}
      tabIndex={0}
    />
  );
}

export { MathView };
