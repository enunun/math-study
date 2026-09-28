import { SCENE_VERSION } from './draft';
import type { SceneDraft } from './draft';
import type { JsonObject } from './json';
import {
  SPACE_AZIMUTH,
  SPACE_ELEVATION,
  SPACE_UNIT,
  spaceAxes,
  symmetricMultiples,
} from './sample-parts';
import type { SceneTemplate } from './template-types';

/**
 * 4次元の図形を，座標空間と時間で見せる見本．4つ目の座標wを時刻とみなし，超平面w = tで切った切り口を，
 * その時刻の姿として描く．tは範囲のある媒介変数なので，図の作成のスライダーや再生で動かせる．
 */

/** 時刻tの範囲の端(-1から1まで)．どの見本も，wの値はこの範囲に収まる． */
const TIME_END = 1;
/**
 * 決まった時刻の切り口．薄い点線で重ね，時刻を動かしたときの全体の形の手がかりにする．
 * 0.4刻みで，-0.8から0.8までの5つである．
 */
const GHOST_TIME_STEP = 0.4;
const GHOST_TIMES_EACH_SIDE = 2;
const GHOST_TIMES = symmetricMultiples(GHOST_TIME_STEP, GHOST_TIMES_EACH_SIDE);
/** 最初に見せる時刻． */
const INITIAL_TIME = 0.2;
/** 薄い切り口と今の切り口のスタイル．写像が作る面は描かないので，どちらも陰線処理をしない． */
const GHOST_STYLE: JsonObject = { color: 'gray', line: 'dotted', hidden: 'visible' };
const SLICE_STYLE: JsonObject = { color: 'blue', width: '1.2pt', hidden: 'visible' };

function timeParameter(): JsonObject {
  return { id: 't', type: 'parameter', value: INITIAL_TIME, range: [-TIME_END, TIME_END] };
}

/** 4次元の曲面．2つの変数で，x，y，z座標の式と，4つ目の座標wの式を書く． */
interface FourSurface {
  vars: readonly [string, string];
  expr: readonly [string, string, string];
  w: string;
  domain: readonly [
    readonly [number | string, number | string],
    readonly [number | string, number | string],
  ];
  mesh: readonly [number, number];
}

/** 決まった時刻の切り口(`ghost`)と，今の時刻tの切り口(`slice`)．どちらも等値線で描く． */
function sliceObjects(surface: FourSurface): JsonObject[] {
  const shape: JsonObject = {
    type: 'implicit_curve',
    vars: [...surface.vars],
    expr: [...surface.expr],
    domain: surface.domain.map((range) => [...range]),
    level: surface.w,
    mesh: [...surface.mesh],
  };
  return [
    { id: 'ghost', ...shape, values: GHOST_TIMES, style: GHOST_STYLE },
    { id: 'slice', ...shape, values: ['t'], style: SLICE_STYLE },
  ];
}

/** 3次元球面の図の単位と，軸の範囲． */
const SPHERE_UNIT = '1.5cm';
const SPHERE_AXIS = 1.5;
/** 切り口の球の半径．時刻tでは√(1 − t²)である． */
const SLICE_RADIUS = 'sqrt(1 - t^2)';

/** 半径1の大円(時刻0の切り口の，赤道と経線)．点線で描き，切り口の大きさを比べる目安にする． */
function greatCircle(id: string, expr: readonly [string, string, string]): JsonObject {
  return { id, type: 'curve', var: 's', expr: [...expr], domain: [0, '2*pi'], style: GHOST_STYLE };
}

const FOUR_SPHERE_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '3次元球面x^2 + y^2 + z^2 + w^2 = 1を，超平面w = tで切った切り口．切り口は半径√(1 − t^2)の球である．tを−1から1まで動かすと，点として現れた球が膨らみ，t = 0で最も大きく(点線の円の大きさ)なり，縮んで消える．',
  view: { azimuth: SPACE_AZIMUTH, elevation: SPACE_ELEVATION, unit: SPHERE_UNIT },
  objects: [
    timeParameter(),
    ...spaceAxes([-SPHERE_AXIS, SPHERE_AXIS], [-SPHERE_AXIS, SPHERE_AXIS]),
    greatCircle('equator', ['cos(s)', 'sin(s)', '0']),
    greatCircle('meridian', ['cos(s)', '0', 'sin(s)']),
    {
      id: 'slice',
      type: 'surface',
      vars: ['a', 'b'],
      expr: [
        `${SLICE_RADIUS} * sin(a) * cos(b)`,
        `${SLICE_RADIUS} * sin(a) * sin(b)`,
        `${SLICE_RADIUS} * cos(a)`,
      ],
      domain: [
        [0, 'pi'],
        ['-pi', 'pi'],
      ],
      boundary: true,
      wireframe: { color: 'blue' },
      wireframe_step: ['pi/4', 'pi/4'],
      style: { color: 'blue' },
    },
  ],
};

/** Kleinの壺の軸の範囲と，網の細かさ(uの方向は1周で断面の円が半回転するので，細かくする)． */
const KLEIN_AXIS = 3.5;
const KLEIN_Z_AXIS = 1.5;
const KLEIN_U_DIVISIONS = 72;
const KLEIN_V_DIVISIONS = 36;

/**
 * ℝ⁴のKleinの壺．半径2の円に沿って，半径1の円を，1周で半回転させながら動かす．回る円のzとwの成分が
 * 入れ替わるので，ℝ³への射影は自己交差するが，ℝ⁴では交わらない．
 */
const KLEIN_BOTTLE_4D_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    'ℝ^4のKleinの壺((2 + cos v)cos u, (2 + cos v)sin u, sin v cos(u/2), sin v sin(u/2))を，超平面w = tで切った切り口．tを動かすと，切り口の曲線が形とつながり方を変える．点線は，決まった時刻の切り口である．Kleinの壺は向き付け不可能で，ℝ^3には自己交差なしに置けないが，ℝ^4には置ける．',
  view: { azimuth: SPACE_AZIMUTH, elevation: SPACE_ELEVATION, unit: SPACE_UNIT },
  objects: [
    timeParameter(),
    ...spaceAxes([-KLEIN_AXIS, KLEIN_AXIS], [-KLEIN_Z_AXIS, KLEIN_Z_AXIS]),
    ...sliceObjects({
      vars: ['u', 'v'],
      expr: ['(2 + cos(v)) * cos(u)', '(2 + cos(v)) * sin(u)', 'sin(v) * cos(u / 2)'],
      w: 'sin(v) * sin(u / 2)',
      domain: [
        [0, '2*pi'],
        [0, '2*pi'],
      ],
      mesh: [KLEIN_U_DIVISIONS, KLEIN_V_DIVISIONS],
    }),
  ],
};

/** 射影平面の図の単位と，軸の範囲，網の細かさ． */
const PROJECTIVE_UNIT = '1.5cm';
const PROJECTIVE_AXIS = 1.5;
const PROJECTIVE_Z_AXIS = 2.5;
const PROJECTIVE_LATITUDE_DIVISIONS = 36;
const PROJECTIVE_LONGITUDE_DIVISIONS = 72;

/** 球面の点(x, y, z)の成分を，緯度aと経度bで書いたもの． */
const SPHERE_X = 'sin(a) * cos(b)';
const SPHERE_Y = 'sin(a) * sin(b)';
const SPHERE_Z = 'cos(a)';

/**
 * 射影平面の，ℝ⁴への埋め込み(x, y, z) ↦ (xy, xz, y^2 − z^2, 2yz)．球面の対蹠点は同じ点に写るので，
 * 上半球(aが0からπ/2)だけを使う．x，y，z座標は，見やすいように2倍する．
 */
const PROJECTIVE_PLANE_4D_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '射影平面を，球面の点(x, y, z)をℝ^4の点(xy, xz, y^2 − z^2, 2yz)に写して埋め込み，超平面w = 2yz = tで切った切り口(x，y，z座標は2倍して描く)．(x, y, z)と(−x, −y, −z)は同じ点に写るので，球面を2つ同一視した射影平面になる．射影平面は向き付け不可能で，ℝ^3には自己交差なしに置けないが，ℝ^4には置ける．',
  view: { azimuth: SPACE_AZIMUTH, elevation: SPACE_ELEVATION, unit: PROJECTIVE_UNIT },
  objects: [
    timeParameter(),
    ...spaceAxes([-PROJECTIVE_AXIS, PROJECTIVE_AXIS], [-PROJECTIVE_Z_AXIS, PROJECTIVE_Z_AXIS]),
    ...sliceObjects({
      vars: ['a', 'b'],
      expr: [
        `2 * ${SPHERE_X} * ${SPHERE_Y}`,
        `2 * ${SPHERE_X} * ${SPHERE_Z}`,
        `2 * ((${SPHERE_Y})^2 - (${SPHERE_Z})^2)`,
      ],
      w: `2 * ${SPHERE_Y} * ${SPHERE_Z}`,
      domain: [
        [0, 'pi/2'],
        ['-pi', 'pi'],
      ],
      mesh: [PROJECTIVE_LATITUDE_DIVISIONS, PROJECTIVE_LONGITUDE_DIVISIONS],
    }),
  ],
};

const FOUR_SPACE_SAMPLE_SCENES: readonly SceneTemplate[] = [
  { id: 'fourSphere', label: '3次元球面の断面(4次元)', scene: FOUR_SPHERE_SCENE },
  { id: 'kleinBottle4d', label: 'Kleinの壺(4次元)', scene: KLEIN_BOTTLE_4D_SCENE },
  { id: 'projectivePlane4d', label: '射影平面(4次元)', scene: PROJECTIVE_PLANE_4D_SCENE },
];

export { FOUR_SPACE_SAMPLE_SCENES };
