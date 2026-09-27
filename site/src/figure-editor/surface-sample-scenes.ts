import { SCENE_VERSION } from './draft';
import type { SceneDraft } from './draft';
import type { JsonObject } from './json';
import { SPACE_AZIMUTH, SPACE_ELEVATION, SPACE_UNIT, multiples } from './sample-parts';
import type { SceneTemplate } from './template-types';

/** 自己交差する曲面の見本の，見る向き．横から見て，交わる所を見やすくする． */
const SIDE_ELEVATION = 20;

/** クラインの壺の図の単位． */
const KLEIN_UNIT = '1.5cm';
/** 首が胴を1周で通り抜けるので，uの方向(首に沿う方向)の網を細かくする． */
const KLEIN_U_DIVISIONS = 96;
const KLEIN_V_DIVISIONS = 48;

/**
 * 壺の形のクラインの壺．uは首に沿う方向(0からπ)，vは断面の円の角度である．u = 0とu = πの断面は，
 * 同じ円を逆向きにたどるので，vの範囲を-π/2から3π/2にして，2つの縁が点ごとに逆順に重なる(継ぎ目として
 * 縁を描かない)ようにする．
 */
const KLEIN_BOTTLE_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    'クラインの壺をℝ^3に置いた図．首が胴の側面を通り抜けて底につながり，内側と外側の区別がない．向き付け不可能な閉曲面で，ℝ^3では首が胴と交わる所(自己交差)が避けられない．見本「クラインの壺(4次元)」は，交わらずに置けるℝ^4での姿である．',
  view: { azimuth: SPACE_AZIMUTH, elevation: SIDE_ELEVATION, unit: KLEIN_UNIT },
  objects: [
    {
      id: 'bottle',
      type: 'surface',
      vars: ['u', 'v'],
      expr: [
        '-2/15 * cos(u) * (3*cos(v) - 30*sin(u) + 90*cos(u)^4*sin(u) - 60*cos(u)^6*sin(u) + 5*cos(u)*cos(v)*sin(u))',
        '2/15 * (3 + 5*cos(u)*sin(u)) * sin(v)',
        '-1/15 * sin(u) * (3*cos(v) - 3*cos(u)^2*cos(v) - 48*cos(u)^4*cos(v) + 48*cos(u)^6*cos(v) - 60*sin(u) + 5*cos(u)*cos(v)*sin(u) - 5*cos(u)^3*cos(v)*sin(u) - 80*cos(u)^5*cos(v)*sin(u) + 80*cos(u)^7*cos(v)*sin(u))',
      ],
      domain: [
        [0, 'pi'],
        ['-pi/2', '3*pi/2'],
      ],
      mesh: [KLEIN_U_DIVISIONS, KLEIN_V_DIVISIONS],
      boundary: true,
      wireframe: {},
      wireframe_step: ['pi/12', 'pi/4'],
    },
  ],
};

/** 交差帽の図の単位． */
const CROSS_CAP_UNIT = '2cm';

/**
 * 交差帽．円板の縁の対蹠点を貼り合わせた射影平面を，ℝ^3に置いたもの．vが0の点(底)から，v = π/2の縁の
 * 円まで広がり，縁の円は2周分たどられる．
 */
const CROSS_CAP_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '交差帽．射影平面(円板の縁の向かい合う点どうしを貼り合わせた曲面)をℝ^3に置いた図で，上部の線分で曲面が自分自身と交わる．射影平面は向き付け不可能な閉曲面で，ℝ^3では自己交差が避けられない．見本「射影平面(4次元)」は，交わらずに置けるℝ^4での姿である．',
  view: { azimuth: SPACE_AZIMUTH, elevation: SIDE_ELEVATION, unit: CROSS_CAP_UNIT },
  objects: [
    {
      id: 'cap',
      type: 'surface',
      vars: ['u', 'v'],
      expr: ['sin(u) * sin(2*v) / 2', 'sin(2*u) * sin(v)^2', 'cos(2*u) * sin(v)^2'],
      domain: [
        ['-pi', 'pi'],
        [0, 'pi/2'],
      ],
      boundary: true,
      wireframe: {},
      wireframe_step: ['pi/6', 'pi/12'],
    },
  ],
};

/** 等高線の図の，曲面の範囲(xは±2.5，yは±2)，等高線図を置く高さ，網の細かさ，ワイヤーフレームの刻み． */
const CONTOUR_X_END = 2.5;
const CONTOUR_Y_END = 2;
const CONTOUR_DOMAIN = [
  [-CONTOUR_X_END, CONTOUR_X_END],
  [-CONTOUR_Y_END, CONTOUR_Y_END],
];
const MAP_HEIGHT = '-1.5';
const CONTOUR_X_DIVISIONS = 40;
const CONTOUR_Y_DIVISIONS = 32;
const HILL_WIREFRAME_STEP = 0.5;
/** 決まった高さの等高線(0.2刻みで6本)と，スライダーで動かす高さcの範囲(曲面のほぼ最低から最高まで)． */
const CONTOUR_STEP = 0.2;
const CONTOUR_COUNT = 6;
const CONTOUR_HEIGHTS = multiples(CONTOUR_STEP, CONTOUR_COUNT);
const HEIGHT_MIN = 0.05;
const HEIGHT_MAX = 1.45;
const INITIAL_HEIGHT = 0.5;
const HEIGHT_STYLE: JsonObject = { color: 'blue', width: '1.2pt' };

/** 高さ関数`f`の等値線を，写像`expr`(曲面の上か，下の平面)の上に引く． */
function contour(id: string, expr: readonly string[], values: JsonObject['values']): JsonObject {
  return {
    id,
    type: 'level_curve',
    vars: ['x', 'y'],
    expr: [...expr],
    domain: CONTOUR_DOMAIN,
    level: 'f(x, y)',
    values,
    mesh: [CONTOUR_X_DIVISIONS, CONTOUR_Y_DIVISIONS],
  };
}

const ON_SURFACE = ['x', 'y', 'f(x, y)'];
const ON_MAP = ['x', 'y', MAP_HEIGHT];

/**
 * 2つの山を持つ曲面z = f(x, y)と，その等高線．下の平面には，同じ等高線を真下に写した等高線図を置く．
 * 高さcの等高線は，cが2つの山の間の峠の高さを越えるときに，1本から2本に分かれる．
 */
const CONTOUR_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '2つの山を持つ曲面z = f(x, y)の等高線と，それを真下の平面に写した等高線図．青い線は，スライダーで動かす高さcの等高線である．cが山の間の峠の高さを越えると，1本の等高線が2本に分かれる．',
  view: { azimuth: SPACE_AZIMUTH, elevation: SPACE_ELEVATION, unit: SPACE_UNIT },
  objects: [
    { id: 'c', type: 'parameter', value: INITIAL_HEIGHT, range: [HEIGHT_MIN, HEIGHT_MAX] },
    {
      id: 'f',
      type: 'function',
      vars: ['x', 'y'],
      expr: '1.5*exp(-((x - 1)^2 + y^2)) + 1.2*exp(-((x + 1)^2 + (y - 0.3)^2))',
    },
    {
      id: 'hill',
      type: 'surface',
      vars: ['x', 'y'],
      expr: ON_SURFACE,
      domain: CONTOUR_DOMAIN,
      boundary: true,
      wireframe: { color: 'gray' },
      wireframe_step: [HILL_WIREFRAME_STEP, HILL_WIREFRAME_STEP],
    },
    { ...contour('contours', ON_SURFACE, CONTOUR_HEIGHTS), style: { color: 'gray' } },
    { ...contour('height', ON_SURFACE, ['c']), style: HEIGHT_STYLE },
    { ...contour('map', ON_MAP, CONTOUR_HEIGHTS), style: { color: 'gray' } },
    { ...contour('map_height', ON_MAP, ['c']), style: HEIGHT_STYLE },
  ],
};

const SURFACE_SAMPLE_SCENES: readonly SceneTemplate[] = [
  { id: 'kleinBottle', label: 'クラインの壺', scene: KLEIN_BOTTLE_SCENE },
  { id: 'crossCap', label: '交差帽(射影平面)', scene: CROSS_CAP_SCENE },
  { id: 'contour', label: '曲面の等高線', scene: CONTOUR_SCENE },
];

export { SURFACE_SAMPLE_SCENES };
