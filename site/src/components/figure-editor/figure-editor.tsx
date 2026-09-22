import { useMemo, useState } from 'react';
import type { ReactElement } from 'react';

import { renderFigure } from '@/components/figure/render-figure';
import { useLoaded } from '@/components/use-loaded';
import {
  NO_SELECTION,
  indexOfId,
  rotateView,
  stringifyDraft,
  viewKind,
} from '@/figure-editor/draft';
import type { SceneDraft } from '@/figure-editor/draft';
import { loadSceneEngine } from '@/figure/wasm';
import type { SceneEngine } from '@/figure/wasm';

import { FormPanel } from './form-panel';
import { JsonPanel, TikzPanel } from './output-panel';
import { Preview } from './preview';
import { Toolbar } from './toolbar';
import { useDraft } from './use-draft';

import './figure-editor.css';

type Tab = 'form' | 'json' | 'tikz';

const TABS: readonly (readonly [Tab, string])[] = [
  ['form', 'フォーム'],
  ['json', 'JSON'],
  ['tikz', 'TikZ'],
];

/** 表示の切り替え． */
function Tabs({ tab, onTab }: { tab: Tab; onTab: (tab: Tab) => void }): ReactElement {
  return (
    <div className="fe-tabs" role="group" aria-label="表示の切り替え">
      {TABS.map(([id, label]) => (
        <button
          key={id}
          type="button"
          aria-pressed={tab === id}
          onClick={() => {
            onTab(id);
          }}
        >
          {label}
        </button>
      ))}
    </div>
  );
}

/** 描いた結果．エンジンを読み込むまでは，`rendered`が`undefined`である． */
function useRendered(json: string): {
  rendered: ReturnType<typeof renderFigure> | undefined;
  failed: boolean;
} {
  const engine = useLoaded<SceneEngine>(loadSceneEngine);
  const rendered = useMemo(
    () => (engine.value === undefined ? undefined : renderFigure(engine.value, json)),
    [engine.value, json],
  );
  return { rendered, failed: engine.failed };
}

interface ControlsProps {
  tab: Tab;
  onTab: (tab: Tab) => void;
  draft: SceneDraft;
  revision: number;
  onDraft: (draft: SceneDraft) => void;
  tikz: string;
  invalidId: string;
  selected: number;
  onSelect: (index: number) => void;
}

/** タブの切り替えと，選んだタブの中身(フォーム，JSON，TikZ)． */
function Controls({
  tab,
  onTab,
  draft,
  revision,
  onDraft,
  tikz,
  invalidId,
  selected,
  onSelect,
}: ControlsProps): ReactElement {
  return (
    <div className="fe-controls">
      <Tabs tab={tab} onTab={onTab} />
      {tab === 'form' && (
        <FormPanel
          draft={draft}
          onDraft={onDraft}
          invalidId={invalidId}
          selected={selected}
          onSelect={onSelect}
        />
      )}
      {tab === 'json' && <JsonPanel key={revision} draft={draft} onChange={onDraft} />}
      {tab === 'tikz' && <TikzPanel tikz={tikz} />}
    </div>
  );
}

interface BodyProps extends ControlsProps {
  rendered: ReturnType<typeof renderFigure> | undefined;
  failed: boolean;
  onPickObject: (id: string) => void;
  onRotate: (deltaAzimuth: number, deltaElevation: number) => void;
}

/** プレビューと，タブで切り替える操作の欄． */
function Body({ rendered, failed, onPickObject, onRotate, ...controls }: BodyProps): ReactElement {
  return (
    <div className="fe-body">
      <Preview
        rendered={rendered}
        failed={failed}
        kind={viewKind(controls.draft)}
        onPickObject={onPickObject}
        onRotate={onRotate}
      />
      <Controls {...controls} />
    </div>
  );
}

/**
 * 図の作成．オブジェクトを足して設定すると，図のシーン(JSON)ができ，RustのエンジンをWasmで動かして描く．
 * シーンのJSONの読み込みと書き出し，対応するTikZの書き出しができる．
 */
function FigureEditor(): ReactElement {
  const { draft, revision, setDraft, load } = useDraft();
  const [tab, setTab] = useState<Tab>('form');
  const [selected, setSelected] = useState(NO_SELECTION);
  const [message, setMessage] = useState('');
  const json = useMemo(() => stringifyDraft(draft), [draft]);
  const { rendered, failed } = useRendered(json);
  const replace = (next: SceneDraft): void => {
    load(next);
    setSelected(NO_SELECTION);
  };
  return (
    <div className="figure-editor not-content">
      <Toolbar
        json={json}
        tikz={rendered?.tikz ?? ''}
        message={message}
        onMessage={setMessage}
        onLoad={replace}
        onNew={() => {
          replace({ ...draft, objects: [] });
        }}
      />
      <Body
        rendered={rendered}
        failed={failed}
        onPickObject={(id) => {
          setSelected(indexOfId(draft, id));
          setTab('form');
        }}
        onRotate={(deltaAzimuth, deltaElevation) => {
          setDraft(rotateView(draft, deltaAzimuth, deltaElevation));
        }}
        tab={tab}
        onTab={setTab}
        draft={draft}
        revision={revision}
        onDraft={setDraft}
        tikz={rendered?.tikz ?? ''}
        invalidId={rendered?.failure?.object ?? ''}
        selected={selected}
        onSelect={setSelected}
      />
    </div>
  );
}

export { FigureEditor };
