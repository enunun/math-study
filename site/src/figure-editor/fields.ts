import { listedType, TRANSFORMABLE } from './create';
import type { ViewKind } from './draft';
import { ANCHORS, ARROWS, PLANE_DIRECTIONS, SOLIDS, SPACE_DIRECTIONS } from './field-options';
import type { FieldSpec } from './field-spec';
import { stringOf } from './json';
import type { JsonObject } from './json';

/** 座標の成分の名前． */
const COORDINATE_NAMES = { plane: ['x', 'y'], space: ['x', 'y', 'z'] } as const;

const PAIR = 2;
const TRIPLE = 3;

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
/**
 * 変換(平行移動・回転・拡大縮小・対称移動・せん断・写像)．変換できる種類(`create.ts`の`TRANSFORMABLE`)は，
 * どれも最後にこの項目を持つ．
 */
const TRANSFORM: FieldSpec = { kind: 'transform', key: 'transform', label: '変換' };
/** 塗り(色と不透明度)．領域と多角形で共通． */
const FILL: FieldSpec = {
  kind: 'json',
  key: 'fill',
  label: '塗りつぶし',
  optional: true,
  hint: '{"color": "blue", "opacity": 0.25}',
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
    { kind: 'list', key: 'x_range', label: 'xの範囲', item: 'number', count: PAIR, optional: true },
    { kind: 'list', key: 'y_range', label: 'yの範囲', item: 'number', count: PAIR, optional: true },
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
    FILL,
  ],
  polygon: [
    { kind: 'number', key: 'sides', label: '辺の数' },
    { kind: 'list', key: 'center', label: '中心', item: 'bound', count: PAIR },
    { kind: 'bound', key: 'radius', label: '半径(中心から頂点まで)' },
    FILL,
  ],
  vertexPolygon: [
    { kind: 'json', key: 'vertices', label: '頂点(座標か点の名前)', hint: '[[0,0],[3,0],"A"]' },
    FILL,
  ],
  fractal: [
    {
      kind: 'json',
      key: 'base',
      label: '基本図形の点',
      hint: '[[0,0],[1,0]]',
    },
    { kind: 'checkbox', key: 'closed', label: '基本図形を閉じる', initial: false },
    {
      kind: 'json',
      key: 'transforms',
      label: '反復の変換(反復関数系)',
      hint: '[[{"scale":[0.5,0.5]},{"translate":[1,0]}]]',
    },
    { kind: 'number', key: 'depth', label: '再帰の深さ' },
    { kind: 'checkbox', key: 'all_depths', label: '途中の深さも重ねて描く', initial: false },
  ],
  taylor: [
    { kind: 'reference', key: 'of', label: '展開するグラフ', of: ['graph'] },
    { kind: 'bound', key: 'at', label: '展開の中心' },
    { kind: 'number', key: 'order', label: '次数' },
    { kind: 'list', key: 'domain', label: '描く範囲', item: 'bound', count: PAIR, optional: true },
  ],
  function: [
    { kind: 'json', key: 'vars', label: '引数の名前', hint: '["x"]' },
    { kind: 'text', key: 'expr', label: '式' },
  ],
  map: [
    { kind: 'list', key: 'vars', label: '座標の変数の名前', item: 'text', count: 'dimension' },
    { kind: 'list', key: 'expr', label: '写した先の座標の式', item: 'text', count: 'dimension' },
  ],
  image: [
    { kind: 'reference', key: 'of', label: '元のオブジェクト', of: TRANSFORMABLE },
    { kind: 'text', key: 'label', label: '名前(点の像だけ)', optional: true },
    TRANSFORM,
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
    WIREFRAME_STEP,
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
    WIREFRAME_STEP,
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
  polyhedron: [
    { kind: 'select', key: 'solid', label: '種類', options: SOLIDS },
    { kind: 'list', key: 'center', label: '中心', item: 'number', count: TRIPLE },
    { kind: 'number', key: 'radius', label: '半径(中心から頂点まで)' },
  ],
  complex: [
    { kind: 'json', key: 'vertices', label: '頂点', hint: '[[1,1,1],[1,-1,-1],[-1,1,-1]]' },
    {
      kind: 'json',
      key: 'faces',
      label: '面(頂点の番号，反時計回り)',
      hint: '[[0,1,2],[0,2,1]]',
    },
  ],
};

/** スタイルを持たない種類． */
const UNSTYLED = new Set(['label', 'parameter', 'function', 'map']);

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

/** オブジェクトの項目．識別子で始まり，変換できる種類なら変換，スタイルで終わる． */
function fieldsFor(object: JsonObject, kind: ViewKind): readonly FieldSpec[] {
  const type = listedType(object);
  const first = type === 'axis' ? [ID, directionField(kind)] : [ID];
  const transform = TRANSFORMABLE.includes(stringOf(object, 'type')) ? [TRANSFORM] : [];
  const last = [...transform, ...(UNSTYLED.has(type) ? [] : [STYLE])];
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

export { COORDINATE_NAMES, fieldsFor, listNames };
export type { FieldSpec } from './field-spec';
