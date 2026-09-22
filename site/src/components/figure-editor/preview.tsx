import type { ReactElement } from 'react';

import type { Rendered } from '@/components/figure/render-figure';
import type { ViewKind } from '@/figure-editor/draft';

import { CheckboxInput } from './field-inputs';
import { useDragRotate } from './use-drag-rotate';

interface Props {
  /** エンジンを読み込むまでは，`undefined`． */
  rendered: Rendered | undefined;
  /** エンジンを読み込めなかったか． */
  failed: boolean;
  kind: ViewKind;
  /** 曲面のワイヤーフレームを示すか(空間の図だけ)． */
  wireframe: boolean;
  onWireframeChange: (wireframe: boolean) => void;
  /** 誤りのあるオブジェクトを選ぶ． */
  onPickObject: (id: string) => void;
  /** 空間の図で，プレビューをドラッグしたときの，方位角と仰角の移動量(度)． */
  onRotate: (deltaAzimuth: number, deltaElevation: number) => void;
}

/**
 * 編集中の図．格子と座標軸は，実際に描かれるオブジェクトと見分けやすいよう，補助のスタイル(灰色の点線)で示す．
 * 空間の図では，曲面ごとに，範囲を目で追える補助のワイヤーフレームを選んで足せ，プレビューをドラッグして
 * 見る向きも変えられる．実際の色や線の種類，見た目は，下の「プレビュー」で確かめる．
 */
function Preview({
  rendered,
  failed,
  kind,
  wireframe,
  onWireframeChange,
  onPickObject,
  onRotate,
}: Props): ReactElement {
  const failure = rendered?.failure;
  const draggable = kind === 'space' && rendered?.figure !== undefined;
  const drag = useDragRotate({ enabled: draggable, onRotate });
  return (
    <div className="fe-preview fe-edit-preview">
      <h2>編集中の図</h2>
      {kind === 'space' && (
        <CheckboxInput
          label="曲面のワイヤーフレームを表示"
          checked={wireframe}
          onChange={onWireframeChange}
        />
      )}
      <div className={draggable ? 'fe-preview-figure fe-draggable' : 'fe-preview-figure'} {...drag}>
        {rendered?.figure}
      </div>
      <p className="fe-hint">
        格子と座標軸は，補助として灰色の点線で示す．実際の見た目は，下の「プレビュー」で確かめる．
      </p>
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
