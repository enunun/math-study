import { useMemo } from 'react';

import { renderFigure } from '@/components/figure/render-figure';
import { useLoaded } from '@/components/use-loaded';
import { stringifyDraft } from '@/figure-editor/draft';
import type { SceneDraft } from '@/figure-editor/draft';
import { buildEditScene } from '@/figure-editor/edit-scene';
import { loadSceneEngine } from '@/figure/wasm';
import type { SceneEngine } from '@/figure/wasm';

type Rendered = ReturnType<typeof renderFigure>;

interface FigureRenders {
  /** シーンのJSON(読みやすい形)．JSONの書き出しと，「JSON」タブに使う． */
  json: string;
  /** 実際の出力(TikZやJSON)と同じ，そのままの図．エンジンを読み込むまでは`undefined`である． */
  rendered: Rendered | undefined;
  /**
   * 編集の補助(格子と座標軸の補助のスタイル，曲面のワイヤーフレーム)を加えた図．
   * 補助だけが原因で描けないときは，`rendered`に落とす(補助が作った，実在しないオブジェクトの誤りを見せないため)．
   */
  editRendered: Rendered | undefined;
  /** エンジンを読み込めなかったか． */
  failed: boolean;
}

/** 実際の出力と，編集の補助を加えた図の，2とおりの描画結果． */
function useFigureRenders(draft: SceneDraft, wireframe: boolean): FigureRenders {
  const engine = useLoaded<SceneEngine>(loadSceneEngine);
  const json = useMemo(() => stringifyDraft(draft), [draft]);
  const editJson = useMemo(
    () => stringifyDraft(buildEditScene(draft, { wireframe })),
    [draft, wireframe],
  );
  const rendered = useMemo(
    () => (engine.value === undefined ? undefined : renderFigure(engine.value, json)),
    [engine.value, json],
  );
  const rawEditRendered = useMemo(
    () => (engine.value === undefined ? undefined : renderFigure(engine.value, editJson)),
    [engine.value, editJson],
  );
  const editOnlyFailure = rawEditRendered?.failure !== undefined && rendered?.failure === undefined;
  return {
    json,
    rendered,
    editRendered: editOnlyFailure ? rendered : rawEditRendered,
    failed: engine.failed,
  };
}

export { useFigureRenders };
