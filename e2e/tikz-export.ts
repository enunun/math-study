import { readdir } from 'node:fs/promises';
import { parseArgs } from 'node:util';

import { exportFigure } from './tikz/latex.ts';
import { loadWasm, render, sceneSource } from './tikz/scenes.ts';

/**
 * 図のTikZの出力を，ファイルとPDFに書き出す(サイトとの比較はしない)．
 * 名前(`site/src/figures/`のJSON)かJSONのパスを渡すと，その図だけを書き出す．省くと，すべての図を書き出す．
 * 使い方：mise run tikz:export [--png] [--dpi 200] [--out tikz-out/export] [図の名前かJSONのパス…]
 */
const USAGE = `使い方：mise run tikz:export -- [オプション] [図の名前かJSONのパス…]
  --png          PDFを画像(PNG)にも書き出す
  --dpi <数>     画像の解像度(既定は200)
  --out <場所>   書き出し先(既定はtikz-out/export)
図の名前を省くと，site/src/figures/のすべての図を書き出す．
書き出すもの：名前.tikz(出力そのもの)，名前.tex(standaloneの文書)，名前.pdf(と，--pngなら名前.png)
`;
const FIGURES_DIR = 'site/src/figures';
const DEFAULT_DPI = '200';

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: {
    png: { type: 'boolean', default: false },
    dpi: { type: 'string', default: DEFAULT_DPI },
    out: { type: 'string', default: 'tikz-out/export' },
    help: { type: 'boolean', default: false },
  },
});

/** 引数の図を，描画の入力にする．引数がなければ，`site/src/figures/`のすべての図である． */
async function targets(): Promise<string[]> {
  if (positionals.length > 0) {
    return positionals;
  }
  const files = await readdir(FIGURES_DIR);
  return files
    .filter((file) => file.endsWith('.json'))
    .map((file) => file.slice(0, -'.json'.length))
    .toSorted();
}

async function main(): Promise<void> {
  const wasm = await loadWasm();
  const dpi = Number(values.dpi);
  if (!Number.isFinite(dpi) || dpi <= 0) {
    throw new Error(`--dpiは，正の数で書く：${values.dpi}`);
  }
  const names = await targets();
  const rendered = await Promise.all(names.map((name) => render(wasm, sceneSource(name))));
  const written = await Promise.all(
    rendered.map((figure) => exportFigure(figure, { directory: values.out, png: values.png, dpi })),
  );
  for (const files of written) {
    process.stdout.write(`${files.join('  ')}\n`);
  }
}

if (values.help) {
  process.stdout.write(USAGE);
} else {
  await main();
}
