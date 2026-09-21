import type { ReactElement } from 'react';

import type { Rendered } from '@/components/figure/render-figure';

interface Props {
  /** エンジンを読み込むまでは，`undefined`． */
  rendered: Rendered | undefined;
  /** エンジンを読み込めなかったか． */
  failed: boolean;
  /** 誤りのあるオブジェクトを選ぶ． */
  onPickObject: (id: string) => void;
}

/** 図のプレビュー．描けないときは，エンジンが返した理由を示す． */
function Preview({ rendered, failed, onPickObject }: Props): ReactElement {
  const failure = rendered?.failure;
  return (
    <div className="fe-preview">
      <h2>プレビュー</h2>
      {rendered?.figure}
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
