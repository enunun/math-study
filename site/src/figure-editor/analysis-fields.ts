import { COLORMAPS, SHADINGS, VALUE_SCALES } from './field-options';
import type { FieldSpec } from './field-spec';

const PAIR = 2;

/** 画像の解像度(SVGの画素とTikZの升目)． */
const RASTER_RESOLUTIONS: readonly FieldSpec[] = [
  { kind: 'number', key: 'resolution', label: '画素の数(長い辺，省くと200)', optional: true },
  { kind: 'number', key: 'tikz_resolution', label: 'TikZの升目の数(省くと40)', optional: true },
];

/** 楕円曲線と，値を色で表す図の項目．`fields.ts`の一覧に加える． */
const ANALYSIS_FIELDS: Readonly<Record<string, readonly FieldSpec[]>> = {
  elliptic_curve: [
    { kind: 'bound', key: 'a', label: '係数a(y^2 = x^3 + ax + b)' },
    { kind: 'bound', key: 'b', label: '係数b' },
    { kind: 'bound', key: 'p', label: '点Pのx座標', optional: true },
    { kind: 'checkbox', key: 'p_lower', label: '点Pを下の枝にとる', initial: false },
    { kind: 'bound', key: 'q', label: '点Qのx座標(省くと2P)', optional: true },
    { kind: 'checkbox', key: 'q_lower', label: '点Qを下の枝にとる', initial: false },
    { kind: 'checkbox', key: 'construction', label: '和の作図を描く', initial: true },
    { kind: 'number', key: 'multiples', label: '倍数の点の数n(P，…，nP)', optional: true },
    { kind: 'checkbox', key: 'labels', label: '点の名前を置く', initial: true },
  ],
  heatmap: [
    { kind: 'list', key: 'vars', label: '変数(2つ)', item: 'text', count: PAIR, optional: true },
    { kind: 'text', key: 'expr', label: '値の式(iは虚数単位)' },
    {
      kind: 'json',
      key: 'domain',
      label: '画像の範囲(省くと見える範囲)',
      optional: true,
      hint: '[[-2, 2], [-1, 1]]',
    },
    {
      kind: 'list',
      key: 'range',
      label: '色の両端の値',
      item: 'bound',
      count: PAIR,
      optional: true,
    },
    { kind: 'select', key: 'colormap', label: '色', options: COLORMAPS, optional: true },
    { kind: 'select', key: 'scale', label: '目盛', options: VALUE_SCALES, optional: true },
    ...RASTER_RESOLUTIONS,
  ],
  domain_coloring: [
    { kind: 'text', key: 'var', label: '複素数の変数(省くとz)', optional: true },
    { kind: 'text', key: 'expr', label: '関数の式(iは虚数単位)' },
    {
      kind: 'json',
      key: 'domain',
      label: '画像の範囲(省くと見える範囲)',
      optional: true,
      hint: '[[-2, 2], [-2, 2]]',
    },
    { kind: 'select', key: 'shading', label: '明るさ', options: SHADINGS, optional: true },
    ...RASTER_RESOLUTIONS,
  ],
};

export { ANALYSIS_FIELDS };
