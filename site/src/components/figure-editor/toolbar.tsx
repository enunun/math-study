import { useRef } from 'react';
import type { ReactElement } from 'react';

import { copyText, downloadText } from '@/figure-editor/download';
import { parseDraft } from '@/figure-editor/draft';
import type { SceneDraft } from '@/figure-editor/draft';
import { EDITOR_SAMPLES } from '@/figure-editor/samples';
import { fileName, standaloneDocument } from '@/figure-editor/tikz-document';

const STEM = 'figure';

interface Props {
  onLoad: (draft: SceneDraft) => void;
  onNew: () => void;
  onMessage: (message: string) => void;
}

/** 見本と，空の図の選択． */
function Samples({ onLoad, onNew }: Pick<Props, 'onLoad' | 'onNew'>): ReactElement {
  return (
    <div className="fe-buttons" role="group" aria-label="見本">
      <span>見本：</span>
      {EDITOR_SAMPLES.map((sample) => (
        <button
          key={sample.id}
          type="button"
          onClick={() => {
            const parsed = parseDraft(sample.json);
            if (parsed.ok) {
              onLoad(parsed.draft);
            }
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
