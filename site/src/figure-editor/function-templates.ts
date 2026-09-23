import type { Json } from './json';
import type { ObjectTemplate } from './template-types';

/** 定義域の境界に使う，名前のついた値．関数ごとに，特異点や急な増加を避けて選ぶ． */
const JUST_ABOVE_ZERO = 0.05;
const NARROW_BOUND = 1.4;
const MODERATE_BOUND = 3;
const WIDE_BOUND = 4;
const DEFAULT_BOUND = 5;
const WIDER_BOUND = 10;
const OSCILLATION_BOUND = 20;
const LAMBERT_W_LOWER_BOUND = -0.36;
/** 円周1周分(ラジアン)．正弦・余弦の定義域に使う，式(`pi`)で正確な値． */
const FULL_TURN_EXPR = '2*pi';

/** 関数のグラフのテンプレート．それぞれ，定義域を，特異点や急な増加を避けて選ぶ． */
const FUNCTION_SPECS: readonly {
  id: string;
  name: string;
  expr: string;
  domain: [Json, Json];
}[] = [
  { id: 'sin', name: '正弦sin', expr: 'sin(x)', domain: [`-${FULL_TURN_EXPR}`, FULL_TURN_EXPR] },
  { id: 'cos', name: '余弦cos', expr: 'cos(x)', domain: [`-${FULL_TURN_EXPR}`, FULL_TURN_EXPR] },
  { id: 'tan', name: '正接tan', expr: 'tan(x)', domain: [-NARROW_BOUND, NARROW_BOUND] },
  { id: 'asin', name: '逆正弦asin', expr: 'asin(x)', domain: [-1, 1] },
  { id: 'acos', name: '逆余弦acos', expr: 'acos(x)', domain: [-1, 1] },
  { id: 'atan', name: '逆正接atan', expr: 'atan(x)', domain: [-WIDER_BOUND, WIDER_BOUND] },
  {
    id: 'sinh',
    name: '双曲線正弦sinh',
    expr: 'sinh(x)',
    domain: [-MODERATE_BOUND, MODERATE_BOUND],
  },
  {
    id: 'cosh',
    name: '双曲線余弦cosh',
    expr: 'cosh(x)',
    domain: [-MODERATE_BOUND, MODERATE_BOUND],
  },
  {
    id: 'tanh',
    name: '双曲線正接tanh',
    expr: 'tanh(x)',
    domain: [-MODERATE_BOUND, MODERATE_BOUND],
  },
  { id: 'exp', name: '指数関数exp', expr: 'exp(x)', domain: [-MODERATE_BOUND, MODERATE_BOUND] },
  { id: 'log', name: '自然対数log', expr: 'log(x)', domain: [JUST_ABOVE_ZERO, DEFAULT_BOUND] },
  { id: 'sqrt', name: '平方根sqrt', expr: 'sqrt(x)', domain: [0, DEFAULT_BOUND] },
  { id: 'abs', name: '絶対値abs', expr: 'abs(x)', domain: [-MODERATE_BOUND, MODERATE_BOUND] },
  { id: 'gamma', name: 'ガンマ関数', expr: 'gamma(x)', domain: [JUST_ABOVE_ZERO, DEFAULT_BOUND] },
  {
    id: 'loggamma',
    name: '対数ガンマ関数',
    expr: 'loggamma(x)',
    domain: [JUST_ABOVE_ZERO, DEFAULT_BOUND],
  },
  { id: 'erf', name: '誤差関数erf', expr: 'erf(x)', domain: [-MODERATE_BOUND, MODERATE_BOUND] },
  {
    id: 'erfc',
    name: '相補誤差関数erfc',
    expr: 'erfc(x)',
    domain: [-MODERATE_BOUND, MODERATE_BOUND],
  },
  { id: 'dawson', name: 'ドーソン関数', expr: 'dawson(x)', domain: [-WIDE_BOUND, WIDE_BOUND] },
  {
    id: 'lambertw',
    name: 'ランベルトのW関数',
    expr: 'lambertw(x)',
    domain: [LAMBERT_W_LOWER_BOUND, DEFAULT_BOUND],
  },
  {
    id: 'besselj0',
    name: '第1種ベッセル関数J0',
    expr: 'besselj0(x)',
    domain: [0, OSCILLATION_BOUND],
  },
  {
    id: 'besselj1',
    name: '第1種ベッセル関数J1',
    expr: 'besselj1(x)',
    domain: [0, OSCILLATION_BOUND],
  },
  {
    id: 'bessely0',
    name: '第2種ベッセル関数Y0',
    expr: 'bessely0(x)',
    domain: [JUST_ABOVE_ZERO, OSCILLATION_BOUND],
  },
  {
    id: 'bessely1',
    name: '第2種ベッセル関数Y1',
    expr: 'bessely1(x)',
    domain: [JUST_ABOVE_ZERO, OSCILLATION_BOUND],
  },
  {
    id: 'besseli0',
    name: '第1種変形ベッセル関数I0',
    expr: 'besseli0(x)',
    domain: [-MODERATE_BOUND, MODERATE_BOUND],
  },
  {
    id: 'besseli1',
    name: '第1種変形ベッセル関数I1',
    expr: 'besseli1(x)',
    domain: [-MODERATE_BOUND, MODERATE_BOUND],
  },
  {
    id: 'besselk0',
    name: '第2種変形ベッセル関数K0',
    expr: 'besselk0(x)',
    domain: [JUST_ABOVE_ZERO, MODERATE_BOUND],
  },
  {
    id: 'besselk1',
    name: '第2種変形ベッセル関数K1',
    expr: 'besselk1(x)',
    domain: [JUST_ABOVE_ZERO, MODERATE_BOUND],
  },
];

const FUNCTION_TEMPLATES: readonly ObjectTemplate[] = FUNCTION_SPECS.map(
  ({ id, name, expr, domain }) => ({
    id: `function-${id}`,
    label: `${name}：y = ${expr}`,
    kind: 'plane' as const,
    objects: [{ id: 'f', type: 'graph', var: 'x', expr, domain }],
  }),
);

export { FUNCTION_TEMPLATES };
