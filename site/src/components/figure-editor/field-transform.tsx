import { useState } from 'react';
import type { ReactElement } from 'react';

import type { SceneDraft, ViewKind } from '@/figure-editor/draft';
import { arrayOf, stringOf, withField } from '@/figure-editor/json';
import type { Json, JsonObject } from '@/figure-editor/json';
import { mapStep, transformTemplates } from '@/figure-editor/transform-templates';
import { readJsonField, stepsToText } from '@/figure-editor/values';

import { SelectInput } from './field-inputs';
import type { FieldProps } from './field-props';

/** 写像の手順の選択肢の値の頭．テンプレートの`id`と重ならないようにする． */
const MAP_PREFIX = 'map:';

/** 手順の選択肢．図の種類ごとのテンプレートと，図の中の写像である． */
function stepOptions(draft: SceneDraft, kind: ViewKind): (readonly [string, string])[] {
  const maps = draft.objects
    .filter((object) => stringOf(object, 'type') === 'map')
    .map((object) => stringOf(object, 'id'));
  return [
    ...transformTemplates(kind).map((template) => [template.id, template.label] as const),
    ...maps.map((id) => [`${MAP_PREFIX}${id}`, `写像${id}`] as const),
  ];
}

/** 選択肢の値から，手順を作る．見つからなければ`undefined`． */
function stepOf(choice: string, kind: ViewKind): JsonObject | undefined {
  if (choice.startsWith(MAP_PREFIX)) {
    return mapStep(choice.slice(MAP_PREFIX.length));
  }
  return transformTemplates(kind).find((template) => template.id === choice)?.step;
}

/** 手順のテンプレートを選んで，並びの最後に足す部分． */
function StepPicker({
  draft,
  kind,
  onAppend,
}: {
  draft: SceneDraft;
  kind: ViewKind;
  onAppend: (step: JsonObject) => void;
}): ReactElement {
  const options = stepOptions(draft, kind);
  const [choice, setChoice] = useState(options[0]?.[0] ?? '');
  return (
    <span className="fe-template-picker fe-step-picker">
      <SelectInput label="手順" value={choice} options={options} onChange={setChoice} />
      <button
        type="button"
        onClick={() => {
          const step = stepOf(choice, kind);
          if (step !== undefined) {
            onAppend(step);
          }
        }}
      >
        手順を足す
      </button>
    </span>
  );
}

/**
 * 変換(`transform`)の入力欄．手順の並びをJSONで書き，手順のテンプレート(回転，対称移動など)や，
 * 図の中の写像を選んで，並びの最後に足せる．足したあとの数は，JSONの欄で直す．
 */
function TransformField({
  spec,
  object,
  draft,
  kind,
  onChange,
}: FieldProps<'transform'>): ReactElement {
  const [text, setText] = useState(stepsToText(arrayOf(object, spec.key)));
  const [message, setMessage] = useState('');
  const commit = (value: Json | undefined): void => {
    onChange(withField(object, spec.key, value));
  };
  const edit = (next: string): void => {
    setText(next);
    const result = readJsonField(next, true);
    setMessage(result.kind === 'keep' ? result.message : '');
    if (result.kind === 'set') {
      commit(result.value);
    }
  };
  const append = (step: JsonObject): void => {
    const next = [...arrayOf(object, spec.key), step];
    setText(stepsToText(next));
    setMessage('');
    commit(next);
  };
  return (
    <fieldset className="fe-group fe-wide">
      <legend>{spec.label}</legend>
      <label className="fe-field fe-wide">
        <span>手順(JSON，上から順に施す)</span>
        <textarea
          rows={3}
          value={text}
          spellCheck={false}
          placeholder='[{"rotate": 90}]'
          aria-invalid={message !== ''}
          onChange={(event) => {
            edit(event.target.value);
          }}
        />
        {message !== '' && <span role="alert">{message}</span>}
      </label>
      <StepPicker draft={draft} kind={kind} onAppend={append} />
    </fieldset>
  );
}

export { TransformField };
