import type { Json, JsonObject } from './json';
import type { ObjectTemplate } from './template-types';

/** 円周1周分と半周分(ラジアン)．媒介変数の範囲に使う，式(`pi`)で正確な値． */
const FULL_TURN_EXPR = '2*pi';
const HALF_TURN_EXPR = 'pi';
/** 円周2周分．らせんのように，2周させる曲線の範囲に使う． */
const TWO_TURNS_EXPR = '4*pi';
const FULL_TURN_DOMAIN: [Json, Json] = [0, FULL_TURN_EXPR];

/** 媒介変数`t`の曲線．`expr`の個数(2か3)で，平面と空間のどちらの曲線かが決まる． */
function parametricCurve(id: string, expr: readonly string[], domain: [Json, Json]): JsonObject {
  return { id, type: 'curve', var: 't', expr: [...expr], domain };
}

/** 放物線と双曲線の媒介変数の範囲．既定の見える範囲(縦`[-3, 3]`)に収まるように選ぶ． */
const PARABOLA_T = 2.4;
const HYPERBOLA_T = 1.5;
/** 双曲線の漸近線を引く範囲． */
const ASYMPTOTE_X = 3;
const ASYMPTOTE_STYLE: JsonObject = { line: 'dashed', color: 'gray' };

/** 双曲線x^2 - y^2 = 1の2つの枝．どちらも，双曲線関数による媒介変数表示． */
const HYPERBOLA_BRANCHES: readonly JsonObject[] = [
  parametricCurve('right', ['cosh(t)', 'sinh(t)'], [-HYPERBOLA_T, HYPERBOLA_T]),
  parametricCurve('left', ['-cosh(t)', 'sinh(t)'], [-HYPERBOLA_T, HYPERBOLA_T]),
];

/** 2次曲線．どれも媒介変数表示の曲線で書くので，式を変えて，大きさや向きを変えられる． */
const CONIC_TEMPLATES: readonly ObjectTemplate[] = [
  {
    id: 'circle',
    label: '円：x^2 + y^2 = 4',
    kind: 'plane',
    objects: [parametricCurve('c', ['2*cos(t)', '2*sin(t)'], FULL_TURN_DOMAIN)],
  },
  {
    id: 'ellipse',
    label: '楕円：x^2/9 + y^2/4 = 1',
    kind: 'plane',
    objects: [parametricCurve('c', ['3*cos(t)', '2*sin(t)'], FULL_TURN_DOMAIN)],
  },
  {
    id: 'parabola',
    label: '放物線：y = x^2/2',
    kind: 'plane',
    objects: [parametricCurve('c', ['t', 't^2/2'], [-PARABOLA_T, PARABOLA_T])],
  },
  {
    id: 'hyperbola',
    label: '双曲線：x^2 - y^2 = 1',
    kind: 'plane',
    objects: HYPERBOLA_BRANCHES,
  },
  {
    id: 'hyperbola-asymptotes',
    label: '双曲線と漸近線：x^2 - y^2 = 1，y = ±x',
    kind: 'plane',
    objects: [
      ...HYPERBOLA_BRANCHES,
      {
        id: 'asymptote1',
        type: 'graph',
        var: 'x',
        expr: 'x',
        domain: [-ASYMPTOTE_X, ASYMPTOTE_X],
        style: ASYMPTOTE_STYLE,
      },
      {
        id: 'asymptote2',
        type: 'graph',
        var: 'x',
        expr: '-x',
        domain: [-ASYMPTOTE_X, ASYMPTOTE_X],
        style: ASYMPTOTE_STYLE,
      },
    ],
  },
];

/** サイクロイドの範囲．転がる円が2周する． */
const CYCLOID_DOMAIN: [Json, Json] = [`-${FULL_TURN_EXPR}`, FULL_TURN_EXPR];

/** よく使う平面曲線．どれも，既定の見える範囲に収まる大きさにする． */
const PLANE_CURVE_TEMPLATES: readonly ObjectTemplate[] = [
  {
    id: 'cycloid',
    label: 'サイクロイド',
    kind: 'plane',
    objects: [parametricCurve('c', ['0.7*(t - sin(t))', '0.7*(1 - cos(t))'], CYCLOID_DOMAIN)],
  },
  {
    id: 'cardioid',
    label: 'カージオイド：r = 1 + cos θ',
    kind: 'plane',
    objects: [
      parametricCurve('c', ['(1 + cos(t))*cos(t)', '(1 + cos(t))*sin(t)'], FULL_TURN_DOMAIN),
    ],
  },
  {
    id: 'astroid',
    label: 'アステロイド',
    kind: 'plane',
    objects: [parametricCurve('c', ['2*cos(t)^3', '2*sin(t)^3'], FULL_TURN_DOMAIN)],
  },
  {
    id: 'lissajous',
    label: 'リサジュー曲線',
    kind: 'plane',
    objects: [parametricCurve('c', ['2*sin(3*t)', '2*sin(2*t)'], FULL_TURN_DOMAIN)],
  },
  {
    id: 'rose',
    label: '正葉曲線：r = 2cos 3θ',
    kind: 'plane',
    objects: [
      parametricCurve('c', ['2*cos(3*t)*cos(t)', '2*cos(3*t)*sin(t)'], [0, HALF_TURN_EXPR]),
    ],
  },
];

/** 空間曲線．どれも，既定の軸の範囲(`[-3, 3]`)に収まる大きさにする． */
const SPACE_CURVE_TEMPLATES: readonly ObjectTemplate[] = [
  {
    id: 'helix',
    label: 'らせん',
    kind: 'space',
    objects: [parametricCurve('c', ['1.5*cos(t)', '1.5*sin(t)', 't/(2*pi)'], [0, TWO_TURNS_EXPR])],
  },
  {
    id: 'conical-helix',
    label: '円錐らせん',
    kind: 'space',
    objects: [
      parametricCurve('c', ['t*cos(t)/(2*pi)', 't*sin(t)/(2*pi)', 't/(2*pi)'], [0, TWO_TURNS_EXPR]),
    ],
  },
  {
    id: 'viviani',
    label: 'ビビアーニの曲線(球と円柱の交線)',
    kind: 'space',
    objects: [parametricCurve('c', ['1 + cos(t)', 'sin(t)', '2*sin(t/2)'], [0, TWO_TURNS_EXPR])],
  },
  {
    id: 'trefoil',
    label: '三葉結び目',
    kind: 'space',
    objects: [
      parametricCurve(
        'c',
        ['sin(t) + 2*sin(2*t)', 'cos(t) - 2*cos(2*t)', '-sin(3*t)'],
        FULL_TURN_DOMAIN,
      ),
    ],
  },
];

export { CONIC_TEMPLATES, PLANE_CURVE_TEMPLATES, SPACE_CURVE_TEMPLATES };
