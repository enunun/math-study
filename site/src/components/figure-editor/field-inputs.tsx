import type { ReactElement } from 'react';

import type { Json } from '@/figure-editor/json';
import type { ItemKind } from '@/figure-editor/values';
import {
  boundFromText,
  boundToText,
  isPointExpression,
  parseItem,
  setItem,
} from '@/figure-editor/values';

/** ラベルつきの，1行の入力欄． */
function TextInput({
  label,
  value,
  onChange,
  invalid = false,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  invalid?: boolean;
}): ReactElement {
  return (
    <label className="fe-field">
      <span>{label}</span>
      <input
        type="text"
        value={value}
        autoComplete="off"
        autoCapitalize="off"
        spellCheck={false}
        aria-invalid={invalid}
        onChange={(event) => {
          onChange(event.target.value);
        }}
      />
    </label>
  );
}

interface SelectProps {
  label: string;
  value: string;
  options: readonly (readonly [string, string])[];
  /** 空の選択肢の名前．空の文字列なら，空の選択肢を出さない． */
  empty?: string;
  onChange: (value: string) => void;
}

function SelectInput({ label, value, options, empty = '', onChange }: SelectProps): ReactElement {
  const known = options.some(([optionValue]) => optionValue === value);
  return (
    <label className="fe-field">
      <span>{label}</span>
      <select
        value={value}
        onChange={(event) => {
          onChange(event.target.value);
        }}
      >
        {empty !== '' && <option value="">{empty}</option>}
        {!known && value !== '' && <option value={value}>{value}</option>}
        {options.map(([optionValue, optionLabel]) => (
          <option key={optionValue} value={optionValue}>
            {optionLabel}
          </option>
        ))}
      </select>
    </label>
  );
}

function CheckboxInput({
  label,
  checked,
  onChange,
  disabled = false,
}: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
}): ReactElement {
  return (
    <label className="fe-check">
      <input
        type="checkbox"
        checked={checked}
        disabled={disabled}
        onChange={(event) => {
          onChange(event.target.checked);
        }}
      />
      <span>{label}</span>
    </label>
  );
}

interface ListProps {
  label: string;
  /** 入力欄ごとの名前． */
  names: readonly string[];
  value: readonly Json[];
  item: ItemKind;
  onChange: (value: Json[]) => void;
}

/** 数か式を，決まった数だけ並べる入力欄． */
function ListInput({ label, names, value, item, onChange }: ListProps): ReactElement {
  return (
    <fieldset className="fe-group">
      <legend>{label}</legend>
      {names.map((name, index) => (
        <TextInput
          key={name}
          label={name}
          value={boundToText(value[index])}
          onChange={(text) => {
            onChange(
              setItem({ list: value, index, item: parseItem(item, text), count: names.length }),
            );
          }}
        />
      ))}
    </fieldset>
  );
}

interface PositionProps {
  label: string;
  value: Json | undefined;
  names: readonly string[];
  onChange: (value: Json) => void;
}

/** 座標(数か式の並び)か，点の式(`A + B`など)の入力欄． */
function PositionInput({ label, value, names, onChange }: PositionProps): ReactElement {
  const expression = isPointExpression(value);
  const list = Array.isArray(value) ? value : [];
  return (
    <fieldset className="fe-group">
      <legend>{label}</legend>
      <SelectInput
        label="書き方"
        value={expression ? 'expression' : 'coordinates'}
        options={[
          ['coordinates', '座標'],
          ['expression', '点の式'],
        ]}
        onChange={(mode) => {
          onChange(mode === 'expression' ? '' : names.map(() => 0));
        }}
      />
      {expression ? (
        <TextInput label="点の式(A + Bなど)" value={value} onChange={onChange} />
      ) : (
        names.map((name, index) => (
          <TextInput
            key={name}
            label={name}
            value={boundToText(list[index])}
            onChange={(text) => {
              onChange(setItem({ list, index, item: boundFromText(text), count: names.length }));
            }}
          />
        ))
      )}
    </fieldset>
  );
}

export { CheckboxInput, ListInput, PositionInput, SelectInput, TextInput };
