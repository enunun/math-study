import { useState } from 'react';
import type { ReactElement } from 'react';

import { OBJECT_TYPES, typesFor } from '@/figure-editor/create';
import type { ViewKind } from '@/figure-editor/draft';
import { stringOf } from '@/figure-editor/json';
import type { JsonObject } from '@/figure-editor/json';

import { SelectInput } from './field-inputs';

/** オブジェクトの種類の，日本語の名前． */
function typeLabel(object: JsonObject): string {
  const type = stringOf(object, 'type');
  const bezier = type === 'surface' && 'bezier' in object;
  return OBJECT_TYPES.find((entry) => entry.type === (bezier ? 'bezier' : type))?.label ?? type;
}

interface RowProps {
  object: JsonObject;
  index: number;
  selected: boolean;
  invalid: boolean;
  onSelect: (index: number) => void;
  onMove: (index: number, delta: number) => void;
  onRemove: (index: number) => void;
}

/** 一覧の1行の，小さいボタン． */
function RowButton({
  label,
  onClick,
  children,
}: {
  label: string;
  onClick: () => void;
  children: string;
}): ReactElement {
  return (
    <button type="button" aria-label={label} onClick={onClick}>
      {children}
    </button>
  );
}

/** 一覧の1行．選ぶ，前後へ動かす，削除する． */
function ObjectRow({
  object,
  index,
  selected,
  invalid,
  onSelect,
  onMove,
  onRemove,
}: RowProps): ReactElement {
  const id = stringOf(object, 'id');
  return (
    <li className={selected ? 'fe-selected' : ''}>
      <button
        type="button"
        className="fe-pick"
        aria-current={selected}
        aria-invalid={invalid}
        onClick={() => {
          onSelect(index);
        }}
      >
        {typeLabel(object)}「{id}」
      </button>
      <RowButton
        label={`「${id}」を1つ前へ`}
        onClick={() => {
          onMove(index, -1);
        }}
      >
        ↑
      </RowButton>
      <RowButton
        label={`「${id}」を1つ後ろへ`}
        onClick={() => {
          onMove(index, 1);
        }}
      >
        ↓
      </RowButton>
      <RowButton
        label={`「${id}」を削除`}
        onClick={() => {
          onRemove(index);
        }}
      >
        削除
      </RowButton>
    </li>
  );
}

/** 追加するオブジェクトの種類を選ぶ欄と，追加のボタン． */
function AddObject({
  kind,
  onAdd,
}: {
  kind: ViewKind;
  onAdd: (type: string) => void;
}): ReactElement {
  const types = typesFor(kind);
  const [type, setType] = useState('point');
  const chosen = types.some((entry) => entry.type === type) ? type : (types[0]?.type ?? '');
  return (
    <div className="fe-add">
      <SelectInput
        label="追加する種類"
        value={chosen}
        options={types.map((entry) => [entry.type, entry.label] as const)}
        onChange={setType}
      />
      <button
        type="button"
        onClick={() => {
          onAdd(chosen);
        }}
      >
        追加
      </button>
    </div>
  );
}

interface Props {
  objects: readonly JsonObject[];
  kind: ViewKind;
  /** 選んだオブジェクトの添字．選んでいなければ，`NO_SELECTION`． */
  selected: number;
  /** 誤りのあるオブジェクトの識別子．なければ空の文字列． */
  invalidId: string;
  onSelect: (index: number) => void;
  onMove: (index: number, delta: number) => void;
  onRemove: (index: number) => void;
  onAdd: (type: string) => void;
}

/** オブジェクトの一覧と，追加の操作．並びは，描く順と，参照の順(点を先に置く)である． */
function ObjectList({ objects, kind, selected, invalidId, onAdd, ...rows }: Props): ReactElement {
  return (
    <div className="fe-list">
      <ul aria-label="オブジェクトの一覧">
        {objects.map((object, index) => (
          <ObjectRow
            key={`${stringOf(object, 'id')}-${index}`}
            object={object}
            index={index}
            selected={index === selected}
            invalid={invalidId !== '' && stringOf(object, 'id') === invalidId}
            {...rows}
          />
        ))}
      </ul>
      {objects.length === 0 && <p>オブジェクトがない．下の「追加」で足す．</p>}
      <AddObject kind={kind} onAdd={onAdd} />
    </div>
  );
}

export { ObjectList, typeLabel };
