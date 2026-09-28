import { cosOf, sinOf } from './bravais-cells';
import { SCENE_VERSION } from './draft';
import type { SceneDraft } from './draft';
import type { JsonObject } from './json';
import {
  foldAmount,
  guide,
  planeLatticeCopy,
  rotationAngle,
  stepped,
  twoCopies,
} from './lattice-symmetry-parts';
import { CENTERED, CENTERED_A, CENTERED_B } from './plane-point-group-scenes';
import { SQUARE, latticeConstants, parametricCopies, view } from './symmetry-sample-scenes';
import type { SceneTemplate } from './template-types';

/**
 * 平面の格子の対称操作の見本(`symmetry-sample-scenes.ts`の回転・鏡映・並進の続き)．格子点でない点を中心とする
 * 回転，回転の角の制限(結晶学的制限)，映進．平面の等長変換は，並進，回転，鏡映，映進の4種類なので，
 * 前の3つの見本と合わせて，格子の対称操作の種類が出そろう．
 */

const GAMMA = 'angle_gamma';
/** 回転の中心の，格子の座標の分母．1/2(辺の中点，胞の中心)と1/3(三角形の重心)を，ちょうど選べる． */
const CENTER_DENOMINATOR = 6;
/** 回転の中心の始めの位置(胞の中心，p = q = 3)． */
const CENTER_START = 3;

/** 格子の座標(p/6, q/6)の点の，x座標とy座標の式． */
function centerAt(): [string, string] {
  const u = `p/${CENTER_DENOMINATOR}`;
  const v = `q/${CENTER_DENOMINATOR}`;
  return [`${u} + ${v}*b*${cosOf(GAMMA)}`, `${v}*b*${sinOf(GAMMA)}`];
}

const CENTER_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '格子点でない点を中心とする回転．青の格子を，点C(赤)のまわりに角θだけ回す．Cは格子の座標で(p/6, q/6)，つまりC = (p/6)a_1 + (q/6)a_2にある．正方格子(b = 1，γ = 90°)では，格子点(p = q = 0)と胞の中心(p = q = 3)は90°ごとに重なる4回回転の中心，辺の中点(p = 3，q = 0)は180°ごとの2回回転の中心である．六方格子(b = 1，γ = 60°)では，格子点が6回，三角形の重心(p = q = 2と，p = q = 4)が3回，辺の中点が2回の中心になる．一般の格子でも，格子点，辺の中点，胞の中心は，2回回転の中心である．',
  view: view(),
  objects: [
    ...latticeConstants(SQUARE),
    rotationAngle(),
    stepped({ id: 'p', value: CENTER_START, range: [0, CENTER_DENOMINATOR], step: 1 }),
    stepped({ id: 'q', value: CENTER_START, range: [0, CENTER_DENOMINATOR], step: 1 }),
    {
      id: 'C',
      type: 'point',
      at: centerAt(),
      label: 'C',
      anchor: 'south west',
      dot: true,
      style: { color: 'red' },
    },
    ...parametricCopies([{ rotate: 'theta', center: ['C_x', 'C_y'] }]),
  ],
};

/** 回転の回数nの上限．5回，7回，8回の回転が，どの格子でも重ならないことを確かめられる． */
const ORDER_END = 8;

const RESTRICTION_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '回転の角の制限(結晶学的制限)．青の格子を，原点のまわりに360°/nだけ回す．2つの格子が重なるのは，n = 1，2，3，4，6のときだけである．n = 2はどの格子でも，n = 4は正方格子(b = 1，γ = 90°)で，n = 3とn = 6は六方格子(b = 1，γ = 60°か120°)で重なる．n = 5，7，8では，格子定数をどう選んでも重ならない．格子の回転は格子ベクトルを格子ベクトルに移すので，基本ベクトルで書いた回転の行列の成分は整数になり，行列の跡2cos(360°/n)も整数でなければならないからである．',
  view: view(),
  objects: [
    ...latticeConstants(SQUARE),
    stepped({ id: 'n', value: 1, range: [1, ORDER_END], step: 1 }),
    ...parametricCopies([{ rotate: '360/n' }]),
  ],
};

/** 鏡の直線の端のx座標(見える範囲に収まる距離)． */
const MIRROR_REACH = 2.9;
/** 鏡の高さと映進の量の刻み(長方形の辺の1/4)． */
const GLIDE_STEP = 0.25;

/** 高さh bの，水平な鏡の直線． */
function glideLine(): JsonObject[] {
  const height = `h*${CENTERED_B}`;
  return guide('mirror', [
    [-MIRROR_REACH, height],
    [MIRROR_REACH, height],
  ]);
}

const GLIDE_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description: `映進(鏡映と，鏡に沿う並進を続けた操作)．面心長方形格子(長方形の辺a = ${CENTERED_A}，b = ${CENTERED_B}と，その中心の格子点)の青の組を，高さy = h bの水平な直線(赤の破線)で鏡映し，直線に沿ってs aだけずらす．tは操作を施す割合で，t = 1で操作が済む．h = 0か0.5(s = 0)なら，鏡映だけで重なる．h = 0.25か0.75では，鏡映だけでも，ずらすだけでも重ならないが，s = 0.5(辺の半分)ずらすと重なる．これが映進である．中心の格子点を持たない長方形格子には，映進はない(鏡映と格子の並進に分けられるものしかない)．`,
  view: view(),
  objects: [
    foldAmount(),
    stepped({ id: 'h', value: GLIDE_STEP, range: [0, 1], step: GLIDE_STEP }),
    stepped({ id: 's', value: 0, range: [0, 1], step: GLIDE_STEP }),
    ...twoCopies(
      (look) => planeLatticeCopy(CENTERED, look),
      [
        { scale: [1, 'cos(t*pi)'], center: [0, `h*${CENTERED_B}`] },
        { translate: [`s*${CENTERED_A}*t`, 0] },
      ],
    ),
    ...glideLine(),
  ],
};

const PLANE_SYMMETRY_OPERATION_SAMPLES: readonly SceneTemplate[] = [
  { id: 'latticeRotationCenter', label: '格子点でない点を中心とする回転', scene: CENTER_SCENE },
  { id: 'crystallographicRestriction', label: '回転の角の制限', scene: RESTRICTION_SCENE },
  { id: 'latticeGlide', label: '映進(面心長方形格子)', scene: GLIDE_SCENE },
];

export { PLANE_SYMMETRY_OPERATION_SAMPLES };
