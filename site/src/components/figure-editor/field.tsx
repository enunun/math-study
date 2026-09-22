import type { ReactElement } from 'react';

import type { FieldSpec } from '@/figure-editor/fields';

import { CheckboxField, ListField, ScalarField, SelectField } from './field-basic';
import type { FieldProps } from './field-props';
import {
  DomainField,
  JsonField,
  PositionField,
  ReferenceField,
  ReferencesField,
} from './field-special';
import { StyleInput, ToggleStyleInput } from './style-input';

type BasicKind = 'text' | 'number' | 'bound' | 'select' | 'checkbox' | 'list';

type LinkKind = 'reference' | 'references' | 'style' | 'toggleStyle';

const BASIC_KINDS = new Set<string>(['text', 'number', 'bound', 'select', 'checkbox', 'list']);

function isBasic(spec: FieldSpec): spec is Extract<FieldSpec, { kind: BasicKind }> {
  return BASIC_KINDS.has(spec.kind);
}

/** 文字列，数，選択，真偽，並びの入力欄． */
function BasicField(props: FieldProps<BasicKind>): ReactElement {
  const { spec } = props;
  if (spec.kind === 'select') {
    return <SelectField {...props} spec={spec} />;
  }
  if (spec.kind === 'checkbox') {
    return <CheckboxField {...props} spec={spec} />;
  }
  if (spec.kind === 'list') {
    return <ListField {...props} spec={spec} />;
  }
  return <ScalarField {...props} spec={spec} />;
}

/** 別のオブジェクトを選ぶ入力欄と，スタイル． */
function LinkField(props: FieldProps<LinkKind>): ReactElement {
  const { spec } = props;
  if (spec.kind === 'reference') {
    return <ReferenceField {...props} spec={spec} />;
  }
  if (spec.kind === 'references') {
    return <ReferencesField {...props} spec={spec} />;
  }
  if (spec.kind === 'toggleStyle') {
    return (
      <ToggleStyleInput
        object={props.object}
        kind={props.kind}
        field={spec.key}
        label={spec.label}
        onChange={props.onChange}
      />
    );
  }
  return <StyleInput object={props.object} kind={props.kind} onChange={props.onChange} />;
}

/** 範囲，座標，JSONの入力欄．残りは，別のオブジェクトを選ぶ入力欄と，スタイルである． */
function ComplexField(props: FieldProps<Exclude<FieldSpec['kind'], BasicKind>>): ReactElement {
  const { spec } = props;
  if (spec.kind === 'domain2') {
    return <DomainField {...props} spec={spec} />;
  }
  if (spec.kind === 'position') {
    return <PositionField {...props} spec={spec} />;
  }
  if (spec.kind === 'json') {
    return <JsonField {...props} spec={spec} />;
  }
  return <LinkField {...props} spec={spec} />;
}

/** 項目の入力欄．仕様の種類ごとに，入力欄を選ぶ． */
function Field(props: FieldProps): ReactElement {
  const { spec } = props;
  return isBasic(spec) ? (
    <BasicField {...props} spec={spec} />
  ) : (
    <ComplexField {...props} spec={spec} />
  );
}

export { Field };
