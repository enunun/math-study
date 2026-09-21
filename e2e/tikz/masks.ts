import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';

import { PNG } from 'pngjs';

import { INK_SUM, LIMITS, OUT_DIR, SCALE, SIZE_TOLERANCE } from './settings.ts';
import type { Kind } from './settings.ts';

/** 1つの図の，1つの種類の比べた結果． */
interface Score {
  kind: Kind;
  mismatch: number;
  sizeGap: number;
  ok: boolean;
}

/** 比べる2つの画像． */
interface Pair {
  name: string;
  kind: Kind;
  svgFile: string;
  pdfFile: string;
}

/** 描いた画素を1，そうでない画素を0にした表． */
interface Mask {
  width: number;
  height: number;
  cells: Uint8Array;
}

type Rgb = readonly [number, number, number];
const WHITE: Rgb = [255, 255, 255];
const BLACK: Rgb = [0, 0, 0];
const OPAQUE = 255;

async function readPng(file: string): Promise<PNG> {
  return PNG.sync.read(await readFile(file));
}

/** 画像の左上の，幅と高さの部分を，描いた画素の表にする．足りない所は，描いていないものとする． */
function inkMask(image: PNG, width: number, height: number): Mask {
  const cells = new Uint8Array(width * height);
  for (let row = 0; row < Math.min(height, image.height); row += 1) {
    for (let column = 0; column < Math.min(width, image.width); column += 1) {
      const at = (row * image.width + column) * 4;
      const sum =
        (image.data[at] ?? OPAQUE) +
        (image.data[at + 1] ?? OPAQUE) +
        (image.data[at + 2] ?? OPAQUE);
      cells[row * width + column] = sum < INK_SUM ? 1 : 0;
    }
  }
  return { width, height, cells };
}

/** 半径`radius`の正方形の中の，ずれの一覧． */
function squareOffsets(radius: number): [number, number][] {
  const range = Array.from({ length: 2 * radius + 1 }, (_, index) => index - radius);
  return range.flatMap((dy) => range.map((dx): [number, number] => [dy, dx]));
}

/** 描いた画素を，縦横に`radius`画素ずつ太らせた表． */
function dilate(mask: Mask, radius: number): Mask {
  const wide = new Uint8Array(mask.cells.length);
  const offsets = squareOffsets(radius);
  for (const [index, value] of mask.cells.entries()) {
    if (value === 1) {
      const [row, column] = [Math.floor(index / mask.width), index % mask.width];
      for (const [dy, dx] of offsets) {
        const [y, x] = [row + dy, column + dx];
        if (y >= 0 && y < mask.height && x >= 0 && x < mask.width) {
          wide[y * mask.width + x] = 1;
        }
      }
    }
  }
  return { ...mask, cells: wide };
}

/** 食い違う割合(相手の近くに描いた画素がない，こちらの画素の数を，こちらの描いた画素の数で割る)． */
function missing(mine: Mask, theirsWide: Mask): number {
  let ink = 0;
  let lost = 0;
  for (const [index, value] of mine.cells.entries()) {
    if (value === 1) {
      ink += 1;
      lost += theirsWide.cells[index] === 1 ? 0 : 1;
    }
  }
  return ink === 0 ? 0 : lost / ink;
}

/** 差の画素の色．SVGだけにある画素は赤，PDFだけにある画素は青，両方にある画素は灰色である． */
function differenceColour(onlySvg: boolean, onlyPdf: boolean, any: boolean): Rgb {
  if (onlySvg) {
    return [220, 30, 30];
  }
  if (onlyPdf) {
    return [30, 30, 220];
  }
  return any ? [150, 150, 150] : WHITE;
}

/** (SVG，PDF，差)を並べた画像を書き，食い違う割合の大きいほうを返す． */
async function writeSheet(
  name: string,
  sides: readonly [PNG, PNG],
  radius: number,
): Promise<number> {
  const width = Math.min(sides[0].width, sides[1].width);
  const height = Math.min(sides[0].height, sides[1].height);
  const [svg, pdf] = [inkMask(sides[0], width, height), inkMask(sides[1], width, height)];
  const [svgWide, pdfWide] = [dilate(svg, radius), dilate(pdf, radius)];
  const sheet = new PNG({ width: width * 3, height });
  const put = (pixel: number, rgb: Rgb): void => {
    const [red, green, blue] = rgb;
    sheet.data.set([red, green, blue, OPAQUE], pixel * 4);
  };
  for (let index = 0; index < width * height; index += 1) {
    const [row, x] = [Math.floor(index / width), index % width];
    const start = row * width * 3 + x;
    const [hasSvg, hasPdf] = [svg.cells[index] === 1, pdf.cells[index] === 1];
    put(start, hasSvg ? BLACK : WHITE);
    put(start + width, hasPdf ? BLACK : WHITE);
    const onlySvg = hasSvg && pdfWide.cells[index] !== 1;
    const onlyPdf = hasPdf && svgWide.cells[index] !== 1;
    put(start + 2 * width, differenceColour(onlySvg, onlyPdf, hasSvg || hasPdf));
  }
  await writeFile(path.join(OUT_DIR, `${name}.png`), PNG.sync.write(sheet));
  return Math.max(missing(svg, pdfWide), missing(pdf, svgWide));
}

/** 1つの図の，1つの種類を比べる． */
async function score(pair: Pair): Promise<Score> {
  const [svg, pdf] = await Promise.all([readPng(pair.svgFile), readPng(pair.pdfFile)]);
  const limit = LIMITS[pair.kind];
  const radius = Math.round(limit.radius * SCALE);
  const mismatch = await writeSheet(`${pair.name}-${pair.kind}`, [svg, pdf], radius);
  const sizeGap = Math.max(Math.abs(svg.width - pdf.width), Math.abs(svg.height - pdf.height));
  return {
    kind: pair.kind,
    mismatch,
    sizeGap,
    ok: sizeGap <= SIZE_TOLERANCE && mismatch <= limit.max,
  };
}

export { score };
export type { Pair, Score };
