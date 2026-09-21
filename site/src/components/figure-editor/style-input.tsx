import type { ReactElement } from 'react';

import type { ViewKind } from '@/figure-editor/draft';
import { objectOf, withField } from '@/figure-editor/json';
import type { JsonObject } from '@/figure-editor/json';

import { SelectInput, TextInput } from './field-inputs';

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
  onChange: (object: JsonObject) => void;
}

/** オブジェクトの`style`(色，線の種類，太さ，隠れた部分の描き方)の入力欄．空にした項目は，取り除く． */
function StyleInput({ object, kind, onChange }: Props): ReactElement {
  const style = objectOf(object, 'style');
  const set = (key: string, value: string): void => {
    const next = withField(style, key, value === '' ? undefined : value);
    onChange(withField(object, 'style', Object.keys(next).length === 0 ? undefined : next));
  };
  return (
    <fieldset className="fe-group">
      <legend>スタイル</legend>
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
      {kind === 'space' && (
        <SelectInput
          label="隠れた部分"
          value={textOf(style, 'hidden')}
          options={HIDDEN}
          empty="既定"
          onChange={(value) => {
            set('hidden', value);
          }}
        />
      )}
    </fieldset>
  );
}

export { StyleInput };
