import type { ReactElement } from 'react';

import type { Rendered } from '@/components/figure/render-figure';
import type { SceneDraft, ViewKind } from '@/figure-editor/draft';
import { slidersOf } from '@/figure-editor/sliders';

import { ParameterSliders } from './parameter-sliders';
import type { Update } from './parameter-sliders';
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
  /** 図．範囲のある媒介変数を，図の下のスライダーにする． */
  draft: SceneDraft;
  /** 今の図から次の図を作る(スライダー)． */
  onUpdate: Update;
}

/**
 * 編集中の図．格子と座標軸は，実際に描かれるオブジェクトと見分けやすいよう，補助のスタイル(灰色の点線)で示し，
 * 平面の図では，座標軸に読みやすい目盛も足す．空間の図では，プレビューをドラッグして，見る向きを変えられる．
 * 実際の色や線の種類，見た目は，下の「プレビュー」で確かめる．曲面のワイヤーフレームは，シーン自身の項目
 * (フォームの「ワイヤーフレームを表示」)で選ぶので，両方の図に，そのまま現れる．
 */
function Preview({
  rendered,
  failed,
  kind,
  onPickObject,
  onRotate,
  draft,
  onUpdate,
}: Props): ReactElement {
  const sliders = slidersOf(draft);
  const failure = rendered?.failure;
  const draggable = kind === 'space' && rendered?.figure !== undefined;
  const drag = useDragRotate({ enabled: draggable, onRotate });
  return (
    <div className="fe-preview fe-edit-preview">
      <h2>編集中の図</h2>
      <div className={draggable ? 'fe-preview-figure fe-draggable' : 'fe-preview-figure'} {...drag}>
        {rendered?.figure}
      </div>
      {sliders.length > 0 && <ParameterSliders sliders={sliders} onUpdate={onUpdate} />}
      <p className="fe-hint">
        格子と座標軸は，補助として灰色の点線と目盛で示す．実際の見た目は，下の「プレビュー」で確かめる．
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
