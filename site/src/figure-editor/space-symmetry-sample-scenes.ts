import { SCENE_VERSION } from './draft';
import type { SceneDraft } from './draft';
import type { JsonObject } from './json';
import type { SceneTemplate } from './template-types';

/**
 * 空間の格子の対称性の見本．平面の見本(`symmetry-sample-scenes.ts`)と同じく，同じ格子を2組置き，灰色の組は
 * そのまま残して，青の組にだけ媒介変数を使う変換を施す．格子は，添字が-1から1までの27個の格子点と，
 * 原点の単位胞の12本の辺である．格子点の塊は原点について対称なので，原点を通る対称操作で塊ごと重なる．
 * 単位胞の辺は，同じ組の格子点を結ぶので，変換は点にだけ施せば，辺もついてくる．
 */

type Vector = readonly [number, number, number];

/** 格子の基本ベクトル． */
type Basis = readonly [Vector, Vector, Vector];

const CUBIC: Basis = [
  [1, 0, 0],
  [0, 1, 0],
  [0, 0, 1],
];
/** 三斜格子．どの2辺も直交せず，長さも違う． */
const B_X = 0.4;
const B_Y = 1.1;
const C_X = 0.3;
const C_Y = 0.2;
const C_Z = 1.2;
const TRICLINIC: Basis = [
  [1, 0, 0],
  [B_X, B_Y, 0],
  [C_X, C_Y, C_Z],
];

const INDICES = [-1, 0, 1];
const SPACE_AZIMUTH = -55;
const SPACE_ELEVATION = 22;
const UNIT = '1.5cm';
const ANGLE_STEP = 5;
const FULL_TURN = 360;
const HALF_TURN = 180;
const FOLD_STEP = 0.05;
/** 回転軸と鏡の面を描く，原点からの長さ． */
const GUIDE_REACH = 1.8;

const GHOST_STYLE: JsonObject = { color: 'gray' };
const MOVED_STYLE: JsonObject = { color: 'blue' };
const EDGE_GHOST_STYLE: JsonObject = { color: 'gray', line: 'dashed' };
const EDGE_MOVED_STYLE: JsonObject = { color: 'blue', width: '1pt' };
const GUIDE_STYLE: JsonObject = { color: 'red', line: 'dashed' };

/** 添字を識別子に使う形(負の数は`m`で書く)． */
function indexName(value: number): string {
  return value < 0 ? `m${-value}` : String(value);
}

function pointId(prefix: string, [i, j, k]: Vector): string {
  return `${prefix}_${indexName(i)}_${indexName(j)}_${indexName(k)}`;
}

/** 格子の添字(i, j, k)の点の座標．小数の誤差を丸める． */
const COORDINATE_DIGITS = 6;
function latticeAt(basis: Basis, indices: Vector): number[] {
  const [first, second, third] = basis;
  const [i, j, k] = indices;
  return first.map((value, axis) =>
    Number(
      (i * value + j * (second[axis] ?? 0) + k * (third[axis] ?? 0)).toFixed(COORDINATE_DIGITS),
    ),
  );
}

/** 単位胞の頂点の添字(各成分が0か1)と，1成分だけが違う頂点の組(辺)． */
const CORNERS: readonly Vector[] = [0, 1].flatMap((i) =>
  [0, 1].flatMap((j) => [0, 1].map((k): Vector => [i, j, k])),
);
const EDGES: readonly (readonly [Vector, Vector])[] = CORNERS.flatMap((from) =>
  CORNERS.filter(
    (to) =>
      from.join(',') < to.join(',') &&
      from.filter((value, axis) => value !== to[axis]).length === 1,
  ).map((to) => [from, to] as const),
);

interface Copy {
  prefix: string;
  basis: Basis;
  point: JsonObject;
  edge: JsonObject;
  /** 格子点に施す変換．元の組では空である． */
  transform: readonly JsonObject[];
}

/** 格子の1組．格子点と，原点の単位胞の辺． */
function latticeCopy({ prefix, basis, point, edge, transform }: Copy): JsonObject[] {
  const moved = (object: JsonObject): JsonObject =>
    transform.length > 0 ? { ...object, transform: [...transform] } : object;
  const triples = INDICES.flatMap((i) =>
    INDICES.flatMap((j) => INDICES.map((k): Vector => [i, j, k])),
  );
  const points = triples.map((indices) =>
    moved({
      id: pointId(prefix, indices),
      type: 'point',
      at: latticeAt(basis, indices),
      dot: true,
      style: point,
    }),
  );
  const edges = EDGES.map(([from, to], index) => ({
    id: `${prefix}_edge${index + 1}`,
    type: 'segment',
    from: pointId(prefix, from),
    to: pointId(prefix, to),
    style: edge,
  }));
  return [...points, ...edges];
}

/** 元の組(灰色)と，変換を施す組(青)． */
function twoCopies(basis: Basis, transform: readonly JsonObject[]): JsonObject[] {
  return [
    ...latticeCopy({
      prefix: 'fixed',
      basis,
      point: GHOST_STYLE,
      edge: EDGE_GHOST_STYLE,
      transform: [],
    }),
    ...latticeCopy({
      prefix: 'moved',
      basis,
      point: MOVED_STYLE,
      edge: EDGE_MOVED_STYLE,
      transform,
    }),
  ];
}

interface Stepped {
  id: string;
  value: number;
  range: readonly [number, number];
  step: number;
}

function stepped({ id, value, range, step }: Stepped): JsonObject {
  return { id, type: 'parameter', value, range: [...range], step };
}

/** 2点を結ぶ補助の線(回転軸や鏡の面の縁)．両端の点と線分． */
function guide(
  id: string,
  [from, to]: readonly [readonly string[], readonly string[]],
): JsonObject[] {
  return [
    { id: `${id}_from`, type: 'point', at: [...from] },
    { id: `${id}_to`, type: 'point', at: [...to] },
    { id, type: 'segment', from: `${id}_from`, to: `${id}_to`, style: GUIDE_STYLE },
  ];
}

function view(): SceneDraft['view'] {
  return { azimuth: SPACE_AZIMUTH, elevation: SPACE_ELEVATION, unit: UNIT };
}

/**
 * 回転軸の向き．媒介変数k(0，1，2)で，[001](4回軸)，[111](3回軸)，[110](2回軸)を選ぶ．kの2次式で，
 * 3つの値を通るように書いてある．
 */
const AXIS: readonly [string, string, string] = ['k*(3 - k)/2', 'k*(3 - k)/2', '1 - k*(k - 1)/2'];
const AXIS_LENGTH = `sqrt(${AXIS.map((component) => `(${component})^2`).join(' + ')})`;
/** kの最大値(軸の種類は0，1，2の3つ)． */
const AXIS_KINDS = 2;

const ROTATION_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '立方格子の回転対称．灰色の格子をそのまま残し，青の格子を，原点を通る軸(赤の破線)のまわりに角θだけ回す．軸は媒介変数kで選ぶ．k = 0は[001]軸(90°ごとに重なる4回軸)，k = 1は[111]軸(体対角線．120°ごとに重なる3回軸)，k = 2は[110]軸(面対角線．180°ごとに重なる2回軸)である．格子が重なっても，単位胞は別の単位胞に移ることがある．',
  view: view(),
  objects: [
    stepped({ id: 'k', value: 0, range: [0, AXIS_KINDS], step: 1 }),
    stepped({ id: 'theta', value: 0, range: [0, FULL_TURN], step: ANGLE_STEP }),
    ...twoCopies(CUBIC, [{ rotate: 'theta', axis: [...AXIS] }]),
    ...guide('axis', [
      AXIS.map((component) => `-${GUIDE_REACH}*(${component})/${AXIS_LENGTH}`),
      AXIS.map((component) => `${GUIDE_REACH}*(${component})/${AXIS_LENGTH}`),
    ]),
  ],
};

const COS_PSI = 'cos(psi*pi/180)';
const SIN_PSI = 'sin(psi*pi/180)';

/** 鏡の面の縁．z軸と，面の中の水平な向き(-sin ψ, cos ψ, 0)で張る正方形である． */
function mirrorOutline(): JsonObject[] {
  const corner = (along: number, up: number): string[] => [
    `${-along * GUIDE_REACH}*${SIN_PSI}`,
    `${along * GUIDE_REACH}*${COS_PSI}`,
    String(up * GUIDE_REACH),
  ];
  const corners = [corner(-1, -1), corner(1, -1), corner(1, 1), corner(-1, 1)];
  return corners.flatMap((from, index) =>
    guide(`mirror${index + 1}`, [from, corners[(index + 1) % corners.length] ?? from]),
  );
}

const MIRROR_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '立方格子の鏡映対称．z軸を含み，法線がx軸と角ψをなす面(赤の破線の正方形)を鏡にする．青の格子は，法線の向きにcos(πt)倍に縮めてから戻す変換で，t = 0では元のまま，t = 1で鏡映になる．t = 1で2つの格子が重なるのは，ψ = 0°，45°，90°，135°のとき(立方体の面に平行な鏡と，対角線を含む鏡)である．',
  view: view(),
  objects: [
    stepped({ id: 'psi', value: 0, range: [0, HALF_TURN], step: ANGLE_STEP }),
    stepped({ id: 't', value: 0, range: [0, 1], step: FOLD_STEP }),
    ...twoCopies(CUBIC, [
      { rotate: '-psi', axis: [0, 0, 1] },
      { scale: ['cos(t*pi)', 1, 1] },
      { rotate: 'psi', axis: [0, 0, 1] },
    ]),
    ...mirrorOutline(),
  ],
};

const INVERSION_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '格子の反転対称．青の格子を，原点を中心に1 − 2t倍する．t = 0では元のまま，t = 0.5で原点に縮み，t = 1で反転(rを−rに移す操作)になって，2つの格子が重なる．格子点rと−rはどちらも格子点なので，三斜格子のように回転や鏡映の対称性を持たない格子も，反転では必ず重なる．単位胞は，原点の反対側の単位胞に移る．',
  view: view(),
  objects: [
    stepped({ id: 't', value: 0, range: [0, 1], step: FOLD_STEP }),
    ...twoCopies(TRICLINIC, [{ scale: '1 - 2*t' }]),
  ],
};

const SPACE_SYMMETRY_SAMPLES: readonly SceneTemplate[] = [
  { id: 'cubicRotation', label: '立方格子の回転対称', scene: ROTATION_SCENE },
  { id: 'cubicMirror', label: '立方格子の鏡映対称', scene: MIRROR_SCENE },
  { id: 'latticeInversion', label: '格子の反転対称', scene: INVERSION_SCENE },
];

export { SPACE_SYMMETRY_SAMPLES };
