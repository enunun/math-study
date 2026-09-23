import { useRef, useState } from 'react';
import type { ReactElement } from 'react';

import { copyText, downloadText } from '@/figure-editor/download';
import { parseDraft } from '@/figure-editor/draft';
import type { SceneDraft, ViewKind } from '@/figure-editor/draft';
import type { JsonObject } from '@/figure-editor/json';
import { EDITOR_SAMPLES } from '@/figure-editor/samples';
import { OBJECT_TEMPLATE_GROUPS, SCENE_TEMPLATES } from '@/figure-editor/templates';
import type { ObjectTemplateGroup } from '@/figure-editor/templates';
import { fileName, standaloneDocument } from '@/figure-editor/tikz-document';

import { SelectInput } from './field-inputs';

const STEM = 'figure';

interface Props {
  kind: ViewKind;
  onLoad: (draft: SceneDraft) => void;
  onNew: () => void;
  onInsert: (objects: readonly JsonObject[]) => void;
  onMessage: (message: string) => void;
}

/** 見本(そのまま使える完成した図)と，空の図の選択．見本は，今の図を置き換える． */
function Samples({ onLoad, onNew }: Pick<Props, 'onLoad' | 'onNew'>): ReactElement {
  return (
    <div className="fe-buttons" role="group" aria-label="見本">
      <span>見本：</span>
      {EDITOR_SAMPLES.map((sample) => (
        <button
          key={sample.id}
          type="button"
          onClick={() => {
            onLoad(sample.scene);
          }}
        >
          {sample.label}
        </button>
      ))}
      <button type="button" onClick={onNew}>
        空の図
      </button>
    </div>
  );
}

/** 部品として組み合わせるテンプレート(正多角形・2次曲線・ベジエ曲面など)を，種類ごとに選んで，今の図に挿入する． */
function ObjectTemplatePicker({
  group,
  onInsert,
}: {
  group: ObjectTemplateGroup;
  onInsert: (objects: readonly JsonObject[]) => void;
}): ReactElement {
  const { templates } = group;
  const [choice, setChoice] = useState(templates[0]?.id ?? '');
  const current = templates.find((template) => template.id === choice) ?? templates[0];
  return (
    <span className="fe-template-picker">
      <SelectInput
        label={group.label}
        value={choice}
        options={templates.map((template) => [template.id, template.label])}
        onChange={setChoice}
      />
      <button
        type="button"
        onClick={() => {
          if (current !== undefined) {
            onInsert(current.objects);
          }
        }}
      >
        挿入
      </button>
    </span>
  );
}

/**
 * 部品のテンプレートの一覧．今の図の種類(平面・空間)で使えないものは出さない．平面と空間の両方を持つ
 * まとまりもあるので，図の種類が変わったら，選択欄を作り直す(`key`に種類を含める)．
 */
function ObjectTemplates({
  kind,
  onInsert,
}: {
  kind: ViewKind;
  onInsert: (objects: readonly JsonObject[]) => void;
}): ReactElement {
  const groups = OBJECT_TEMPLATE_GROUPS.map((group) => ({
    label: group.label,
    templates: group.templates.filter((template) => template.kind === kind),
  })).filter((group) => group.templates.length > 0);
  return (
    <div className="fe-buttons" role="group" aria-label="部品のテンプレート">
      <span>テンプレート(部品)：</span>
      {groups.map((group) => (
        <ObjectTemplatePicker key={`${kind}-${group.label}`} group={group} onInsert={onInsert} />
      ))}
    </div>
  );
}

/** 図のテンプレート(座標軸だけの図など)．中身のない出発点で，選ぶと，今の図を置き換える． */
function SceneTemplates({ onLoad }: Pick<Props, 'onLoad'>): ReactElement {
  return (
    <div className="fe-buttons" role="group" aria-label="図のテンプレート">
      <span>テンプレート(図)：</span>
      {SCENE_TEMPLATES.map((template) => (
        <button
          key={template.id}
          type="button"
          onClick={() => {
            onLoad(template.scene);
          }}
        >
          {template.label}
        </button>
      ))}
    </div>
  );
}

/** 選んだファイルを読み，図にする．読めなければ，理由を知らせる． */
async function loadFile(file: File, { onLoad, onMessage }: Props): Promise<void> {
  const parsed = parseDraft(await file.text());
  if (parsed.ok) {
    onLoad(parsed.draft);
    onMessage(`${file.name}を読み込んだ．`);
  } else {
    onMessage(`${file.name}を読めない：${parsed.message}`);
  }
}

/** JSONのファイルを読み込むボタン． */
function ImportButton(props: Props): ReactElement {
  const input = useRef<HTMLInputElement>(null);
  return (
    <>
      <button
        type="button"
        onClick={() => {
          input.current?.click();
        }}
      >
        JSONを読み込む
      </button>
      <input
        ref={input}
        type="file"
        accept=".json,application/json"
        className="fe-hidden"
        aria-label="読み込むJSONのファイル"
        onChange={(event) => {
          const [file] = event.target.files ?? [];
          event.target.value = '';
          if (file !== undefined) {
            void loadFile(file, props);
          }
        }}
      />
    </>
  );
}

interface ExportProps {
  json: string;
  /** 図を描けているときの，TikZ．描けていなければ，空の文字列． */
  tikz: string;
  onMessage: (message: string) => void;
}

/** コピーの結果を知らせる． */
async function copyTikz(tikz: string, onMessage: (message: string) => void): Promise<void> {
  const ok = await copyText(tikz);
  onMessage(ok ? 'TikZをコピーした．' : 'コピーできなかった．');
}

/** JSONとTikZの書き出しと，TikZのコピー． */
function ExportButtons({ json, tikz, onMessage }: ExportProps): ReactElement {
  const empty = tikz === '';
  return (
    <>
      <button
        type="button"
        onClick={() => {
          downloadText(fileName(STEM, 'json'), json, 'application/json');
        }}
      >
        JSONを書き出す
      </button>
      <button
        type="button"
        disabled={empty}
        onClick={() => {
          downloadText(fileName(STEM, 'tikz'), tikz, 'text/plain');
        }}
      >
        TikZを書き出す(.tikz)
      </button>
      <button
        type="button"
        disabled={empty}
        onClick={() => {
          downloadText(fileName(STEM, 'tex'), standaloneDocument(tikz), 'text/x-tex');
        }}
      >
        単体の文書を書き出す(.tex)
      </button>
      <button
        type="button"
        disabled={empty}
        onClick={() => {
          void copyTikz(tikz, onMessage);
        }}
      >
        TikZをコピー
      </button>
    </>
  );
}

/** 見本，JSONの読み込みと書き出し，TikZの書き出し． */
function Toolbar({
  message,
  json,
  tikz,
  ...actions
}: Props & ExportProps & { message: string }): ReactElement {
  return (
    <div className="fe-toolbar">
      <Samples onLoad={actions.onLoad} onNew={actions.onNew} />
      <ObjectTemplates kind={actions.kind} onInsert={actions.onInsert} />
      <SceneTemplates onLoad={actions.onLoad} />
      <div className="fe-buttons" role="group" aria-label="ファイル">
        <ImportButton {...actions} />
        <ExportButtons json={json} tikz={tikz} onMessage={actions.onMessage} />
      </div>
      <p role="status" className="fe-message">
        {message}
      </p>
    </div>
  );
}

export { Toolbar };
