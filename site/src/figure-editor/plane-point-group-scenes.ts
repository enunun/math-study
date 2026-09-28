import { SCENE_VERSION } from './draft';
import type { SceneDraft } from './draft';
import type { JsonObject } from './json';
import {
  foldAmount,
  guide,
  planeLatticeCopy,
  rotationAngle,
  twoCopies,
} from './lattice-symmetry-parts';
import type { PlaneLattice } from './lattice-symmetry-parts';
import type { SceneTemplate } from './template-types';

/**
 * 平面の5つのBravais格子の点群(格子を重ねる，原点を動かさない操作の全体)．青の格子に，x軸での鏡映を割合tだけ
 * 施してから，原点のまわりに角θだけ回す．tが0か1で，θが15°刻みなので，θとtの組を全部試すと，点群の元を
 * 1つ残らず見つけられる．t = 1の操作は，x軸と角θ/2をなす直線(赤の破線)での鏡映である．
 */

const VIEW_END = 3;
const UNIT = '1cm';
/** 鏡の直線の端(原点から，見える範囲に収まる距離)． */
const MIRROR_REACH = 2.9;

const HALF = 0.5;
const HEX_HEIGHT = Math.sqrt(1 - HALF * HALF);
const OBLIQUE_X = 0.35;
const OBLIQUE_Y = 1.1;
const RECTANGLE_A = 1.4;
const CENTERED_A = 1.6;
const CENTERED_B = 1.1;

/** 基本ベクトルで張る平行四辺形の単位胞． */
function primitive(basis: PlaneLattice['basis']): PlaneLattice {
  const [[ax, ay], [bx, by]] = basis;
  return {
    basis,
    cell: [
      [0, 0],
      [ax, ay],
      [ax + bx, ay + by],
      [bx, by],
    ],
  };
}

const OBLIQUE = primitive([
  [1, 0],
  [OBLIQUE_X, OBLIQUE_Y],
]);
const RECTANGLE = primitive([
  [RECTANGLE_A, 0],
  [0, 1],
]);
/** 面心長方形格子．基本ベクトルは長方形の中心へ向かう2本で，単位胞には，見慣れた長方形(面心の胞)を描く． */
const CENTERED: PlaneLattice = {
  basis: [
    [CENTERED_A * HALF, CENTERED_B * HALF],
    [CENTERED_A * HALF, -CENTERED_B * HALF],
  ],
  cell: [
    [0, 0],
    [CENTERED_A, 0],
    [CENTERED_A, CENTERED_B],
    [0, CENTERED_B],
  ],
};
const SQUARE = primitive([
  [1, 0],
  [0, 1],
]);
const HEXAGONAL = primitive([
  [1, 0],
  [HALF, HEX_HEIGHT],
]);

/** 角θ/2の鏡の直線． */
function mirrorLine(): JsonObject[] {
  const cos = 'cos(theta*pi/360)';
  const sin = 'sin(theta*pi/360)';
  return guide('mirror', [
    [`-${MIRROR_REACH}*${cos}`, `-${MIRROR_REACH}*${sin}`],
    [`${MIRROR_REACH}*${cos}`, `${MIRROR_REACH}*${sin}`],
  ]);
}

function pointGroupScene(lattice: PlaneLattice, description: string): SceneDraft {
  return {
    version: SCENE_VERSION,
    description,
    view: {
      x: [-VIEW_END, VIEW_END],
      y: [-VIEW_END, VIEW_END],
      unit: { x: UNIT, y: UNIT },
    },
    objects: [
      rotationAngle(),
      foldAmount(),
      ...twoCopies(
        (look) => planeLatticeCopy(lattice, look),
        [{ scale: [1, 'cos(t*pi)'] }, { rotate: 'theta' }],
      ),
      ...mirrorLine(),
    ],
  };
}

const COMMON =
  '青の格子は，x軸での鏡映を割合tだけ施してから，原点のまわりに角θだけ回す．t = 0なら回転，t = 1ならx軸と角θ/2をなす直線(赤の破線)での鏡映である．';

const PLANE_POINT_GROUP_SAMPLES: readonly SceneTemplate[] = [
  {
    id: 'planeObliquePointGroup',
    label: '斜交格子の点群(2)',
    scene: pointGroupScene(
      OBLIQUE,
      `斜交格子の点群2．${COMMON}重なるのは，t = 0でθ = 0°，180°の2通りだけで，点群の元は恒等変換と2回回転の2つである．鏡映では重ならない．どんな格子も，180°の回転(原点についての反転)では重なる．`,
    ),
  },
  {
    id: 'planeRectanglePointGroup',
    label: '長方形格子の点群(2mm)',
    scene: pointGroupScene(
      RECTANGLE,
      `長方形格子の点群2mm．${COMMON}重なるのは，θ = 0°，180°とt = 0，1の4通りで，点群の元は，恒等変換，2回回転，長方形の辺に平行な2本の鏡(θ = 0°と180°で，鏡は角0°と90°)の4つである．`,
    ),
  },
  {
    id: 'planeCenteredRectanglePointGroup',
    label: '面心長方形格子の点群(2mm)',
    scene: pointGroupScene(
      CENTERED,
      `面心長方形格子の点群2mm．長方形の中心にも格子点がある格子で，基本ベクトルは中心へ向かう2本の同じ長さのベクトルである(菱形の格子と同じもの)．${COMMON}点群は長方形格子と同じ2mmで，重なるのはθ = 0°，180°とt = 0，1の4通りである．格子点の並びは違っても点群が同じなので，結晶系は同じである．`,
    ),
  },
  {
    id: 'planeSquarePointGroup',
    label: '正方格子の点群(4mm)',
    scene: pointGroupScene(
      SQUARE,
      `正方格子の点群4mm．${COMMON}重なるのは，θ = 0°，90°，180°，270°とt = 0，1の8通りである．点群の元は，恒等変換，90°，180°，270°の回転，辺に平行な2本と対角線に沿う2本の鏡(角0°，45°，90°，135°)の8つである．`,
    ),
  },
  {
    id: 'planeHexagonalPointGroup',
    label: '六方格子の点群(6mm)',
    scene: pointGroupScene(
      HEXAGONAL,
      `六方格子の点群6mm．${COMMON}重なるのは，θが60°の倍数で，t = 0，1の12通りである．点群の元は，恒等変換，60°ごとの5つの回転，30°ごとに並ぶ6本の鏡の12個で，平面の格子で最も大きい点群である．`,
    ),
  },
];

export { CENTERED, CENTERED_A, CENTERED_B, PLANE_POINT_GROUP_SAMPLES };
