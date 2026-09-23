import { useState } from 'react';
import type { ReactElement } from 'react';

import type { Rendered } from '@/components/figure/render-figure';
import { NO_SELECTION, indexOfId, rotateView, viewKind } from '@/figure-editor/draft';
import type { SceneDraft } from '@/figure-editor/draft';

import { FormPanel } from './form-panel';
import { JsonPanel, TikzPanel } from './output-panel';
import { OutputPreview } from './output-preview';
import { Preview } from './preview';
import { Toolbar } from './toolbar';
import { useDraft } from './use-draft';
import { useFigureRenders } from './use-figure-renders';

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
  /** 実際の出力と同じ，そのままの図． */
  rendered: Rendered | undefined;
  /** 編集の補助を加えた図． */
  editRendered: Rendered | undefined;
  failed: boolean;
  onPickObject: (id: string) => void;
  onRotate: (deltaAzimuth: number, deltaElevation: number) => void;
}

/**
 * 編集中の図(補助つき，操作できる)と，実際の出力のプレビュー(補助なし)を分けて示し，タブで切り替える操作の欄．
 */
function Body({
  rendered,
  editRendered,
  failed,
  onPickObject,
  onRotate,
  ...controls
}: BodyProps): ReactElement {
  const kind = viewKind(controls.draft);
  return (
    <div className="fe-body">
      <div className="fe-preview-stack">
        <Preview
          rendered={editRendered}
          failed={failed}
          kind={kind}
          onPickObject={onPickObject}
          onRotate={onRotate}
        />
        <OutputPreview rendered={rendered} failed={failed} />
      </div>
      <Controls {...controls} />
    </div>
  );
}

/**
 * 図の作成．オブジェクトを足して設定すると，図のシーン(JSON)ができ，RustのエンジンをWasmで動かして描く．
 * シーンのJSONの読み込みと書き出し，対応するTikZの書き出しができる．
 */
function FigureEditor(): ReactElement {
  const { draft, revision, setDraft, load, insert } = useDraft();
  const [tab, setTab] = useState<Tab>('form');
  const [selected, setSelected] = useState(NO_SELECTION);
  const [message, setMessage] = useState('');
  const { json, rendered, editRendered, failed } = useFigureRenders(draft);
  const replace = (next: SceneDraft): void => {
    load(next);
    setSelected(NO_SELECTION);
  };
  return (
    <div className="figure-editor not-content">
      <Toolbar
        kind={viewKind(draft)}
        json={json}
        tikz={rendered?.tikz ?? ''}
        message={message}
        onMessage={setMessage}
        onLoad={replace}
        onNew={() => {
          replace({ ...draft, objects: [] });
        }}
        onInsert={insert}
      />
      <Body
        rendered={rendered}
        editRendered={editRendered}
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
