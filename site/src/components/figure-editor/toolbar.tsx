import { useRef } from 'react';
import type { ReactElement } from 'react';

import { parseDraft } from '@/figure-editor/draft';
import type { SceneDraft, ViewKind } from '@/figure-editor/draft';
import type { JsonObject } from '@/figure-editor/json';
import { SAMPLE_TIERS } from '@/figure-editor/samples';
import { OBJECT_TEMPLATE_CATEGORIES, SCENE_TEMPLATES } from '@/figure-editor/templates';
import type { Figure } from '@/wasm/figure';

import { ExportButtons, STEM } from './export-buttons';
import type { ExportProps } from './export-buttons';
import { GroupedPicker, TieredPicker } from './grouped-picker';
import { HistoryControls } from './history-controls';
import { ImageExportControls } from './image-export-controls';

interface Props {
  kind: ViewKind;
  onLoad: (draft: SceneDraft) => void;
  onInsert: (objects: readonly JsonObject[]) => void;
  onMessage: (message: string) => void;
}

/**
 * 見本(そのまま使える完成した図)の選択．平面・空間，種別，見本の3段で選ぶ．見本は，今の図を置き換える．
 */
function Samples({ onLoad }: Pick<Props, 'onLoad'>): ReactElement {
  const tiers = SAMPLE_TIERS.map((tier) => ({
    label: tier.label,
    categories: tier.groups.map((group) => ({
      label: group.label,
      groups: [{ label: group.label, items: group.samples }],
    })),
  }));
  return (
    <TieredPicker
      title="見本"
      name="見本"
      tiers={tiers}
      action="読み込む"
      onPick={(sample) => {
        onLoad(sample.scene);
      }}
    />
  );
}

/**
 * 部品のテンプレート(正多角形・2次曲線・Bézier曲面など)を，分類とまとまりから選んで，今の図に挿入する．
 * 今の図の種類(平面・空間)で使えないものは出さない．平面と空間の両方を持つまとまりもあるので，図の種類が
 * 変わったら，選択欄を作り直す(`key`に種類を含める)．
 */
function ObjectTemplates({
  kind,
  onInsert,
}: {
  kind: ViewKind;
  onInsert: (objects: readonly JsonObject[]) => void;
}): ReactElement {
  const categories = OBJECT_TEMPLATE_CATEGORIES.map((category) => ({
    label: category.label,
    groups: category.groups
      .map((group) => ({
        label: group.label,
        items: group.templates.filter((template) => template.kind === kind),
      }))
      .filter((group) => group.items.length > 0),
  })).filter((category) => category.groups.length > 0);
  return (
    <GroupedPicker
      key={kind}
      title="テンプレート(部品)"
      name="部品"
      categories={categories}
      action="挿入"
      onPick={(template) => {
        onInsert(template.objects);
      }}
    />
  );
}

/** 図のテンプレート(空の図，座標軸だけの図など)．中身のない出発点で，選ぶと，今の図を置き換える． */
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

/** 元に戻す・やり直す，見本とテンプレート，JSONの読み込みと書き出し，TikZと画像の書き出し． */
function Toolbar({
  message,
  json,
  tikz,
  figure,
  history,
  ...actions
}: Props &
  ExportProps & {
    message: string;
    /** 図を描けているときの，中間表現．描けていなければ，`undefined`． */
    figure: Figure | undefined;
    /** 元に戻す・やり直す． */
    history: Parameters<typeof HistoryControls>[0];
  }): ReactElement {
  return (
    <div className="fe-toolbar">
      <HistoryControls {...history} />
      <Samples onLoad={actions.onLoad} />
      <ObjectTemplates kind={actions.kind} onInsert={actions.onInsert} />
      <SceneTemplates onLoad={actions.onLoad} />
      <div className="fe-buttons" role="group" aria-label="ファイル">
        <ImportButton {...actions} />
        <ExportButtons json={json} tikz={tikz} onMessage={actions.onMessage} />
      </div>
      <ImageExportControls figure={figure} stem={STEM} onMessage={actions.onMessage} />
      <p role="status" className="fe-message">
        {message}
      </p>
    </div>
  );
}

export { Toolbar };
