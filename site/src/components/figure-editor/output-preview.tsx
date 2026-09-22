import type { ReactElement } from 'react';

import type { Rendered } from '@/components/figure/render-figure';

interface Props {
  /** エンジンを読み込むまでは，`undefined`． */
  rendered: Rendered | undefined;
  /** エンジンを読み込めなかったか． */
  failed: boolean;
}

/**
 * 実際の出力(TikZやJSON)と同じ図．編集の補助(格子や座標軸のスタイル，ワイヤーフレーム)を含まず，
 * ドラッグもできない，そのままのプレビューである．
 */
function OutputPreview({ rendered, failed }: Props): ReactElement {
  const failure = rendered?.failure;
  return (
    <div className="fe-preview fe-output-preview">
      <h2>プレビュー</h2>
      <div className="fe-preview-figure">{rendered?.figure}</div>
      {rendered === undefined && !failed && <p role="status">図を読み込んでいる．</p>}
      {failed && <p role="alert">図を描く部品を読み込めなかった．ページを開き直す．</p>}
      {failure !== undefined && <p role="alert">上の「編集中の図」と同じ理由で描けない．</p>}
    </div>
  );
}

export { OutputPreview };
