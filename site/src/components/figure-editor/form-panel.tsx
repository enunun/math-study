import type { ReactElement } from 'react';

import { createObject } from '@/figure-editor/create';
import {
  NO_SELECTION,
  addObject,
  moveObject,
  removeObject,
  replaceObject,
  viewKind,
} from '@/figure-editor/draft';
import type { SceneDraft } from '@/figure-editor/draft';

import { ObjectForm } from './object-form';
import { ObjectList } from './object-list';
import { ViewForm } from './view-form';

interface Props {
  draft: SceneDraft;
  onDraft: (draft: SceneDraft) => void;
  /** 誤りのあるオブジェクトの識別子．なければ空の文字列． */
  invalidId: string;
  selected: number;
  onSelect: (index: number) => void;
}

/** 動かした先の添字．端を越えないようにする． */
function movedIndex(count: number, index: number, delta: number): number {
  return Math.max(0, Math.min(count - 1, index + delta));
}

/** フォームの操作．図の設定と，オブジェクトの一覧，選んだオブジェクトの入力欄． */
function FormPanel({ draft, onDraft, invalidId, selected, onSelect }: Props): ReactElement {
  const kind = viewKind(draft);
  const object = draft.objects[selected];
  return (
    <div className="fe-panel">
      <ViewForm draft={draft} onChange={onDraft} />
      <ObjectList
        objects={draft.objects}
        kind={kind}
        selected={selected}
        invalidId={invalidId}
        onSelect={onSelect}
        onMove={(index, delta) => {
          onDraft(moveObject(draft, index, delta));
          onSelect(movedIndex(draft.objects.length, index, delta));
        }}
        onRemove={(index) => {
          onDraft(removeObject(draft, index));
          onSelect(NO_SELECTION);
        }}
        onAdd={(type) => {
          onDraft(addObject(draft, createObject(type, draft, kind)));
          onSelect(draft.objects.length);
        }}
      />
      {object !== undefined && (
        <ObjectForm
          object={object}
          draft={draft}
          kind={kind}
          onChange={(next) => {
            onDraft(replaceObject(draft, selected, next));
          }}
        />
      )}
    </div>
  );
}

export { FormPanel };
