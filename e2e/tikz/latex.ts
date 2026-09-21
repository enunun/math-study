import { execFile } from 'node:child_process';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { promisify } from 'node:util';

import { DPI, OUT_DIR } from './settings.ts';
import type { Kind } from './settings.ts';

// promisifyは，execFileのコールバックの型を，値を返す関数として扱う．ここでは，値は使わない．
// eslint-disable-next-line typescript/strict-void-return
const execFileAsync = promisify(execFile);

/** 外部のコマンドを実行し，終わりを待つ．失敗したら，例外になる． */
async function run(command: string, args: string[]): Promise<void> {
  await execFileAsync(command, args);
}

/** 描画した図．`tikz`は，Wasmが出したTikZで，`bounds`は，描く範囲(cm)である． */
interface Rendered {
  name: string;
  bounds: { min: [number, number]; max: [number, number] };
  tikz: string;
}

/** ノードの文字は，塗りとして描かれるので，線と塗りを透明にすると消える．文字を残すため，線と塗りだけを消す． */
const HIDE_PATHS = ', draw opacity=0, fill opacity=0]';

/** ラベルだけを見る種類では，線と塗りの指定の最後に，透明にする指定を足す(指定の順で，あとが優先される)． */
function hidePaths(tikz: string): string {
  return tikz.replaceAll(/(?<head>\\(?:draw|fill)\[[^\]]*)\]/gu, `$<head>${HIDE_PATHS}`);
}

/** 図の描く範囲を，TikZの絵の大きさに指定した，コンパイル用の文書．種類ごとに，隠すものが違う． */
function document(rendered: Rendered, kind: Kind): string {
  const { min, max } = rendered.bounds;
  const box = String.raw`\useasboundingbox (${min[0]},${min[1]}) rectangle (${max[0]},${max[1]});`;
  const hide =
    kind === 'paths' ? String.raw`\tikzset{every node/.append style={text opacity=0}}` : '';
  const body = kind === 'labels' ? hidePaths(rendered.tikz) : rendered.tikz;
  const start = String.raw`\begin{tikzpicture}`;
  const picture = body.replace(`${start}\n`, () => `${start}\n${box}\n${hide}\n`);
  return [
    String.raw`\documentclass[tikz,border=0pt]{standalone}`,
    String.raw`\usepackage[haranoaji]{luatexja-preset}`,
    String.raw`\usetikzlibrary{arrows.meta}`,
    String.raw`\begin{document}`,
    picture,
    String.raw`\end{document}`,
    '',
  ].join('\n');
}

/** TikZをPDFにして，画像にする．画像のファイル名を返す． */
async function compile(rendered: Rendered, kind: Kind): Promise<string> {
  const base = path.join(OUT_DIR, `${rendered.name}-${kind}`);
  await writeFile(`${base}.tex`, document(rendered, kind));
  try {
    await run('lualatex', [
      '-interaction=nonstopmode',
      '-halt-on-error',
      `-output-directory=${OUT_DIR}`,
      `${base}.tex`,
    ]);
  } catch (error) {
    const log = await readFile(`${base}.log`, 'utf8').catch(() => '');
    throw new Error(`${rendered.name}のコンパイルに失敗した\n${log.slice(-1500)}`, {
      cause: error,
    });
  }
  await run('pdftoppm', ['-r', DPI, '-png', '-singlefile', `${base}.pdf`, `${base}-pdf`]);
  return `${base}-pdf.png`;
}

export { compile };
export type { Rendered };
