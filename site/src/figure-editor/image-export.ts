import { BACKGROUND_COLOR, standaloneSvg } from '@/figure/export-svg';
import type { SvgFile } from '@/figure/export-svg';
import { labelToMath } from '@/figure/label';
import { labelSvg } from '@/figure/label-svg';
import type { Figure } from '@/wasm/figure';

import { IMAGE_FORMATS, isTransparent, pixelSize } from './image-formats';
import type { FormatSpec, ImageOptions } from './image-formats';

/**
 * 図を，画像のファイルにする．ラベルを組むMathJaxのSVG出力が大きいため，書き出すときに初めて読み込む
 * (`image-export-controls.tsx`から，動的に`import`する)．
 */

/** JPEGとWebPの品質． */
const QUALITY = 0.92;

/** 図のラベルを，すべて，MathJaxのSVGで組み，SVGのファイルにする． */
async function figureSvg(figure: Figure, transparent: boolean): Promise<SvgFile> {
  const labels = await Promise.all(
    figure.items.flatMap((item) =>
      item.type === 'label' ? [labelSvg(labelToMath(item.tex))] : [],
    ),
  );
  return standaloneSvg(figure, labels, { transparent });
}

/** SVGの文字列を，画像として読み込む． */
async function loadImage(svg: string): Promise<HTMLImageElement> {
  const url = URL.createObjectURL(new Blob([svg], { type: IMAGE_FORMATS.svg.mime }));
  const image = new Image();
  image.src = url;
  try {
    await image.decode();
    return image;
  } finally {
    URL.revokeObjectURL(url);
  }
}

/**
 * SVGを，解像度に合わせた大きさのcanvasに描く．透過しないときは，先にcanvasを白く塗る．SVGの背景の長方形だけでは，
 * 画素の数に丸めた端が，半透明に残る．
 */
async function drawOnCanvas(
  file: SvgFile,
  resolution: number,
  transparent: boolean,
): Promise<OffscreenCanvas> {
  const image = await loadImage(file.svg);
  const canvas = new OffscreenCanvas(
    pixelSize(file.width, resolution),
    pixelSize(file.height, resolution),
  );
  const context = canvas.getContext('2d');
  if (context === null) {
    throw new Error('canvasを使えない');
  }
  if (!transparent) {
    context.fillStyle = BACKGROUND_COLOR;
    context.fillRect(0, 0, canvas.width, canvas.height);
  }
  context.drawImage(image, 0, 0, canvas.width, canvas.height);
  return canvas;
}

/** SVGを，canvasで画素にして，指定の形式の画像にする． */
async function rasterize(file: SvgFile, spec: FormatSpec, options: ImageOptions): Promise<Blob> {
  const canvas = await drawOnCanvas(file, options.resolution, isTransparent(options));
  const blob = await canvas.convertToBlob({ type: spec.mime, quality: QUALITY });
  // 対応していない形式を指定すると，ブラウザは，黙ってPNGにする．
  if (blob.type !== spec.mime) {
    throw new Error(`このブラウザは，${spec.label}を書き出せない`);
  }
  return blob;
}

/** 図を，指定した形式の画像にする．ブラウザだけで動く． */
async function exportImage(figure: Figure, options: ImageOptions): Promise<Blob> {
  const spec = IMAGE_FORMATS[options.format];
  const file = await figureSvg(figure, isTransparent(options));
  if (!spec.raster) {
    return new Blob([file.svg], { type: `${spec.mime};charset=utf-8` });
  }
  return rasterize(file, spec, options);
}

export { exportImage, figureSvg };
