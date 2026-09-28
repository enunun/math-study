import type { FieldSpec } from './field-spec';

const PAIR = 2;
const TRIPLE = 3;

const MESH: FieldSpec = {
  kind: 'list',
  key: 'mesh',
  label: '網の細かさ',
  item: 'number',
  count: PAIR,
  optional: true,
};
const BOUNDARY: FieldSpec = {
  kind: 'checkbox',
  key: 'boundary',
  label: '縁を描く',
  initial: false,
};
/** 曲面と球で共通の，ワイヤーフレーム(u一定・v一定の断面，球では経線と緯線)の項目． */
const WIREFRAME: FieldSpec = {
  kind: 'toggleStyle',
  key: 'wireframe',
  label: 'ワイヤーフレームを表示',
};
/**
 * 曲面だけの，ワイヤーフレームの刻み(u方向，v方向)．断面は，刻みの整数倍の所に引く．球の経線・緯線の
 * 本数は，今のところ変えられない．
 */
const WIREFRAME_STEP: FieldSpec = {
  kind: 'list',
  key: 'wireframe_step',
  label: 'ワイヤーフレームの刻み(u方向，v方向)',
  item: 'bound',
  count: PAIR,
  optional: true,
};
/** Bézier曲面だけの，制御点の網の項目． */
const CONTROL_NET: FieldSpec = {
  kind: 'toggleStyle',
  key: 'control_net',
  label: '制御点の網を表示',
};

/** 2つの変数の式で書くもの(曲面と陰関数の曲線)で共通の，変数の名前，写像の式，変数の範囲． */
const TWO_VARIABLE_MAP: readonly FieldSpec[] = [
  { kind: 'list', key: 'vars', label: '変数の名前', item: 'text', count: PAIR },
  { kind: 'list', key: 'expr', label: 'x，y，zの式', item: 'text', count: TRIPLE },
  { kind: 'domain2', key: 'domain', label: '変数の範囲' },
];

/** 球，曲面，Bézier曲面，陰関数の曲線の，識別子とスタイル以外の項目． */
const SURFACE_FIELDS: Readonly<Record<string, readonly FieldSpec[]>> = {
  sphere: [
    { kind: 'list', key: 'center', label: '中心', item: 'number', count: TRIPLE },
    { kind: 'number', key: 'radius', label: '半径' },
    WIREFRAME,
  ],
  surface: [...TWO_VARIABLE_MAP, MESH, BOUNDARY, WIREFRAME, WIREFRAME_STEP],
  bezier: [
    {
      kind: 'json',
      key: 'bezier',
      label: '制御点の網',
      hint: '[[[0,0,0],[0,1,0]],[[1,0,0],[1,1,1]]]',
    },
    MESH,
    BOUNDARY,
    WIREFRAME,
    WIREFRAME_STEP,
    CONTROL_NET,
  ],
  implicit_curve: [
    { kind: 'list', key: 'vars', label: '変数の名前', item: 'text', count: PAIR },
    { kind: 'list', key: 'expr', label: '写像の座標の式', item: 'text', count: 'dimension' },
    { kind: 'domain2', key: 'domain', label: '変数の範囲' },
    { kind: 'text', key: 'level', label: '値を比べる関数の式' },
    { kind: 'json', key: 'values', label: '線を引く値', hint: '[0, 0.5, "t"]' },
    MESH,
  ],
};

export { SURFACE_FIELDS };
