import type { ReactElement } from 'react';

import { changeKind } from '@/figure-editor/create';
import { viewKind } from '@/figure-editor/draft';
import type { SceneDraft, ViewKind } from '@/figure-editor/draft';
import { arrayOf, objectOf, withField } from '@/figure-editor/json';
import type { JsonObject } from '@/figure-editor/json';
import { boundToText, numberFromText } from '@/figure-editor/values';

import { ListInput, SelectInput, TextInput } from './field-inputs';

const KINDS = [
  ['plane', '平面の図'],
  ['space', '空間の図'],
] as const;

const ENDS = ['下端', '上端'] as const;

interface Props {
  draft: SceneDraft;
  onChange: (draft: SceneDraft) => void;
}

function unitText(unit: JsonObject, key: string): string {
  const value = unit[key];
  return typeof value === 'string' ? value : '';
}

/** 平面の図の，見える範囲と，1単位の実寸． */
function PlaneView({ draft, onChange }: Props): ReactElement {
  const { view } = draft;
  const unit = objectOf(view, 'unit');
  const set = (next: JsonObject): void => {
    onChange({ ...draft, view: next });
  };
  return (
    <>
      <ListInput
        label="xの範囲"
        names={ENDS}
        item="number"
        value={arrayOf(view, 'x')}
        onChange={(value) => {
          set(withField(view, 'x', value));
        }}
      />
      <ListInput
        label="yの範囲"
        names={ENDS}
        item="number"
        value={arrayOf(view, 'y')}
        onChange={(value) => {
          set(withField(view, 'y', value));
        }}
      />
      <TextInput
        label="x方向の1単位の実寸(1cmなど)"
        value={unitText(unit, 'x')}
        onChange={(value) => {
          set(withField(view, 'unit', withField(unit, 'x', value)));
        }}
      />
      <TextInput
        label="y方向の1単位の実寸(1cmなど)"
        value={unitText(unit, 'y')}
        onChange={(value) => {
          set(withField(view, 'unit', withField(unit, 'y', value)));
        }}
      />
    </>
  );
}

/** 空間の図の，見る向き(方位角と仰角)と，1単位の実寸． */
function SpaceView({ draft, onChange }: Props): ReactElement {
  const { view } = draft;
  const set = (key: string, value: string): void => {
    onChange({
      ...draft,
      view: withField(view, key, key === 'unit' ? value : numberFromText(value)),
    });
  };
  return (
    <>
      <TextInput
        label="方位角(度)"
        value={boundToText(view.azimuth)}
        onChange={(value) => {
          set('azimuth', value);
        }}
      />
      <TextInput
        label="仰角(度)"
        value={boundToText(view.elevation)}
        onChange={(value) => {
          set('elevation', value);
        }}
      />
      <TextInput
        label="1単位の実寸(1cmなど)"
        value={typeof view.unit === 'string' ? view.unit : ''}
        onChange={(value) => {
          set('unit', value);
        }}
      />
    </>
  );
}

/** 図の説明と，種類，見える範囲か見る向き． */
function ViewForm({ draft, onChange }: Props): ReactElement {
  const kind: ViewKind = viewKind(draft);
  return (
    <div className="fe-view">
      <TextInput
        label="図の説明(画像の代替テキスト)"
        value={draft.description}
        onChange={(value) => {
          onChange({ ...draft, description: value });
        }}
      />
      <SelectInput
        label="図の種類"
        value={kind}
        options={KINDS}
        onChange={(value) => {
          onChange(changeKind(draft, value === 'space' ? 'space' : 'plane'));
        }}
      />
      {kind === 'plane' ? (
        <PlaneView draft={draft} onChange={onChange} />
      ) : (
        <SpaceView draft={draft} onChange={onChange} />
      )}
    </div>
  );
}

export { ViewForm };
