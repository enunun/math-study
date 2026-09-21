import { useState } from 'react';
import type { ReactElement } from 'react';

import type { SceneDraft } from '@/figure-editor/draft';
import { COORDINATE_NAMES } from '@/figure-editor/fields';
import { arrayOf, stringOf, withField } from '@/figure-editor/json';
import type { Json } from '@/figure-editor/json';
import { boundToText, readJsonField, setItem } from '@/figure-editor/values';

import { ListInput, PositionInput, SelectInput } from './field-inputs';
import type { FieldProps } from './field-props';

const ROWS = [0, 1] as const;
const DOMAIN_ENDS = ['下端', '上端'] as const;
const DOMAIN_ROWS = ROWS.length;

/** 指定の種類のオブジェクトの識別子を，選択肢にする． */
function idsOf(draft: SceneDraft, types: readonly string[]): (readonly [string, string])[] {
  return draft.objects
    .filter((object) => types.includes(stringOf(object, 'type')))
    .map((object) => [stringOf(object, 'id'), stringOf(object, 'id')] as const);
}

/** JSONの値を，そのまま書く入力欄．読めない間は，直前の値のままにして，理由を示す． */
function JsonField({ spec, object, onChange }: FieldProps<'json'>): ReactElement {
  const current = object[spec.key];
  const [text, setText] = useState(current === undefined ? '' : JSON.stringify(current));
  const [message, setMessage] = useState('');
  const edit = (next: string): void => {
    setText(next);
    const result = readJsonField(next, spec.optional === true);
    setMessage(result.kind === 'keep' ? result.message : '');
    if (result.kind === 'set') {
      onChange(withField(object, spec.key, result.value));
    }
  };
  return (
    <label className="fe-field fe-wide">
      <span>{spec.label}(JSON)</span>
      <textarea
        rows={3}
        value={text}
        spellCheck={false}
        placeholder={spec.hint}
        aria-invalid={message !== ''}
        onChange={(event) => {
          edit(event.target.value);
        }}
      />
      {message !== '' && <span role="alert">{message}</span>}
    </label>
  );
}

/** 範囲の`row`番目の変数の，下端と上端． */
function rowOf(rows: readonly Json[], row: number): readonly Json[] {
  const pair = rows[row];
  return Array.isArray(pair) ? pair : [];
}

/** 2つの変数の範囲． */
function DomainField({ spec, object, onChange }: FieldProps<'domain2'>): ReactElement {
  const rows = arrayOf(object, spec.key);
  return (
    <>
      {ROWS.map((row) => (
        <ListInput
          key={row}
          label={`${spec.label}(${row + 1}番目の変数)`}
          names={DOMAIN_ENDS}
          item="bound"
          value={rowOf(rows, row)}
          onChange={(pair) => {
            const next = setItem({ list: rows, index: row, item: pair, count: DOMAIN_ROWS });
            onChange(withField(object, spec.key, next));
          }}
        />
      ))}
    </>
  );
}

/** 座標か，点の式の入力欄． */
function PositionField({ spec, object, kind, onChange }: FieldProps<'position'>): ReactElement {
  return (
    <PositionInput
      label={spec.label}
      value={object[spec.key]}
      names={COORDINATE_NAMES[kind]}
      onChange={(value) => {
        onChange(withField(object, spec.key, value));
      }}
    />
  );
}

/** 別のオブジェクトを1つ選ぶ入力欄． */
function ReferenceField({ spec, object, draft, onChange }: FieldProps<'reference'>): ReactElement {
  return (
    <SelectInput
      label={spec.label}
      value={stringOf(object, spec.key)}
      options={idsOf(draft, spec.of)}
      empty="選ぶ"
      onChange={(value) => {
        onChange(withField(object, spec.key, value));
      }}
    />
  );
}

/** 別のオブジェクトを，決まった数だけ選ぶ入力欄． */
function ReferencesField({
  spec,
  object,
  draft,
  onChange,
}: FieldProps<'references'>): ReactElement {
  const current = arrayOf(object, spec.key);
  return (
    <fieldset className="fe-group">
      <legend>{spec.label}</legend>
      {Array.from({ length: spec.count }, (_, at) => (
        <SelectInput
          key={at}
          label={`${at + 1}つ目`}
          value={boundToText(current[at])}
          options={idsOf(draft, spec.of)}
          empty="選ぶ"
          onChange={(value) => {
            const next = setItem({ list: current, index: at, item: value, count: spec.count });
            onChange(
              withField(
                object,
                spec.key,
                next.filter((item) => item !== ''),
              ),
            );
          }}
        />
      ))}
    </fieldset>
  );
}

export { DomainField, JsonField, PositionField, ReferenceField, ReferencesField };
