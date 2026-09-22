import { listedType } from './create';
import type { ViewKind } from './draft';
import type { JsonObject } from './json';

/** 項目の入力欄の種類と，内容． */
type FieldSpec =
  | { kind: 'text'; key: string; label: string; optional?: boolean }
  | { kind: 'number'; key: string; label: string; optional?: boolean }
  | { kind: 'bound'; key: string; label: string; optional?: boolean }
  /** 数か式を並べる．`count`が`'dimension'`なら，平面で2個，空間で3個である． */
  | {
      kind: 'list';
      key: string;
      label: string;
      item: 'text' | 'number' | 'bound';
      count: number | 'dimension';
      optional?: boolean;
    }
  /** 2つの変数の範囲(`[[下端, 上端], [下端, 上端]]`)． */
  | { kind: 'domain2'; key: string; label: string }
  | { kind: 'select'; key: string; label: string; options: readonly Option[]; optional?: boolean }
  | { kind: 'checkbox'; key: string; label: string; initial: boolean }
  /** 座標(数か式の並び)か，点の式(`A + B`など)． */
  | { kind: 'position'; key: string; label: string }
  /** 別のオブジェクトの識別子． */
  | { kind: 'reference'; key: string; label: string; of: readonly string[] }
  | { kind: 'references'; key: string; label: string; of: readonly string[]; count: number }
  /** JSONの値を，そのまま書く． */
  | { kind: 'json'; key: string; label: string; optional?: boolean; hint: string }
  | { kind: 'style' }
  /** あれば描く，スタイルつきの項目(曲面のワイヤーフレームなど)．チェックボックスで有無を選ぶ． */
  | { kind: 'toggleStyle'; key: string; label: string };

type Option = readonly [value: string, label: string];

/** 座標の成分の名前． */
const COORDINATE_NAMES = { plane: ['x', 'y'], space: ['x', 'y', 'z'] } as const;

const PAIR = 2;
const TRIPLE = 3;

const ANCHORS: readonly Option[] = [
  ['center', '中央'],
  ['north', '上'],
  ['south', '下'],
  ['east', '右'],
  ['west', '左'],
  ['north east', '右上'],
  ['north west', '左上'],
  ['south east', '右下'],
  ['south west', '左下'],
];

const ARROWS: readonly Option[] = [
  ['stealth', '矢じりあり'],
  ['none', '矢じりなし'],
];

const PLANE_DIRECTIONS: readonly Option[] = [
  ['x', 'x軸'],
  ['y', 'y軸'],
];

const SPACE_DIRECTIONS: readonly Option[] = [...PLANE_DIRECTIONS, ['z', 'z軸']];

const POINTS = ['point'];
const SURFACES = ['surface'];

const DOMAIN: FieldSpec = {
  kind: 'list',
  key: 'domain',
  label: '範囲',
  item: 'bound',
  count: PAIR,
};
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
/** ベジエ曲面だけの，制御点の網の項目． */
const CONTROL_NET: FieldSpec = {
  kind: 'toggleStyle',
  key: 'control_net',
  label: '制御点の網を表示',
};

const AXIS: readonly FieldSpec[] = [
  { kind: 'select', key: 'arrow', label: '矢じり', options: ARROWS, optional: true },
  { kind: 'text', key: 'label', label: '名前', optional: true },
  { kind: 'list', key: 'range', label: '範囲', item: 'number', count: PAIR, optional: true },
  {
    kind: 'json',
    key: 'ticks',
    label: '目盛',
    optional: true,
    hint: '[{"at": 1, "label": "$1$"}]',
  },
];

/** 種類ごとの，識別子とスタイル以外の項目．軸の向きは，図の種類で選択肢が変わるので，別に足す． */
const SPECS: Readonly<Record<string, readonly FieldSpec[]>> = {
  axis: AXIS,
  label: [
    { kind: 'position', key: 'at', label: '位置' },
    { kind: 'select', key: 'anchor', label: '位置に合わせる部分', options: ANCHORS },
    { kind: 'text', key: 'tex', label: 'TeXの文字列' },
  ],
  parameter: [{ kind: 'number', key: 'value', label: '値' }],
  graph: [
    { kind: 'text', key: 'var', label: '変数の名前' },
    { kind: 'text', key: 'expr', label: '式' },
    DOMAIN,
  ],
  curve: [
    { kind: 'text', key: 'var', label: '変数の名前' },
    { kind: 'list', key: 'expr', label: '座標の式', item: 'text', count: 'dimension' },
    DOMAIN,
  ],
  bezierCurve: [
    {
      kind: 'json',
      key: 'bezier',
      label: '制御点',
      hint: '[[0,0],[1,2],[2,0]]',
    },
  ],
  splineCurve: [
    {
      kind: 'json',
      key: 'spline',
      label: '通る点',
      hint: '[[0,0],[1,2],[2,0],[3,1]]',
    },
  ],
  tangent_line: [
    { kind: 'reference', key: 'of', label: '接する対象', of: ['graph', 'curve'] },
    { kind: 'bound', key: 'at', label: '接する点' },
  ],
  grid: [
    { kind: 'bound', key: 'x_step', label: 'x方向の間隔', optional: true },
    { kind: 'bound', key: 'y_step', label: 'y方向の間隔', optional: true },
  ],
  point: [
    { kind: 'position', key: 'at', label: '位置' },
    { kind: 'text', key: 'label', label: '名前', optional: true },
    { kind: 'select', key: 'anchor', label: '名前を置く位置', options: ANCHORS, optional: true },
    { kind: 'checkbox', key: 'dot', label: '点の印を描く', initial: false },
  ],
  vector: [
    { kind: 'reference', key: 'from', label: '始点', of: POINTS },
    { kind: 'reference', key: 'to', label: '終点', of: POINTS },
    { kind: 'select', key: 'arrow', label: '矢じり', options: ARROWS, optional: true },
  ],
  segment: [
    { kind: 'reference', key: 'from', label: '一方の端', of: POINTS },
    { kind: 'reference', key: 'to', label: 'もう一方の端', of: POINTS },
  ],
  region: [
    { kind: 'references', key: 'between', label: '挟むグラフ', of: ['graph'], count: PAIR },
    { kind: 'list', key: 'domain', label: 'xの範囲', item: 'bound', count: PAIR },
    { kind: 'checkbox', key: 'hatch', label: '斜線で埋める', initial: true },
    { kind: 'number', key: 'angle', label: '斜線の角度(度)', optional: true },
    { kind: 'text', key: 'gap', label: '斜線の間隔(2mmなど)', optional: true },
    {
      kind: 'json',
      key: 'fill',
      label: '塗りつぶし',
      optional: true,
      hint: '{"color": "blue", "opacity": 0.25}',
    },
  ],
  sphere: [
    { kind: 'list', key: 'center', label: '中心', item: 'number', count: TRIPLE },
    { kind: 'number', key: 'radius', label: '半径' },
    WIREFRAME,
  ],
  surface: [
    { kind: 'list', key: 'vars', label: '変数の名前', item: 'text', count: PAIR },
    { kind: 'list', key: 'expr', label: 'x，y，zの式', item: 'text', count: TRIPLE },
    { kind: 'domain2', key: 'domain', label: '変数の範囲' },
    MESH,
    BOUNDARY,
    WIREFRAME,
  ],
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
    CONTROL_NET,
  ],
  cut: [
    { kind: 'reference', key: 'surface', label: '切る曲面', of: SURFACES },
    { kind: 'list', key: 'normal', label: '平面の法線', item: 'bound', count: TRIPLE },
    { kind: 'bound', key: 'offset', label: '平面の定数' },
  ],
  intersection: [
    { kind: 'references', key: 'surfaces', label: '交わる2つの曲面', of: SURFACES, count: PAIR },
  ],
  tangent_plane: [
    { kind: 'reference', key: 'of', label: '接する曲面', of: SURFACES },
    { kind: 'list', key: 'at', label: '接する点(変数の値)', item: 'bound', count: PAIR },
    { kind: 'bound', key: 'size', label: '半径(cm)' },
  ],
};

/** スタイルを持たない種類． */
const UNSTYLED = new Set(['label', 'parameter']);

const ID: FieldSpec = { kind: 'text', key: 'id', label: '識別子' };
const STYLE: FieldSpec = { kind: 'style' };

function directionField(kind: ViewKind): FieldSpec {
  return {
    kind: 'select',
    key: 'direction',
    label: '向き',
    options: kind === 'space' ? SPACE_DIRECTIONS : PLANE_DIRECTIONS,
  };
}

/** オブジェクトの項目．識別子で始まり，スタイルで終わる． */
function fieldsFor(object: JsonObject, kind: ViewKind): readonly FieldSpec[] {
  const type = listedType(object);
  const first = type === 'axis' ? [ID, directionField(kind)] : [ID];
  const last = UNSTYLED.has(type) ? [] : [STYLE];
  return [...first, ...(SPECS[type] ?? []), ...last];
}

/** 並びの入力欄ごとの名前．空間の座標なら成分の名前，そうでなければ「1番目」などである． */
function listNames(spec: Extract<FieldSpec, { kind: 'list' }>, kind: ViewKind): readonly string[] {
  const count = spec.count === 'dimension' ? COORDINATE_NAMES[kind].length : spec.count;
  if (kind === 'space' && spec.item !== 'text' && count === COORDINATE_NAMES.space.length) {
    return COORDINATE_NAMES.space;
  }
  return Array.from({ length: count }, (_, at) => `${at + 1}番目`);
}

export { ANCHORS, ARROWS, COORDINATE_NAMES, fieldsFor, listNames };
export type { FieldSpec, Option };
