import type { ReactElement } from 'react';

import type { SceneDraft, ViewKind } from '@/figure-editor/draft';
import { fieldsFor } from '@/figure-editor/fields';
import { stringOf } from '@/figure-editor/json';
import type { JsonObject } from '@/figure-editor/json';
import { objectAnchor } from '@/figure-reference/schema-text';

import { Field } from './field';
import { typeLabel } from './object-list';

interface Props {
  object: JsonObject;
  draft: SceneDraft;
  kind: ViewKind;
  onChange: (object: JsonObject) => void;
}

/** 種類の説明(図のシーンのリファレンスの節)のURL． */
function referenceHref(type: string): string {
  const base = import.meta.env.BASE_URL.replace(/\/$/u, '');
  return `${base}/tools/graphics/figure-reference/#${objectAnchor(type)}`;
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
      <a className="fe-reference-link" href={referenceHref(stringOf(object, 'type'))}>
        この種類の説明
      </a>
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
