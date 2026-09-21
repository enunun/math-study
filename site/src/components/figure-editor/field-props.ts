import type { SceneDraft, ViewKind } from '@/figure-editor/draft';
import type { FieldSpec } from '@/figure-editor/fields';
import type { JsonObject } from '@/figure-editor/json';

/** 種類`K`の項目の仕様． */
type SpecOf<K extends FieldSpec['kind']> = Extract<FieldSpec, { kind: K }>;

/** 項目の入力欄が受け取る値．`onChange`には，項目を書き換えた，新しいオブジェクトを渡す． */
interface FieldProps<K extends FieldSpec['kind'] = FieldSpec['kind']> {
  spec: SpecOf<K>;
  object: JsonObject;
  draft: SceneDraft;
  kind: ViewKind;
  onChange: (object: JsonObject) => void;
}

export type { FieldProps, SpecOf };
