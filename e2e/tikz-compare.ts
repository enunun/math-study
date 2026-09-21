import { mkdir, readFile } from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

import { server } from './serve.ts';
import { captureSvgs } from './tikz/capture.ts';
import { compile } from './tikz/latex.ts';
import type { Rendered } from './tikz/latex.ts';
import { score } from './tikz/masks.ts';
import { FIGURE_PAGES, KINDS, OUT_DIR, SIZE_TOLERANCE } from './tikz/settings.ts';

/**
 * 図のTikZの出力を，LuaLaTeXでコンパイルし，PDFを画像にして，サイトのSVGの図と重ねて比べる．
 * 線や矢じりだけ(paths)と，ラベルだけ(labels)に分けて比べる．線は，位置が2px(約0.5mm)ずれると失敗し，
 * ラベルは，MathJaxとLaTeXの書体の細かい違いを許す．
 * 結果は，tikz-out/に，図と種類ごとの，(SVG，PDF，差)を並べた画像として書く．
 * 使い方：mise run tikz [図の名前…]
 */

/** Wasmの窓口のうち，ここで使う部分．生成されたファイルは，実行時に読み込む． */
interface FigureWasm {
  initSync: (options: { module: Buffer }) => void;
  renderScene: (
    json: string,
  ) =>
    | { status: 'ok'; figure: { bounds: Rendered['bounds'] }; tikz: string }
    | { status: 'error'; message: string };
}

/** MDXに書かれた，`<Figure src="…" />`の名前を，書かれた順に返す． */
async function figureNames(mdx: string): Promise<string[]> {
  const source = await readFile(mdx, 'utf8');
  return [...source.matchAll(/<Figure src="(?<name>[a-z0-9-]+)"/gu)].flatMap((found) =>
    found.groups?.name === undefined ? [] : [found.groups.name],
  );
}

/** 読み込んだものが，使うWasmの窓口を持っているか． */
function isFigureWasm(value: unknown): value is FigureWasm {
  return (
    typeof value === 'object' &&
    value !== null &&
    'initSync' in value &&
    typeof value.initSync === 'function' &&
    'renderScene' in value &&
    typeof value.renderScene === 'function'
  );
}

async function loadWasm(): Promise<FigureWasm> {
  const wasm: unknown = await import(pathToFileURL(path.resolve('site/src/wasm/figure.js')).href);
  if (!isFigureWasm(wasm)) {
    throw new Error(
      'site/src/wasm/figure.jsに，initSyncとrenderSceneがない．mise run wasmを実行する',
    );
  }
  wasm.initSync({ module: await readFile('site/src/wasm/figure_bg.wasm') });
  return wasm;
}

/** シーンのJSONを，Wasmで描画して，TikZの文字列と描く範囲を得る． */
async function render(wasm: FigureWasm, name: string): Promise<Rendered> {
  const outcome = wasm.renderScene(await readFile(`site/src/figures/${name}.json`, 'utf8'));
  if (outcome.status !== 'ok') {
    throw new Error(`${name}を描画できない：${outcome.message}`);
  }
  return { name, bounds: outcome.figure.bounds, tikz: outcome.tikz };
}

async function main(): Promise<void> {
  const wasm = await loadWasm();
  await mkdir(OUT_DIR, { recursive: true });
  const wanted = new Set(process.argv.slice(2));
  const pages = new Map<string, string[]>(
    await Promise.all(
      FIGURE_PAGES.map(async ([page, mdx]) => [page, await figureNames(mdx)] as const),
    ),
  );
  const names = [...pages.values()].flat().filter((name) => wanted.size === 0 || wanted.has(name));
  const rendered = await Promise.all(names.map((name) => render(wasm, name)));
  const [svgs, pdfs] = await Promise.all([
    captureSvgs(pages),
    Promise.all(
      rendered.flatMap((item) =>
        KINDS.map(async (kind) => [`${item.name}-${kind}`, await compile(item, kind)] as const),
      ),
    ),
  ]);
  const pdfFiles = new Map(pdfs);
  const lines = await Promise.all(
    names.map(async (name) => {
      const scores = await Promise.all(
        KINDS.map((kind) =>
          score({
            name,
            kind,
            svgFile: svgs.get(`${name}-${kind}`) ?? '',
            pdfFile: pdfFiles.get(`${name}-${kind}`) ?? '',
          }),
        ),
      );
      const detail = scores
        .map((s) => {
          const note = s.sizeGap > SIZE_TOLERANCE ? '(大きさ違い)' : '';
          return `${s.kind} ${(s.mismatch * 100).toFixed(1)}%${note}`;
        })
        .join('，');
      const ok = scores.every((s) => s.ok);
      return { ok, text: `${ok ? 'OK' : 'NG'}  ${name}：${detail}` };
    }),
  );
  for (const line of lines) {
    process.stdout.write(`${line.text}\n`);
  }
  process.exitCode = lines.every((line) => line.ok) ? 0 : 1;
}

try {
  await main();
} finally {
  server.close();
}
