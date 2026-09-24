/** 書き出せる画像の形式． */
type ImageFormat = 'svg' | 'png' | 'jpeg' | 'webp';

interface FormatSpec {
  label: string;
  extension: string;
  mime: string;
  /** 背景を透明にできるか． */
  transparency: boolean;
  /** 画素の画像か．画素の画像は，解像度を選ぶ． */
  raster: boolean;
}

const IMAGE_FORMATS: Record<ImageFormat, FormatSpec> = {
  svg: { label: 'SVG', extension: 'svg', mime: 'image/svg+xml', transparency: true, raster: false },
  png: { label: 'PNG', extension: 'png', mime: 'image/png', transparency: true, raster: true },
  jpeg: { label: 'JPEG', extension: 'jpg', mime: 'image/jpeg', transparency: false, raster: true },
  webp: { label: 'WebP', extension: 'webp', mime: 'image/webp', transparency: true, raster: true },
};

/** 選べる解像度(dpi)．既定は，印刷に向く300dpiである． */
const DRAFT_RESOLUTION = 150;
const DEFAULT_RESOLUTION = 300;
const FINE_RESOLUTION = 600;
const RESOLUTIONS = [DRAFT_RESOLUTION, DEFAULT_RESOLUTION, FINE_RESOLUTION] as const;
/** 1インチの長さ(cm)． */
const CM_PER_INCH = 2.54;
/** 画像の幅と高さの上限(画素)．ブラウザのcanvasが扱える大きさに収める． */
const MAX_PIXELS = 16_384;

interface ImageOptions {
  format: ImageFormat;
  /** 背景を透明にするか．透明にできない形式(JPEG)では，無視して白く塗る． */
  transparent: boolean;
  /** 画素の画像の解像度(dpi)． */
  resolution: number;
}

/** 実寸(cm)を，解像度(dpi)で画素の数にする．1画素以上，上限以下に収める． */
function pixelSize(centimeters: number, resolution: number): number {
  const pixels = Math.round((centimeters / CM_PER_INCH) * resolution);
  return Math.min(MAX_PIXELS, Math.max(1, pixels));
}

/** 形式が選ばれたとき，実際に背景を透明にするか．透明にできない形式では，選択にかかわらず塗る． */
function isTransparent({
  format,
  transparent,
}: Pick<ImageOptions, 'format' | 'transparent'>): boolean {
  return IMAGE_FORMATS[format].transparency && transparent;
}

function isImageFormat(value: string): value is ImageFormat {
  return Object.hasOwn(IMAGE_FORMATS, value);
}

export { DEFAULT_RESOLUTION, IMAGE_FORMATS, RESOLUTIONS, isImageFormat, isTransparent, pixelSize };
export type { FormatSpec, ImageFormat, ImageOptions };
