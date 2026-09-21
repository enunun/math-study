import type { ReactElement } from 'react';

import { listNames } from '@/figure-editor/fields';
import { arrayOf, stringOf, withField } from '@/figure-editor/json';
import { boundToText, commitField, parseItem } from '@/figure-editor/values';

import { CheckboxInput, ListInput, SelectInput, TextInput } from './field-inputs';
import type { FieldProps } from './field-props';

/** 文字列，数，数か式の入力欄． */
function ScalarField({
  spec,
  object,
  onChange,
}: FieldProps<'text' | 'number' | 'bound'>): ReactElement {
  return (
    <TextInput
      label={spec.label}
      value={boundToText(object[spec.key])}
      onChange={(value) => {
        onChange(
          commitField({
            object,
            key: spec.key,
            value: parseItem(spec.kind, value),
            optional: spec.optional,
          }),
        );
      }}
    />
  );
}

/** 選択肢から選ぶ入力欄．省略できる項目には，「既定」を出す． */
function SelectField({ spec, object, onChange }: FieldProps<'select'>): ReactElement {
  return (
    <SelectInput
      label={spec.label}
      value={stringOf(object, spec.key)}
      options={spec.options}
      empty={spec.optional === true ? '既定' : ''}
      onChange={(value) => {
        onChange(commitField({ object, key: spec.key, value, optional: spec.optional }));
      }}
    />
  );
}

/** 真偽の入力欄．初めの値と同じなら，項目を取り除く． */
function CheckboxField({ spec, object, onChange }: FieldProps<'checkbox'>): ReactElement {
  const current = object[spec.key];
  return (
    <CheckboxInput
      label={spec.label}
      checked={current === undefined ? spec.initial : current === true}
      onChange={(checked) => {
        onChange(withField(object, spec.key, checked === spec.initial ? undefined : checked));
      }}
    />
  );
}

/** 決まった数の，数か式を並べる入力欄． */
function ListField({ spec, object, kind, onChange }: FieldProps<'list'>): ReactElement {
  return (
    <ListInput
      label={spec.label}
      names={listNames(spec, kind)}
      item={spec.item}
      value={arrayOf(object, spec.key)}
      onChange={(value) => {
        onChange(commitField({ object, key: spec.key, value, optional: spec.optional }));
      }}
    />
  );
}

export { CheckboxField, ListField, ScalarField, SelectField };
