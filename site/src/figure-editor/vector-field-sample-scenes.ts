import { SCENE_VERSION } from './draft';
import type { SceneDraft } from './draft';
import type { JsonObject } from './json';
import type { SceneTemplate } from './template-types';

/**
 * ベクトル場の見本．場は，物理や解析の教科書と同じ形のベクトルの式で書く．点電荷の位置は，数で書いた点なので，
 * 編集中の図でドラッグして動かせる．
 */

const X_END = 4;
const Y_END = 3;
const UNIT = '1cm';
const STEP = 0.5;
const CHARGE_X = 1.5;
const CHARGE_VALUE = 1;
const CHARGE_END = 2;
const CHARGE_STEP = 0.5;
/** そろえた矢印の長さ(刻みの0.8倍)． */
const ARROW_LENGTH = 0.4;

function view(): SceneDraft['view'] {
  return { x: [-X_END, X_END], y: [-Y_END, Y_END], unit: { x: UNIT, y: UNIT } };
}

/** 電荷の大きさの媒介変数．負の値にもでき，0.5刻みで動かせる． */
function charge(id: string, value: number): JsonObject {
  return {
    id,
    type: 'parameter',
    value,
    range: [-CHARGE_END, CHARGE_END],
    step: CHARGE_STEP,
  };
}

/** 電荷を置く点．数で位置を書くので，ドラッグで動かせる． */
function chargePoint(id: string, x: number, color: string): JsonObject {
  return {
    id,
    type: 'point',
    at: [x, 0],
    label: id,
    anchor: 'south east',
    dot: true,
    style: { color },
  };
}

/** 電気力線の起点の数と，電荷Q_1からの距離． */
const SEED_COUNT = 12;
const SEED_RADIUS = 0.15;
const FULL_TURN = 360;

/** 電荷のまわりの円に，等間隔に並べた起点．電荷をドラッグすると，起点もついてくる． */
function seedsAround(center: string): string[][] {
  return Array.from({ length: SEED_COUNT }, (_, k) => {
    const degrees = (FULL_TURN * k) / SEED_COUNT;
    return [
      `${center}_x + ${SEED_RADIUS}*cos(${degrees}*pi/180)`,
      `${center}_y + ${SEED_RADIUS}*sin(${degrees}*pi/180)`,
    ];
  });
}

const DIPOLE_FIELD = 'q1 * (r - Q1) / norm(r - Q1)^3 + q2 * (r - Q2) / norm(r - Q2)^3';

const DIPOLE_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '2つの点電荷の電場．点Q_1，Q_2にある電荷q_1，q_2がつくる電場E(r) = q_1 (r − Q_1)/|r − Q_1|^3 + q_2 (r − Q_2)/|r − Q_2|^3(定数は省く)を，向きだけをそろえた矢印(灰色)と，Q_1のまわりから引いた電気力線(青の流線)で描く．場は，教科書の式のまま，ベクトルの式で書いてある．編集中の図で電荷の点をドラッグして動かし，スライダーで電荷の大きさや符号を変えると，場と電気力線が追従する．',
  view: view(),
  objects: [
    charge('q1', CHARGE_VALUE),
    charge('q2', -CHARGE_VALUE),
    chargePoint('Q1', -CHARGE_X, 'red'),
    chargePoint('Q2', CHARGE_X, 'blue'),
    {
      id: 'E',
      type: 'vector_field',
      field: DIPOLE_FIELD,
      x_step: STEP,
      y_step: STEP,
      length: 'normalized',
      scale: ARROW_LENGTH,
      style: { color: 'gray' },
    },
    {
      id: 'lines',
      type: 'field_line',
      field: DIPOLE_FIELD,
      seeds: seedsAround('Q1'),
      style: { color: 'blue' },
    },
  ],
};

/** 勾配の場を描く関数f(x, y) = x^2 − y^2の，等高線を引く値． */
const LOW_LEVEL = 1;
const MIDDLE_LEVEL = 2;
const HIGH_LEVEL = 4;
const CONTOUR_VALUES = [
  -HIGH_LEVEL,
  -MIDDLE_LEVEL,
  -LOW_LEVEL,
  0,
  LOW_LEVEL,
  MIDDLE_LEVEL,
  HIGH_LEVEL,
];
const GRADIENT_SCALE = 0.1;
const GRADIENT_MAX = 0.4;
const GRADIENT_STEP = 0.5;

const GRADIENT_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '勾配の場と等高線．関数f(x, y) = x^2 − y^2の勾配∇f = (2x, −2y)の矢印を，等高線f = cに重ねる．勾配は，どこでも等高線に垂直で，fが増える向きを指し，等高線が詰まっている所ほど長い．矢印は，場の値の0.1倍で，長すぎる所は切ってある．',
  view: view(),
  objects: [
    {
      id: 'contours',
      type: 'level_curve',
      vars: ['x', 'y'],
      expr: ['x', 'y'],
      domain: [
        [-X_END, X_END],
        [-Y_END, Y_END],
      ],
      level: 'x^2 - y^2',
      values: CONTOUR_VALUES,
      style: { color: 'gray' },
    },
    {
      id: 'gradient',
      type: 'vector_field',
      field: ['2*r_x', '-2*r_y'],
      x_step: GRADIENT_STEP,
      y_step: GRADIENT_STEP,
      length: 'clamped',
      scale: GRADIENT_SCALE,
      max_length: GRADIENT_MAX,
      style: { color: 'blue' },
    },
  ],
};

/** 直線電流の見本の格子．xとyは-2から2まで，zは-1から1まで，刻み1である． */
const WIRE_END = 2;
const WIRE_Z_END = 1;
const WIRE_ARROW = 0.6;
/** 磁力線(同心円)の起点の，z軸からの距離． */
const WIRE_SEEDS = [1, WIRE_END];
const WIRE_LENGTH = 1.8;
const SPACE_AZIMUTH = 35;
const SPACE_ELEVATION = 25;

const MAGNETIC_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '直線電流のまわりの磁場．z軸に沿って流れる電流(赤の矢印)がつくる磁場B(r) = (e_z × r)/(r_x^2 + r_y^2)(定数は省く)を，向きだけをそろえた矢印(灰色)と，磁力線(青の流線．z = 0の面の同心円)で描く．e_zは点Zの位置ベクトルで，場は外積crossのまま書いてある．磁場は電流を軸とする円に沿って回り，右ねじの向きを向く．z軸の上では値が有限でないので，矢印を描かない．',
  view: { azimuth: SPACE_AZIMUTH, elevation: SPACE_ELEVATION, unit: UNIT },
  objects: [
    { id: 'Z', type: 'point', at: [0, 0, 1] },
    { id: 'wire_from', type: 'point', at: [0, 0, -WIRE_LENGTH] },
    { id: 'wire_to', type: 'point', at: [0, 0, WIRE_LENGTH] },
    {
      id: 'current',
      type: 'vector',
      from: 'wire_from',
      to: 'wire_to',
      style: { color: 'red', width: '1pt' },
    },
    {
      id: 'B',
      type: 'vector_field',
      field: 'cross(Z, r) / (r_x^2 + r_y^2)',
      x_step: 1,
      y_step: 1,
      z_step: 1,
      x_range: [-WIRE_END, WIRE_END],
      y_range: [-WIRE_END, WIRE_END],
      z_range: [-WIRE_Z_END, WIRE_Z_END],
      length: 'normalized',
      scale: WIRE_ARROW,
      style: { color: 'gray' },
    },
    {
      id: 'lines',
      type: 'field_line',
      field: 'cross(Z, r) / (r_x^2 + r_y^2)',
      seeds: WIRE_SEEDS.map((radius) => [radius, 0, 0]),
      length: `2*pi*${WIRE_END}`,
      direction: 'forward',
      style: { color: 'blue' },
    },
  ],
};

const SPACE_VECTOR_FIELD_SAMPLES: readonly SceneTemplate[] = [
  { id: 'wireMagneticField', label: '直線電流のまわりの磁場', scene: MAGNETIC_SCENE },
];

const VECTOR_FIELD_SAMPLES: readonly SceneTemplate[] = [
  { id: 'dipoleField', label: '2つの点電荷の電場', scene: DIPOLE_SCENE },
  { id: 'gradientField', label: '勾配の場と等高線', scene: GRADIENT_SCENE },
];

export { SPACE_VECTOR_FIELD_SAMPLES, VECTOR_FIELD_SAMPLES };
