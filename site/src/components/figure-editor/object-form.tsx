import type { ReactElement } from 'react';

import type { SceneDraft, ViewKind } from '@/figure-editor/draft';
import { fieldsFor } from '@/figure-editor/fields';
import { stringOf } from '@/figure-editor/json';
import type { JsonObject } from '@/figure-editor/json';

import { Field } from './field';
import { typeLabel } from './object-list';

interface Props {
  object: JsonObject;
  draft: SceneDraft;
  kind: ViewKind;
  onChange: (object: JsonObject) => void;
}

/** 選んだオブジェクトの，項目の入力欄．識別子を変えても，ほかのオブジェクトの参照は書き換わらないので，参照先を選び直す． */
function ObjectForm({ object, draft, kind, onChange }: Props): ReactElement {
  const fields = fieldsFor(object, kind);
  return (
    <section
      className="fe-form"
      aria-label={`${typeLabel(object)}「${stringOf(object, 'id')}」の設定`}
    >
      <h3>
        {typeLabel(object)}「{stringOf(object, 'id')}」
      </h3>
      {fields.map((spec, index) => (
        <Field
          key={`${stringOf(object, 'id')}-${index}-${'key' in spec ? spec.key : spec.kind}`}
          spec={spec}
          object={object}
          draft={draft}
          kind={kind}
          onChange={onChange}
        />
      ))}
    </section>
  );
}

export { ObjectForm };
