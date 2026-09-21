/** 図の比べ方の設定．TikZのPDFと，サイトのSVGを，同じ倍率の画像にして比べる． */

/** 画像の倍率．細い線(0.3pt)も，1画素以上になるよう，3倍の細かさで撮る． */
const SCALE = 3;
/** SVGの1cmは，画面で37.795px(96dpi)である．PDFも，倍率をかけた細かさで画像にする． */
const DPI = String(96 * SCALE);
/** 描いた画素とみなす，明るさ(RGBの和)の上限． */
const INK_SUM = 720;
/** 大きさの違いの許容(画素)． */
const SIZE_TOLERANCE = 2 * SCALE;

type Kind = 'full' | 'paths' | 'labels';
const KINDS: readonly Kind[] = ['full', 'paths', 'labels'];

/**
 * 種類ごとの，許すずれの画素数(表示の1px単位)と，食い違う割合の上限．
 * 線は，位置が2px(約0.5mm)ずれると失敗する．ラベルは，MathJaxとLaTeXの書体の細かい違いを許す．
 */
const LIMITS: Record<Kind, { radius: number; max: number }> = {
  full: { radius: 2, max: 0.05 },
  paths: { radius: 2, max: 0.02 },
  labels: { radius: 4, max: 0.15 },
};

/** 図の名前を書いたMDXと，その公開ページ． */
const FIGURE_PAGES = [
  ['dev/figure-first/', 'site/src/content/docs/dev/figure-first.mdx'],
  ['dev/figure-space/', 'site/src/content/docs/dev/figure-space.mdx'],
] as const;

const OUT_DIR = 'tikz-out';

export { DPI, FIGURE_PAGES, INK_SUM, KINDS, LIMITS, OUT_DIR, SCALE, SIZE_TOLERANCE };
export type { Kind };
