import {
  COLORMAPS,
  QUANTITIES,
  SERIES_MODES,
  SHADINGS,
  TRANSFORM_PARTS,
  VALUE_SCALES,
} from './field-options';
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
  fourier_series: [
    { kind: 'text', key: 'var', label: '変数(省くとx)', optional: true },
    { kind: 'text', key: 'expr', label: '関数の式' },
    { kind: 'list', key: 'period', label: '1周期の区間', item: 'bound', count: PAIR },
    { kind: 'bound', key: 'terms', label: '最高次数N' },
    { kind: 'select', key: 'mode', label: '描くもの', options: SERIES_MODES, optional: true },
    { kind: 'list', key: 'domain', label: '描く範囲', item: 'bound', count: PAIR, optional: true },
  ],
  fourier_transform: [
    { kind: 'text', key: 'var', label: '変数(省くとx)', optional: true },
    { kind: 'text', key: 'expr', label: '関数の式(iは虚数単位)' },
    { kind: 'list', key: 'support', label: '関数が0でない区間', item: 'bound', count: PAIR },
    { kind: 'select', key: 'part', label: '描く部分', options: TRANSFORM_PARTS, optional: true },
    { kind: 'list', key: 'domain', label: 'kの範囲', item: 'bound', count: PAIR, optional: true },
  ],
  fourier_intensity: [
    { kind: 'list', key: 'vars', label: '変数(2つ)', item: 'text', count: PAIR, optional: true },
    { kind: 'text', key: 'expr', label: '開口の関数の式' },
    { kind: 'json', key: 'support', label: '関数が0でない範囲', hint: '[[-1, 1], [-1, 1]]' },
    { kind: 'number', key: 'samples', label: '標本の数(各方向，省くと128)', optional: true },
    { kind: 'select', key: 'quantity', label: '表す量', options: QUANTITIES, optional: true },
    {
      kind: 'json',
      key: 'domain',
      label: '画像の範囲(省くと見える範囲)',
      optional: true,
      hint: '[[-10, 10], [-10, 10]]',
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
  diffraction: [
    {
      kind: 'json',
      key: 'basis',
      label: '実格子の基本ベクトル(2つか3つ)',
      hint: '[[1, 0, 0], [0, 1, 0], [0, 0, 1]]',
    },
    {
      kind: 'json',
      key: 'atoms',
      label: '単位胞の中の原子(分率座標と散乱因子)',
      optional: true,
      hint: '[{"position": [0, 0, 0]}, {"position": [0.5, 0.5, 0.5], "factor": 2}]',
    },
    { kind: 'list', key: 'zone', label: '晶帯軸[u v w]', item: 'number', count: 3, optional: true },
    { kind: 'bound', key: 'radius', label: '|G|の上限', optional: true },
    { kind: 'text', key: 'spot', label: '最も強い点の半径(3ptなど)', optional: true },
    { kind: 'checkbox', key: 'labels', label: '指数の名前を置く', initial: false },
    {
      kind: 'json',
      key: 'cells',
      label: '有限の結晶の単位胞の数(書くと画像)',
      optional: true,
      hint: '[8, 8, 8]',
    },
    {
      kind: 'list',
      key: 'range',
      label: '画像の色の両端の値',
      item: 'bound',
      count: PAIR,
      optional: true,
    },
    { kind: 'select', key: 'colormap', label: '画像の色', options: COLORMAPS, optional: true },
    { kind: 'select', key: 'scale', label: '画像の目盛', options: VALUE_SCALES, optional: true },
    ...RASTER_RESOLUTIONS,
  ],
};

export { ANALYSIS_FIELDS };
