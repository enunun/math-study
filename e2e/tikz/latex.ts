import { execFile } from 'node:child_process';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { promisify } from 'node:util';

import type { Rendered } from './scenes.ts';
import { DPI, OUT_DIR } from './settings.ts';
import type { Kind } from './settings.ts';

// promisifyは，execFileのコールバックの型を，値を返す関数として扱う．ここでは，値は使わない．
// eslint-disable-next-line typescript/strict-void-return
const execFileAsync = promisify(execFile);

/** 外部のコマンドを実行し，終わりを待つ．失敗したら，例外になる． */
async function run(command: string, args: string[]): Promise<void> {
  await execFileAsync(command, args);
}

/** ノードの文字は，塗りとして描かれるので，線と塗りを透明にすると消える．文字を残すため，線と塗りだけを消す． */
const HIDE_PATHS = ', draw opacity=0, fill opacity=0]';

/** ラベルだけを見る種類では，線と塗りの指定の最後に，透明にする指定を足す(指定の順で，あとが優先される)． */
function hidePaths(tikz: string): string {
  return tikz.replaceAll(/(?<head>\\(?:draw|fill)\[[^\]]*)\]/gu, `$<head>${HIDE_PATHS}`);
}

/** TikZの絵を，standaloneの文書に入れる．`border`は，絵のまわりの余白である． */
function wrap(picture: string, border: string): string {
  return [
    String.raw`\documentclass[tikz,border=${border}]{standalone}`,
    String.raw`\usepackage[haranoaji]{luatexja-preset}`,
    String.raw`\usetikzlibrary{arrows.meta}`,
    String.raw`\begin{document}`,
    picture,
    String.raw`\end{document}`,
    '',
  ].join('\n');
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
  return wrap(picture, '0pt');
}

/** `base`.texをLuaLaTeXでPDFにする．失敗したら，LaTeXのログの終わりを添えた例外にする． */
async function typeset(base: string, name: string): Promise<void> {
  try {
    await run('lualatex', [
      '-interaction=nonstopmode',
      '-halt-on-error',
      `-output-directory=${path.dirname(base)}`,
      `${base}.tex`,
    ]);
  } catch (error) {
    const log = await readFile(`${base}.log`, 'utf8').catch(() => '');
    throw new Error(`${name}のコンパイルに失敗した\n${log.slice(-1500)}`, { cause: error });
  }
}

/** TikZをPDFにして，画像にする．画像のファイル名を返す． */
async function compile(rendered: Rendered, kind: Kind): Promise<string> {
  const base = path.join(OUT_DIR, `${rendered.name}-${kind}`);
  await writeFile(`${base}.tex`, document(rendered, kind));
  await typeset(base, rendered.name);
  await run('pdftoppm', ['-r', DPI, '-png', '-singlefile', `${base}.pdf`, `${base}-pdf`]);
  return `${base}-pdf.png`;
}

/** 書き出しの設定．`png`が真なら，PDFを画像にも書き出す． */
interface ExportOptions {
  directory: string;
  png: boolean;
  dpi: number;
}

/**
 * 図のTikZを，書き出す．`名前.tikz`は，出力そのもの(文書に貼る)，`名前.tex`は，それをstandaloneの文書に入れたもの，
 * `名前.pdf`は，そのPDFで，図の中身に，2ptの余白を付けた大きさである．書き出したファイルの名前を返す．
 */
async function exportFigure(rendered: Rendered, options: ExportOptions): Promise<string[]> {
  const base = path.join(options.directory, rendered.name);
  await mkdir(options.directory, { recursive: true });
  await writeFile(`${base}.tikz`, rendered.tikz);
  await writeFile(`${base}.tex`, wrap(rendered.tikz, '2pt'));
  await typeset(base, rendered.name);
  // 成功したら，LaTeXの補助ファイルは，要らない(失敗したときのログは，残す)．
  await Promise.all([`${base}.aux`, `${base}.log`].map((file) => rm(file, { force: true })));
  const files = [`${base}.tikz`, `${base}.tex`, `${base}.pdf`];
  if (options.png) {
    await run('pdftoppm', ['-r', String(options.dpi), '-png', '-singlefile', `${base}.pdf`, base]);
    files.push(`${base}.png`);
  }
  return files;
}

export { compile, exportFigure };
export type { ExportOptions };
