import type { ReactElement } from 'react';

import type { ViewKind } from '@/figure-editor/draft';
import { objectOf, withField } from '@/figure-editor/json';
import type { JsonObject } from '@/figure-editor/json';

import { CheckboxInput, SelectInput, TextInput } from './field-inputs';

const COLORS = [
  ['gray', '灰'],
  ['red', '赤'],
  ['blue', '青'],
  ['green', '緑'],
  ['orange', '橙'],
  ['purple', '紫'],
] as const;

const LINES = [
  ['solid', '実線'],
  ['dotted', '点線'],
  ['dashed', '破線'],
] as const;

const HIDDEN = [
  ['dotted', '点線'],
  ['dashed', '破線'],
  ['none', '描かない'],
] as const;

function textOf(style: JsonObject, key: string): string {
  const value = style[key];
  return typeof value === 'string' ? value : '';
}

interface Props {
  object: JsonObject;
  kind: ViewKind;
  /** スタイルを持つ項目の名前．既定は`style`で，`wireframe`や`control_net`にも使う． */
  field?: string;
  legend?: string;
  onChange: (object: JsonObject) => void;
}

/** オブジェクトのスタイル(色，線の種類，太さ，隠れた部分の描き方)の入力欄．空にした項目は，取り除く． */
interface SetterProps {
  style: JsonObject;
  set: (key: string, value: string) => void;
}

/** 隠れた部分の描き方の入力欄．空間の図でだけ出す． */
function HiddenInput({ style, set }: SetterProps): ReactElement {
  return (
    <SelectInput
      label="隠れた部分"
      value={textOf(style, 'hidden')}
      options={HIDDEN}
      empty="既定"
      onChange={(value) => {
        set('hidden', value);
      }}
    />
  );
}

function StyleInput({
  object,
  kind,
  field = 'style',
  legend = 'スタイル',
  onChange,
}: Props): ReactElement {
  const style = objectOf(object, field);
  const set = (key: string, value: string): void => {
    const next = withField(style, key, value === '' ? undefined : value);
    onChange(withField(object, field, Object.keys(next).length === 0 ? undefined : next));
  };
  return (
    <fieldset className="fe-group">
      <legend>{legend}</legend>
      <SelectInput
        label="色"
        value={textOf(style, 'color')}
        options={COLORS}
        empty="既定"
        onChange={(value) => {
          set('color', value);
        }}
      />
      <SelectInput
        label="線の種類"
        value={textOf(style, 'line')}
        options={LINES}
        empty="既定"
        onChange={(value) => {
          set('line', value);
        }}
      />
      <TextInput
        label="線の太さ(1.2ptなど)"
        value={textOf(style, 'width')}
        onChange={(value) => {
          set('width', value);
        }}
      />
      {kind === 'space' && <HiddenInput style={style} set={set} />}
    </fieldset>
  );
}

interface ToggleProps {
  object: JsonObject;
  kind: ViewKind;
  /** あれば描く，項目の名前(`wireframe`，`control_net`など)． */
  field: string;
  label: string;
  onChange: (object: JsonObject) => void;
}

/**
 * 項目自体の有無をチェックボックスで選ぶ，スタイルの入力欄．曲面のワイヤーフレームや，ベジエ曲面の
 * 制御点の網のように，「描くかどうか」と「描くときのスタイル」を，1つの項目(あれば描く)で持つ場合に使う．
 */
function ToggleStyleInput({ object, kind, field, label, onChange }: ToggleProps): ReactElement {
  const enabled = field in object;
  return (
    <fieldset className="fe-group">
      <CheckboxInput
        label={label}
        checked={enabled}
        onChange={(checked) => {
          onChange(withField(object, field, checked ? {} : undefined));
        }}
      />
      {enabled && (
        <StyleInput
          object={object}
          kind={kind}
          field={field}
          legend={`${label}のスタイル`}
          onChange={onChange}
        />
      )}
    </fieldset>
  );
}

export { StyleInput, ToggleStyleInput };
