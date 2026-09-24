import type { Json, JsonObject } from './json';
import type { ObjectTemplate } from './template-types';

/**
 * 1周する角度の変数の範囲．継ぎ目(`±pi`)を，既定の向き(方位角60度)から見て裏側の，
 * x軸の負の側に置く．継ぎ目は定義域の端なので，ワイヤーフレームの断面が引かれない．
 */
const ANGLE_DOMAIN: [Json, Json] = ['-pi', 'pi'];
/** 角度の方向のワイヤーフレームの刻み．30度ごとに断面を引く． */
const ANGLE_STEP = 'pi/6';

interface SurfaceSpec {
  id: string;
  vars: [string, string];
  expr: [string, string, string];
  domain: [[Json, Json], [Json, Json]];
  step: [Json, Json];
}

/**
 * 式で書いた曲面．縁とワイヤーフレームを描き，刻みを添える．縁は，継ぎ目や1点に縮む辺をエンジンが除くので，
 * 定義域の端が本当の縁になる辺だけが描かれる．
 */
function formulaSurface({ id, vars, expr, domain, step }: SurfaceSpec): JsonObject {
  return {
    id,
    type: 'surface',
    vars: [...vars],
    expr: [...expr],
    domain: domain.map((pair) => [...pair]),
    boundary: true,
    wireframe: {},
    wireframe_step: [...step],
  };
}

/** 球の半径． */
const SPHERE_RADIUS = 2;
/** 双曲面の，双曲線関数の変数`u`の範囲の端と，その方向の刻み． */
const HYPERBOLOID_U = 1.2;
const HYPERBOLOID_STEP = 0.4;
/** 楕円放物面の，中心からの割合`r`の範囲の端と，その方向の刻み． */
const PARABOLOID_R = 1.5;
const PARABOLOID_STEP = 0.5;
/** 双曲放物面の，x，yの範囲の端と，その方向の刻み． */
const SADDLE_HALF = 2;
const SADDLE_STEP = 0.5;
/** 円錐と円柱の，高さの範囲の端と，その方向の刻み． */
const CONE_HALF = 1.5;
const CYLINDER_HALF = 1.5;
const HEIGHT_STEP = 0.5;

/**
 * 2次曲面．球はエンジンの`sphere`で書き，経線と緯線を描く．
 */
const QUADRIC_TEMPLATES: readonly ObjectTemplate[] = [
  {
    id: 'sphere',
    label: '球：x^2 + y^2 + z^2 = 4',
    kind: 'space',
    objects: [{ id: 's', type: 'sphere', center: [0, 0, 0], radius: SPHERE_RADIUS, wireframe: {} }],
  },
  {
    id: 'ellipsoid',
    label: '楕円面：x^2/4 + y^2/2.25 + z^2/1.44 = 1',
    kind: 'space',
    objects: [
      formulaSurface({
        id: 's',
        vars: ['u', 'v'],
        expr: ['2*sin(u)*cos(v)', '1.5*sin(u)*sin(v)', '1.2*cos(u)'],
        domain: [[0, 'pi'], ANGLE_DOMAIN],
        step: [ANGLE_STEP, ANGLE_STEP],
      }),
    ],
  },
  {
    id: 'hyperboloid-one-sheet',
    label: '一葉双曲面：x^2 + y^2 - z^2 = 1',
    kind: 'space',
    objects: [
      formulaSurface({
        id: 's',
        vars: ['u', 'v'],
        expr: ['cosh(u)*cos(v)', 'cosh(u)*sin(v)', 'sinh(u)'],
        domain: [[-HYPERBOLOID_U, HYPERBOLOID_U], ANGLE_DOMAIN],
        step: [HYPERBOLOID_STEP, ANGLE_STEP],
      }),
    ],
  },
  {
    id: 'hyperboloid-two-sheets',
    label: '二葉双曲面：z^2 - x^2 - y^2 = 1',
    kind: 'space',
    objects: [
      formulaSurface({
        id: 'upper',
        vars: ['u', 'v'],
        expr: ['sinh(u)*cos(v)', 'sinh(u)*sin(v)', 'cosh(u)'],
        domain: [[0, HYPERBOLOID_U], ANGLE_DOMAIN],
        step: [HYPERBOLOID_STEP, ANGLE_STEP],
      }),
      formulaSurface({
        id: 'lower',
        vars: ['u', 'v'],
        expr: ['sinh(u)*cos(v)', 'sinh(u)*sin(v)', '-cosh(u)'],
        domain: [[0, HYPERBOLOID_U], ANGLE_DOMAIN],
        step: [HYPERBOLOID_STEP, ANGLE_STEP],
      }),
    ],
  },
  {
    id: 'elliptic-paraboloid',
    label: '楕円放物面：z = x^2/4 + y^2/2.25',
    kind: 'space',
    objects: [
      formulaSurface({
        id: 's',
        vars: ['r', 't'],
        expr: ['2*r*cos(t)', '1.5*r*sin(t)', 'r^2'],
        domain: [[0, PARABOLOID_R], ANGLE_DOMAIN],
        step: [PARABOLOID_STEP, ANGLE_STEP],
      }),
    ],
  },
  {
    id: 'hyperbolic-paraboloid',
    label: '双曲放物面：z = (x^2 - y^2)/4',
    kind: 'space',
    objects: [
      formulaSurface({
        id: 's',
        vars: ['u', 'v'],
        expr: ['u', 'v', '(u^2 - v^2)/4'],
        domain: [
          [-SADDLE_HALF, SADDLE_HALF],
          [-SADDLE_HALF, SADDLE_HALF],
        ],
        step: [SADDLE_STEP, SADDLE_STEP],
      }),
    ],
  },
  {
    id: 'cone',
    label: '円錐：x^2 + y^2 = z^2',
    kind: 'space',
    objects: [
      formulaSurface({
        id: 's',
        vars: ['s', 't'],
        expr: ['s*cos(t)', 's*sin(t)', 's'],
        domain: [[-CONE_HALF, CONE_HALF], ANGLE_DOMAIN],
        step: [HEIGHT_STEP, ANGLE_STEP],
      }),
    ],
  },
  {
    id: 'cylinder',
    label: '円柱：x^2 + y^2 = 2.25',
    kind: 'space',
    objects: [
      formulaSurface({
        id: 's',
        vars: ['t', 'z'],
        expr: ['1.5*cos(t)', '1.5*sin(t)', 'z'],
        domain: [ANGLE_DOMAIN, [-CYLINDER_HALF, CYLINDER_HALF]],
        step: [ANGLE_STEP, HEIGHT_STEP],
      }),
    ],
  },
];

/** 常らせん面の，中心軸からの距離の範囲の端と，その方向の刻み． */
const HELICOID_R = 2;
const HELICOID_STEP = 0.5;
/** 関数のグラフの曲面の，x，yの範囲の端と，その方向の刻み． */
const GRAPH_HALF = 3;
const GRAPH_STEP = 0.5;

/** 2次曲面のほかの，よく使う曲面． */
const OTHER_SURFACE_TEMPLATES: readonly ObjectTemplate[] = [
  {
    id: 'torus',
    label: 'トーラス',
    kind: 'space',
    objects: [
      formulaSurface({
        id: 's',
        vars: ['u', 'v'],
        expr: ['(2 + 0.7*cos(v))*cos(u)', '(2 + 0.7*cos(v))*sin(u)', '0.7*sin(v)'],
        domain: [ANGLE_DOMAIN, ANGLE_DOMAIN],
        step: [ANGLE_STEP, ANGLE_STEP],
      }),
    ],
  },
  {
    id: 'helicoid',
    label: '常らせん面',
    kind: 'space',
    objects: [
      formulaSurface({
        id: 's',
        vars: ['r', 't'],
        expr: ['r*cos(t)', 'r*sin(t)', 't/pi'],
        domain: [
          [-HELICOID_R, HELICOID_R],
          [0, '2*pi'],
        ],
        step: [HELICOID_STEP, ANGLE_STEP],
      }),
    ],
  },
  {
    id: 'function-graph-surface',
    label: '関数のグラフ：z = sin(x)cos(y)',
    kind: 'space',
    objects: [
      formulaSurface({
        id: 's',
        vars: ['x', 'y'],
        expr: ['x', 'y', 'sin(x)*cos(y)'],
        domain: [
          [-GRAPH_HALF, GRAPH_HALF],
          [-GRAPH_HALF, GRAPH_HALF],
        ],
        step: [GRAPH_STEP, GRAPH_STEP],
      }),
    ],
  },
];

export { OTHER_SURFACE_TEMPLATES, QUADRIC_TEMPLATES };
