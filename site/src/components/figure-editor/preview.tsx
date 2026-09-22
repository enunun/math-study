import type { ReactElement } from 'react';

import type { Rendered } from '@/components/figure/render-figure';
import type { ViewKind } from '@/figure-editor/draft';

import { useDragRotate } from './use-drag-rotate';

interface Props {
  /** エンジンを読み込むまでは，`undefined`． */
  rendered: Rendered | undefined;
  /** エンジンを読み込めなかったか． */
  failed: boolean;
  kind: ViewKind;
  /** 誤りのあるオブジェクトを選ぶ． */
  onPickObject: (id: string) => void;
  /** 空間の図で，プレビューをドラッグしたときの，方位角と仰角の移動量(度)． */
  onRotate: (deltaAzimuth: number, deltaElevation: number) => void;
}

/**
 * 図のプレビュー．描けないときは，エンジンが返した理由を示す．
 * 空間の図では，プレビューをドラッグして，方位角と仰角を動かせる．
 */
function Preview({ rendered, failed, kind, onPickObject, onRotate }: Props): ReactElement {
  const failure = rendered?.failure;
  const draggable = kind === 'space' && rendered?.figure !== undefined;
  const drag = useDragRotate({ enabled: draggable, onRotate });
  return (
    <div className="fe-preview">
      <h2>プレビュー</h2>
      <div className={draggable ? 'fe-preview-figure fe-draggable' : 'fe-preview-figure'} {...drag}>
        {rendered?.figure}
      </div>
      {draggable && <p className="fe-hint">ドラッグすると，見る向きが変わる．</p>}
      {rendered === undefined && !failed && <p role="status">図を読み込んでいる．</p>}
      {failed && <p role="alert">図を描く部品を読み込めなかった．ページを開き直す．</p>}
      {failure !== undefined && (
        <div className="fe-error" role="alert">
          <p>{failure.message}</p>
          {failure.object !== null && (
            <button
              type="button"
              onClick={() => {
                onPickObject(failure.object ?? '');
              }}
            >
              「{failure.object}」を選ぶ
            </button>
          )}
        </div>
      )}
    </div>
  );
}

export { Preview };
