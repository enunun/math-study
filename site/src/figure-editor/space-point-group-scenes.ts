import { SCENE_VERSION } from './draft';
import type { SceneDraft } from './draft';
import type { JsonObject } from './json';
import {
  HALF_TURN,
  foldAmount,
  guide,
  rotationAngle,
  selectBy,
  stepped,
  twoCopies,
} from './lattice-symmetry-parts';
import type { Vector3 } from './lattice-symmetry-parts';
import { ballSites, siteCopy } from './space-lattice-parts';
import type { Basis } from './space-lattice-parts';
import type { SceneTemplate } from './template-types';

/**
 * 空間の7つの晶系の格子の点群．どの格子も原点について対称なので，点群の元は，回転Rと，回転のあとの反転−Rで
 * 全部である．青の格子を，媒介変数kで選んだ回転軸のまわりに角θだけ回し，そのあと原点を中心に1 − 2t倍する
 * (t = 1で反転)．kで点群の回転軸を1本ずつ選び，θを動かすと，点群の回転が全部見つかる．t = 1にすると，
 * 鏡映(2回回転と反転の積)と回反(回転と反転の積)が得られる．
 */

const SPACE_AZIMUTH = -55;
const SPACE_ELEVATION = 22;
const UNIT = '1.5cm';
/** 回転軸を描く，原点からの長さ． */
const AXIS_REACH = 1.9;
/** 空間の座標の数． */
const DIMENSION = 3;

interface Axis {
  /** 軸の向き(数学の座標)． */
  direction: Vector3;
  /** 軸の名前(方向の指数など)． */
  name: string;
}

/** 式に書く数の有効数字．cos 90°の6e-17のような誤差を0にし，式に指数表記が出ないようにする． */
const COMPONENT_DIGITS = 12;

function unit([x, y, z]: Vector3): Vector3 {
  const length = Math.hypot(x, y, z);
  const clean = (value: number): number => Number((value / length).toFixed(COMPONENT_DIGITS));
  return [clean(x), clean(y), clean(z)];
}

/** 媒介変数kで選ぶ回転軸の，成分の式．軸が1本なら，数で書く． */
function axisComponents(axes: readonly Axis[]): (number | string)[] {
  const units = axes.map((axis) => unit(axis.direction));
  return Array.from({ length: DIMENSION }, (_, component) => {
    const values = units.map((vector) => vector[component] ?? 0);
    return values.length === 1 ? (values[0] ?? 0) : selectBy('k', values);
  });
}

/** 回転軸(赤の破線)． */
function axisGuide(components: readonly (number | string)[]): JsonObject[] {
  return guide('axis', [
    components.map((value) => `-${AXIS_REACH}*(${value})`),
    components.map((value) => `${AXIS_REACH}*(${value})`),
  ]);
}

interface System {
  basis: Basis;
  axes: readonly Axis[];
  description: string;
}

function pointGroupScene({ basis, axes, description }: System): SceneDraft {
  const copy = (look: Parameters<typeof siteCopy>[1]): JsonObject[] =>
    siteCopy(ballSites(basis), look);
  const components = axisComponents(axes);
  const rotation: JsonObject[] =
    axes.length === 0 ? [] : [{ rotate: 'theta', axis: [...components] }];
  const selector =
    axes.length > 1 ? [stepped({ id: 'k', value: 0, range: [0, axes.length - 1], step: 1 })] : [];
  return {
    version: SCENE_VERSION,
    description,
    view: { azimuth: SPACE_AZIMUTH, elevation: SPACE_ELEVATION, unit: UNIT },
    objects: [
      ...selector,
      ...(axes.length === 0 ? [] : [rotationAngle()]),
      foldAmount(),
      ...twoCopies(copy, [...rotation, { scale: '1 - 2*t' }]),
      ...(axes.length === 0 ? [] : axisGuide(components)),
    ],
  };
}

/** 軸の並びを，説明の文にする(「k = 0は[001]軸，…」)． */
function axisList(axes: readonly Axis[], offset = 0): string {
  return axes.map((axis, index) => `k = ${index + offset}は${axis.name}`).join('，');
}

const HALF = 0.5;
const ROOT3_HALF = Math.sqrt(1 - HALF * HALF);

function radian(degrees: number): number {
  return (degrees * Math.PI) / HALF_TURN;
}

/** 格子の基本ベクトルの長さと角(度)から，基本ベクトルを作る．aはx軸に，bはxy平面に置く． */
function basisOf({
  lengths: [a, b, c],
  angles: [alpha, beta, gamma],
}: {
  lengths: Vector3;
  angles: Vector3;
}): Basis {
  const [cosA, cosB, cosG] = [alpha, beta, gamma].map((angle) => Math.cos(radian(angle)));
  const sinG = Math.sin(radian(gamma));
  const cx = c * (cosB ?? 0);
  const cy = (c * ((cosA ?? 0) - (cosB ?? 0) * (cosG ?? 0))) / sinG;
  const cz = Math.sqrt(Math.max(c * c - cx * cx - cy * cy, 0));
  return [
    [a, 0, 0],
    [b * (cosG ?? 0), b * sinG, 0],
    [cx, cy, cz],
  ];
}

/* 各晶系の格子定数．長さと角が，より対称な格子と偶然に一致しないように選ぶ． */
const RIGHT = 90;
const TRICLINIC_B = 1.15;
const TRICLINIC_C = 1.3;
const TRICLINIC_ALPHA = 80;
const TRICLINIC_BETA = 105;
const TRICLINIC_GAMMA = 75;
const MONOCLINIC_B = 1.25;
const MONOCLINIC_C = 1.1;
const MONOCLINIC_BETA = 105;
const ORTHORHOMBIC_A = 0.9;
const ORTHORHOMBIC_B = 1.2;
const ORTHORHOMBIC_C = 1.4;
const TETRAGONAL_C = 1.4;
const RHOMBOHEDRAL_HEIGHT = 0.5;
const HEXAGONAL_C = 1.2;
const CUBIC_EDGE = 1;

const X: Axis = { direction: [1, 0, 0], name: '[100]軸' };
const Y: Axis = { direction: [0, 1, 0], name: '[010]軸' };
const Z: Axis = { direction: [0, 0, 1], name: '[001]軸' };

/** xy平面の中で，x軸と角(度)をなす軸． */
function inPlane(degrees: number): Axis {
  const angle = radian(degrees);
  return { direction: [Math.cos(angle), Math.sin(angle), 0], name: `xy平面の${degrees}°の向き` };
}

/** 符号の並びの方向の指数([1-10]など)の軸． */
function indexed(direction: Vector3): Axis {
  const name = direction.map((value) => (value < 0 ? `-${-value}` : String(value))).join('');
  return { direction, name: `[${name}]軸` };
}

const TETRAGONAL_TWOFOLD: readonly Axis[] = [X, Y, indexed([1, 1, 0]), indexed([1, -1, 0])];

const TRIGONAL_START = 90;
const TRIGONAL_STEP = 120;
const TRIGONAL_TWOFOLD_COUNT = 3;
const TRIGONAL_TWOFOLD = Array.from({ length: TRIGONAL_TWOFOLD_COUNT }, (_, index) =>
  inPlane(TRIGONAL_START + TRIGONAL_STEP * index),
);
const HEXAGONAL_STEP = 30;
const HEXAGONAL_TWOFOLD_COUNT = 6;
const HEXAGONAL_TWOFOLD = Array.from({ length: HEXAGONAL_TWOFOLD_COUNT }, (_, index) =>
  inPlane(HEXAGONAL_STEP * index),
);

const CUBIC_THREEFOLD: readonly Axis[] = [
  indexed([1, 1, 1]),
  indexed([-1, 1, 1]),
  indexed([1, -1, 1]),
  indexed([1, 1, -1]),
];
const CUBIC_TWOFOLD: readonly Axis[] = [
  indexed([1, 1, 0]),
  indexed([1, -1, 0]),
  indexed([1, 0, 1]),
  indexed([1, 0, -1]),
  indexed([0, 1, 1]),
  indexed([0, 1, -1]),
];

/** 菱面体格子．3本の基本ベクトルは同じ長さで，z軸のまわりに120°ずつ回した向きにある． */
const RHOMBOHEDRAL: Basis = [
  [1, 0, RHOMBOHEDRAL_HEIGHT],
  [-HALF, ROOT3_HALF, RHOMBOHEDRAL_HEIGHT],
  [-HALF, -ROOT3_HALF, RHOMBOHEDRAL_HEIGHT],
];

const INVERSION =
  't = 1(θ = 0°)は反転で，どの格子でも重なる．t = 1で重なる回転は，回転と反転の積(回反)も格子の対称操作であることを示す．2回回転と反転の積は，軸に垂直な面での鏡映である．';

const SYSTEMS: readonly (readonly [string, string, System])[] = [
  [
    'triclinicPointGroup',
    '三斜晶系の格子の点群(1̄)',
    {
      basis: basisOf({
        lengths: [1, TRICLINIC_B, TRICLINIC_C],
        angles: [TRICLINIC_ALPHA, TRICLINIC_BETA, TRICLINIC_GAMMA],
      }),
      axes: [],
      description: `三斜晶系の格子(三斜格子)の点群1̄．辺の長さも角も全部違う格子で，回転の対称性を持たない．青の格子を原点を中心に1 − 2t倍する．t = 1の反転で重なり，点群の元は恒等変換と反転の2つである．`,
    },
  ],
  [
    'monoclinicPointGroup',
    '単斜晶系の格子の点群(2/m)',
    {
      basis: basisOf({
        lengths: [1, MONOCLINIC_B, MONOCLINIC_C],
        angles: [RIGHT, MONOCLINIC_BETA, RIGHT],
      }),
      axes: [Y],
      description: `単斜晶系の格子(単純単斜格子)の点群2/m．b軸(y軸)がa軸とc軸に垂直で，a軸とc軸の角βは90°でない．青の格子をy軸(赤の破線)のまわりに角θだけ回し，1 − 2t倍する．重なるのはθ = 0°，180°とt = 0，1の4通りで，点群の元は，恒等変換，y軸のまわりの2回回転，反転，y軸に垂直な鏡映の4つである．${INVERSION}`,
    },
  ],
  [
    'orthorhombicPointGroup',
    '直方晶系の格子の点群(mmm)',
    {
      basis: basisOf({
        lengths: [ORTHORHOMBIC_A, ORTHORHOMBIC_B, ORTHORHOMBIC_C],
        angles: [RIGHT, RIGHT, RIGHT],
      }),
      axes: [X, Y, Z],
      description: `直方晶系の格子(単純直方格子，長さの違う3辺が直交する)の点群mmm．青の格子を，kで選んだ軸(${axisList([X, Y, Z])})のまわりに角θだけ回し，1 − 2t倍する．どの軸も180°ごとに重なる2回軸で，回転は恒等変換を含めて4つ，反転を掛けたものと合わせて，点群の元は8つである．${INVERSION}`,
    },
  ],
  [
    'tetragonalPointGroup',
    '正方晶系の格子の点群(4/mmm)',
    {
      basis: basisOf({ lengths: [1, 1, TETRAGONAL_C], angles: [RIGHT, RIGHT, RIGHT] }),
      axes: [Z, ...TETRAGONAL_TWOFOLD],
      description: `正方晶系の格子(単純正方格子，底面が正方形の直方体)の点群4/mmm．青の格子を，kで選んだ軸のまわりに角θだけ回し，1 − 2t倍する．k = 0は[001]軸で，90°ごとに重なる4回軸である．${axisList(TETRAGONAL_TWOFOLD, 1)}で，どれも180°ごとの2回軸である．回転は8つ，反転を掛けたものと合わせて，点群の元は16である．[001]軸の90°とt = 1で重なる操作は，4回回反4̄である．${INVERSION}`,
    },
  ],
  [
    'trigonalPointGroup',
    '三方晶系の格子の点群(3̄m)',
    {
      basis: RHOMBOHEDRAL,
      axes: [Z, ...TRIGONAL_TWOFOLD],
      description: `三方晶系の格子(菱面体格子．長さの等しい3本の基本ベクトルが，互いに同じ角をなす)の点群3̄m．z軸は3本の基本ベクトルの和の向きである．青の格子を，kで選んだ軸のまわりに角θだけ回し，1 − 2t倍する．k = 0はz軸で，120°ごとに重なる3回軸である．${axisList(TRIGONAL_TWOFOLD, 1)}で，どれも2回軸である．回転は6つ，点群の元は12である．z軸の120°とt = 1で重なる操作は，3回回反3̄である．z軸の60°では，t = 1でも重ならない(6回回反を持つのは六方晶系である)．${INVERSION}`,
    },
  ],
  [
    'hexagonalPointGroup',
    '六方晶系の格子の点群(6/mmm)',
    {
      basis: basisOf({ lengths: [1, 1, HEXAGONAL_C], angles: [RIGHT, RIGHT, TRIGONAL_STEP] }),
      axes: [Z, ...HEXAGONAL_TWOFOLD],
      description: `六方晶系の格子(六方格子．底面が120°の菱形)の点群6/mmm．青の格子を，kで選んだ軸のまわりに角θだけ回し，1 − 2t倍する．k = 0は[001]軸で，60°ごとに重なる6回軸である．${axisList(HEXAGONAL_TWOFOLD, 1)}で，どれも2回軸である．回転は12，点群の元は24で，立方晶系に次いで大きい．${INVERSION}`,
    },
  ],
  [
    'cubicPointGroup',
    '立方晶系の格子の点群(m3̄m)',
    {
      basis: [
        [CUBIC_EDGE, 0, 0],
        [0, CUBIC_EDGE, 0],
        [0, 0, CUBIC_EDGE],
      ],
      axes: [X, Y, Z, ...CUBIC_THREEFOLD, ...CUBIC_TWOFOLD],
      description: `立方晶系の格子(単純立方格子)の点群m3̄m．青の格子を，kで選んだ軸のまわりに角θだけ回し，1 − 2t倍する．k = 0，1，2は[100]，[010]，[001]軸で，90°ごとに重なる4回軸である．k = 3から6は体対角線の[111]，[-111]，[1-11]，[11-1]軸で，120°ごとの3回軸である．k = 7から12は面対角線の[110]，[1-10]，[101]，[10-1]，[011]，[01-1]軸で，2回軸である．回転は24，点群の元は48で，空間の格子で最も大きい点群である．${INVERSION}`,
    },
  ],
];

const SPACE_POINT_GROUP_SAMPLES: readonly SceneTemplate[] = SYSTEMS.map(([id, label, system]) => ({
  id,
  label,
  scene: pointGroupScene(system),
}));

export { SPACE_POINT_GROUP_SAMPLES };
