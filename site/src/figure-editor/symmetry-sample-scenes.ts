import { cosOf, sinOf } from './bravais-cells';
import { SCENE_VERSION } from './draft';
import type { SceneDraft } from './draft';
import type { JsonObject } from './json';
import type { SceneTemplate } from './template-types';

/**
 * 格子の対称性の見本．同じ格子(格子点と単位胞)を2組置き，一方(灰色)はそのまま残し，もう一方(青)にだけ，
 * 媒介変数を使う対称操作の変換を施す．媒介変数を動かすと，操作が格子の対称操作になる値で，2組が重なる．
 * 格子は，a_1 = (1, 0)と，a_1と角γをなす長さbのa_2で張る．
 */

const GAMMA = 'angle_gamma';
/** 格子の座標(u, v)の点の，x座標とy座標の式． */
function latticeAt(u: number, v: number): [string, string] {
  return [`${u} + ${v}*b*${cosOf(GAMMA)}`, `${v}*b*${sinOf(GAMMA)}`];
}

/** 見える範囲．格子点は，範囲の外なら描かれない(TikZの図も広げない)． */
const VIEW_END = 3;
const UNIT = '1cm';
/**
 * 格子点を置く添字の範囲．格子定数の範囲(b ≥ 0.8，60度 ≤ γ ≤ 120度)で，見える範囲を回しても覆える大きさ
 * (半径は見える範囲の対角線の半分)にする．
 */
const INDEX_END = 6;
const B_MIN = 0.8;
const B_MAX = 1.5;
const B_STEP = 0.05;
const GAMMA_MIN = 60;
const GAMMA_MAX = 120;
const ANGLE_STEP = 5;
const FULL_TURN = 360;
const HALF_TURN = 180;

const GHOST_STYLE: JsonObject = { color: 'gray' };
const MOVED_STYLE: JsonObject = { color: 'blue' };
const CELL_GHOST_STYLE: JsonObject = { color: 'gray', line: 'dashed' };
const CELL_MOVED_STYLE: JsonObject = { color: 'blue', width: '1pt' };

function integersBetween(low: number, high: number): number[] {
  return Array.from({ length: high - low + 1 }, (_, index) => low + index);
}

/** 添字を識別子に使う形(負の数は`m`で書く)． */
function indexName(value: number): string {
  return value < 0 ? `m${-value}` : String(value);
}

interface Copy {
  /** 識別子の頭． */
  prefix: string;
  point: JsonObject;
  cell: JsonObject;
  /** 変換の手順．元の組では空である． */
  transform: readonly JsonObject[];
}

/** 格子の1組．格子点と，原点の単位胞(破線か太線)． */
function latticeCopy({ prefix, point, cell, transform }: Copy): JsonObject[] {
  const moved = (object: JsonObject): JsonObject =>
    transform.length > 0 ? { ...object, transform: [...transform] } : object;
  const indices = integersBetween(-INDEX_END, INDEX_END);
  const points = indices.flatMap((u) =>
    indices.map((v) =>
      moved({
        id: `${prefix}_${indexName(u)}_${indexName(v)}`,
        type: 'point',
        at: latticeAt(u, v),
        dot: true,
        style: point,
      }),
    ),
  );
  const corners: readonly (readonly [number, number])[] = [
    [0, 0],
    [1, 0],
    [1, 1],
    [0, 1],
  ];
  return [
    ...points,
    moved({
      id: `${prefix}_cell`,
      type: 'polygon',
      vertices: corners.map(([u, v]) => latticeAt(u, v)),
      style: cell,
    }),
  ];
}

/** 元の組(灰色)と，変換を施す組(青)． */
function twoCopies(transform: readonly JsonObject[]): JsonObject[] {
  return [
    ...latticeCopy({ prefix: 'fixed', point: GHOST_STYLE, cell: CELL_GHOST_STYLE, transform: [] }),
    ...latticeCopy({ prefix: 'moved', point: MOVED_STYLE, cell: CELL_MOVED_STYLE, transform }),
  ];
}

function parameter(id: string, value: number, range: readonly [number, number]): JsonObject {
  return { id, type: 'parameter', value, range: [...range] };
}

interface Stepped {
  id: string;
  value: number;
  range: readonly [number, number];
  step: number;
}

/** 刻みのある媒介変数．角度は5度刻みなどにして，対称操作の角にちょうど合わせられるようにする． */
function stepped({ id, value, range, step }: Stepped): JsonObject {
  return { ...parameter(id, value, range), step };
}

interface Lattice {
  b: number;
  gamma: number;
}

/** 格子定数の媒介変数．bは0.05刻み，γは5度刻みなので，正方格子と六方格子にちょうど合わせられる． */
function latticeConstants({ b, gamma }: Lattice): JsonObject[] {
  return [
    stepped({ id: 'b', value: b, range: [B_MIN, B_MAX], step: B_STEP }),
    stepped({ id: GAMMA, value: gamma, range: [GAMMA_MIN, GAMMA_MAX], step: ANGLE_STEP }),
  ];
}

function view(): SceneDraft['view'] {
  return { x: [-VIEW_END, VIEW_END], y: [-VIEW_END, VIEW_END], unit: { x: UNIT, y: UNIT } };
}

const SQUARE: Lattice = { b: 1, gamma: 90 };
const RECTANGLE: Lattice = { b: 1.4, gamma: 90 };
const OBLIQUE: Lattice = { b: 1.2, gamma: 70 };

const ROTATION_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '格子の回転対称．灰色の格子をそのまま残し，青の格子を原点のまわりに角θだけ回す．θが格子の回転対称の角になると，2つの格子が重なる．正方格子(b = 1，γ = 90°)では90°ごと，六方格子(b = 1，γ = 60°か120°)では60°ごと，一般の格子では180°ごとに重なる．単位胞は，格子が重なっても，別の単位胞に移ることがある．',
  view: view(),
  objects: [
    ...latticeConstants(SQUARE),
    stepped({ id: 'theta', value: 0, range: [0, FULL_TURN], step: ANGLE_STEP }),
    ...twoCopies([{ rotate: 'theta' }]),
  ],
};

/** 鏡の直線の端(原点から，見える範囲に収まる距離)． */
const MIRROR_REACH = 2.9;
const FOLD_STEP = 0.05;

const MIRROR_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '格子の鏡映対称．原点を通り，x軸と角φをなす直線(赤の破線)を鏡にする．青の格子は，鏡に垂直な向きにcos(πt)倍に縮めてから戻す変換で，t = 0では元のまま，t = 1で鏡映になる(tを動かすと，紙を鏡の直線で折り返すように見える)．t = 1で2つの格子が重なるのは，鏡が格子の鏡映対称の直線のときである．長方形格子ではφ = 0°，90°，正方格子ではさらに45°，135°，b = 1の菱形の格子では，φ = γ/2とγ/2 + 90°で重なる．',
  view: view(),
  objects: [
    ...latticeConstants(RECTANGLE),
    stepped({ id: 'phi', value: 0, range: [0, HALF_TURN], step: ANGLE_STEP }),
    stepped({ id: 't', value: 0, range: [0, 1], step: FOLD_STEP }),
    ...twoCopies([{ rotate: '-phi' }, { scale: [1, 'cos(t*pi)'] }, { rotate: 'phi' }]),
    {
      id: 'mirror_from',
      type: 'point',
      at: [`-${MIRROR_REACH}*${cosOf('phi')}`, `-${MIRROR_REACH}*${sinOf('phi')}`],
    },
    {
      id: 'mirror_to',
      type: 'point',
      at: [`${MIRROR_REACH}*${cosOf('phi')}`, `${MIRROR_REACH}*${sinOf('phi')}`],
    },
    {
      id: 'mirror',
      type: 'segment',
      from: 'mirror_from',
      to: 'mirror_to',
      style: { color: 'red', line: 'dashed' },
    },
  ],
};

/** 並進の係数の範囲．矢印の先端が，格子定数のどの値でも見える範囲に収まる大きさにする． */
const SHIFT_END = 1.5;
const SHIFT_STEP = 0.05;

const TRANSLATION_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '格子の並進対称．青の格子を，ベクトルs a_1 + t a_2だけ平行移動する(赤の矢印)．sとtがどちらも整数のとき，移動のベクトルは格子ベクトルになり，2つの格子が重なる．どんな格子も，格子ベクトルの並進では重なる．',
  view: view(),
  objects: [
    ...latticeConstants(OBLIQUE),
    stepped({ id: 's', value: 0, range: [-SHIFT_END, SHIFT_END], step: SHIFT_STEP }),
    stepped({ id: 't', value: 0, range: [-SHIFT_END, SHIFT_END], step: SHIFT_STEP }),
    ...twoCopies([{ translate: [`s + t*b*${cosOf(GAMMA)}`, `t*b*${sinOf(GAMMA)}`] }]),
    { id: 'origin', type: 'point', at: [0, 0] },
    {
      id: 'shift_end',
      type: 'point',
      at: [`s + t*b*${cosOf(GAMMA)}`, `t*b*${sinOf(GAMMA)}`],
    },
    {
      id: 'shift',
      type: 'vector',
      from: 'origin',
      to: 'shift_end',
      style: { color: 'red', width: '1pt' },
    },
  ],
};

const SYMMETRY_SAMPLES: readonly SceneTemplate[] = [
  { id: 'latticeRotation', label: '格子の回転対称', scene: ROTATION_SCENE },
  { id: 'latticeMirror', label: '格子の鏡映対称', scene: MIRROR_SCENE },
  { id: 'latticeTranslation', label: '格子の並進対称', scene: TRANSLATION_SCENE },
];

export { SQUARE, SYMMETRY_SAMPLES, latticeConstants, twoCopies as parametricCopies, view };
export type { Lattice };
