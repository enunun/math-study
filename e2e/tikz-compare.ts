import { mkdir } from 'node:fs/promises';

import { server } from './serve.ts';
import { captureSvgs } from './tikz/capture.ts';
import { compile } from './tikz/latex.ts';
import { score } from './tikz/masks.ts';
import { figureNames, loadWasm, render, sceneSource } from './tikz/scenes.ts';
import { FIGURE_PAGES, KINDS, OUT_DIR, SIZE_TOLERANCE } from './tikz/settings.ts';

/**
 * 図のTikZの出力を，LuaLaTeXでコンパイルし，PDFを画像にして，サイトのSVGの図と重ねて比べる．
 * 線や矢じりだけ(paths)と，ラベルだけ(labels)に分けて比べる．線は，位置が2px(約0.5mm)ずれると失敗し，
 * ラベルは，MathJaxとLaTeXの書体の細かい違いを許す．
 * 結果は，tikz-out/に，図と種類ごとの，(SVG，PDF，差)を並べた画像として書く．
 * 使い方：mise run tikz [図の名前…]
 */

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
  const rendered = await Promise.all(names.map((name) => render(wasm, sceneSource(name))));
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
