import type { ReactElement } from 'react';

import { copyText, downloadText } from '@/figure-editor/download';
import { fileName, standaloneDocument } from '@/figure-editor/tikz-document';

/** 書き出すファイルの名前の元． */
const STEM = 'figure';

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

export { ExportButtons, STEM };
export type { ExportProps };
