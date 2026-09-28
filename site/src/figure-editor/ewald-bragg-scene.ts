import { SCENE_VERSION } from './draft';
import type { SceneDraft } from './draft';
import type { JsonObject } from './json';
import {
  FAMILY_STYLE,
  G_STYLE,
  INDICES,
  LATTICE_STYLE,
  RECIPROCAL_STYLE,
  indexName,
  indexParameter,
  integersBetween,
  latticeAt,
  latticeLines,
  numberText,
  parameter,
} from './reciprocal-parts';
import type { Lattice, Pair } from './reciprocal-parts';
import type { SceneTemplate } from './template-types';

/**
 * Ewald球とBragg反射．格子定数は数で決め，結晶の回転角θ，波長λ，反射の指数(h, k)を媒介変数にする．
 * 左は逆格子とEwald球の断面，右は実格子とその格子面の族で，入射波と回折波の向きを描く．
 */

const A = 1;
const B = 1.25;
const GAMMA_DEGREES = 75;
const DEGREES_PER_HALF_TURN = 180;
const GAMMA = (GAMMA_DEGREES * Math.PI) / DEGREES_PER_HALF_TURN;

/** 逆格子の基本ベクトルの数値(見える範囲に収まる点を選ぶのと，最初の波長を求めるのに使う)． */
const B1 = [1 / A, -Math.cos(GAMMA) / (A * Math.sin(GAMMA))] as const;
const B2 = [0, 1 / (B * Math.sin(GAMMA))] as const;
const RECIPROCAL: Lattice = {
  origin: 0,
  basis: [
    [String(B1[0]), String(B1[1])],
    [String(B2[0]), String(B2[1])],
  ],
};

/** 右の図(実空間)の原点のx座標と，格子の座標の範囲．結晶は，原点のまわりに回してから移す． */
const CRYSTAL_X = 5;
const CRYSTAL_PATCH = 1.3;
const CRYSTAL: Lattice = {
  origin: 0,
  basis: [
    [String(A), '0'],
    [`${B}*cos(${GAMMA_DEGREES}*pi/180)`, `${B}*sin(${GAMMA_DEGREES}*pi/180)`],
  ],
};

const THETA_END = 30;
const LAMBDA_MIN = 0.3;
const LAMBDA_MAX = 1;
const LAMBDA_RANGE = [LAMBDA_MIN, LAMBDA_MAX] as const;
/** 指数の範囲．回折はG・k < 0のときにだけ起きるので，hは0以下にする． */
const REFLECTION_END = 2;
const H_RANGE = [-REFLECTION_END, 0] as const;
const K_RANGE = [-REFLECTION_END, REFLECTION_END] as const;
const INITIAL_H = -1;
const INITIAL_K = 2;
/** 丸めた波長の桁数(小数)． */
const LAMBDA_DIGITS = 4;
const DOUBLE = 2;

/**
 * 最初の波長．θ = 0で，逆格子点G = h b_1 + k b_2がEwald球に載る波長である．球の中心は(-1/λ, 0)なので，
 * |G + k|^2 = |k|^2から，λ = -2 G_x / |G|^2となる．
 */
function initialLambda(): number {
  const x = INITIAL_H * B1[0] + INITIAL_K * B2[0];
  const y = INITIAL_H * B1[1] + INITIAL_K * B2[1];
  return Number(((-DOUBLE * x) / (x * x + y * y)).toFixed(LAMBDA_DIGITS));
}

/** 見える範囲．左の図(逆空間)は，x座標が`RECIPROCAL_X_MAX`までである． */
const X_MIN = -6;
const X_MAX = 7.5;
const Y_END = 3.5;
const RECIPROCAL_X_MAX = 2;
const UNIT = '0.8cm';
/** 下端のラベルを，見える範囲の下端から離す距離． */
const LABEL_MARGIN = 0.2;
/** 回しても見える範囲に収まるかを確かめる角度の刻み(度)． */
const THETA_CHECK_STEP = 5;
/** 逆格子点の印を付ける指数の範囲．ここから，回しても左の図に収まる点だけを選ぶ． */
const POINT_INDEX_END = 6;

function rotated([x, y]: readonly [number, number], degrees: number): [number, number] {
  const radians = (degrees * Math.PI) / DEGREES_PER_HALF_TURN;
  return [
    x * Math.cos(radians) - y * Math.sin(radians),
    x * Math.sin(radians) + y * Math.cos(radians),
  ];
}

/** 逆格子点(h, k)を，範囲のどの角度θで回しても，左の図の見える範囲に収まるか． */
function staysInView(h: number, k: number): boolean {
  const point = [h * B1[0] + k * B2[0], h * B1[1] + k * B2[1]] as const;
  const steps = integersBetween(-THETA_END / THETA_CHECK_STEP, THETA_END / THETA_CHECK_STEP);
  return steps.every((step) => {
    const [x, y] = rotated(point, step * THETA_CHECK_STEP);
    return x >= X_MIN && x <= RECIPROCAL_X_MAX && Math.abs(y) <= Y_END;
  });
}

const ROTATE: JsonObject = { rotate: 'theta' };

function reciprocalPoints(): JsonObject[] {
  const indices = integersBetween(-POINT_INDEX_END, POINT_INDEX_END);
  return indices.flatMap((h) =>
    indices
      .filter((k) => staysInView(h, k))
      .map((k) => ({
        id: `g_${indexName(h)}_${indexName(k)}`,
        type: 'point',
        at: [...latticeAt(RECIPROCAL, [numberText(h), numberText(k)])],
        dot: true,
        style: LATTICE_STYLE,
        transform: [ROTATE],
      })),
  );
}

interface CrystalLinesSpec {
  id: string;
  level: string;
  values: readonly number[];
  style: JsonObject;
}

/** 結晶(右の図)の線．結晶と同じ角θだけ回し，右の図の原点へ移す． */
function crystalLines(spec: CrystalLinesSpec): JsonObject {
  return {
    ...latticeLines({ ...spec, lattice: CRYSTAL, patch: CRYSTAL_PATCH }),
    transform: [ROTATE, { translate: [CRYSTAL_X, 0] }],
  };
}

/** 回した逆格子ベクトルGの成分． */
const G_AT: Pair = (() => {
  const [gx, gy] = [`h*${B1[0]} + k*${B2[0]}`, `h*(${B1[1]}) + k*${B2[1]}`];
  const cos = 'cos(theta*pi/180)';
  const sin = 'sin(theta*pi/180)';
  return [`(${gx})*${cos} - (${gy})*${sin}`, `(${gx})*${sin} + (${gy})*${cos}`];
})();
/** 回折波の向きk' = G + kの長さ．Gが球の中心Cに重なっても値が有限になるよう，小さな数を足す． */
const K_PRIME_LENGTH = 'sqrt((G_x - C_x)^2 + (G_y - C_y)^2 + 0.000000000001)';
const BEAM_LENGTH = 1.8;
const FAMILY_END = Math.ceil((Math.abs(H_RANGE[0]) + Math.abs(K_RANGE[1])) * CRYSTAL_PATCH);

/** ラベル．`[anchor, tex]`は，位置に合わせる部分とTeXの文字列である． */
function label(
  id: string,
  at: JsonObject['at'],
  [anchor, tex]: readonly [string, string],
): JsonObject {
  return { id, type: 'label', at, anchor, tex };
}

/** 左の図．逆格子，Ewald球の断面と，3つのベクトルk，k′，G． */
function ewaldObjects(): JsonObject[] {
  return [
    ...reciprocalPoints(),
    {
      id: 'ewald',
      type: 'curve',
      var: 's',
      expr: ['-1/lambda + cos(s)/lambda', 'sin(s)/lambda'],
      domain: [0, '2*pi'],
      style: { color: 'blue' },
    },
    { id: 'O_star', type: 'point', at: [0, 0], label: 'O^{*}', anchor: 'north west', dot: true },
    { id: 'C', type: 'point', at: ['-1/lambda', 0], label: 'C', anchor: 'north east', dot: true },
    { id: 'G', type: 'point', at: [...G_AT], dot: true, style: { color: 'red' } },
    { id: 'k_vector', type: 'vector', from: 'C', to: 'O_star', style: { width: '1pt' } },
    { id: 'k_prime', type: 'vector', from: 'C', to: 'G', style: RECIPROCAL_STYLE },
    { id: 'G_vector', type: 'vector', from: 'O_star', to: 'G', style: G_STYLE },
    label('k_label', '(C + O_star) / 2', ['north', String.raw`$\boldsymbol{k}$`]),
    label('k_prime_label', '(C + G) / 2', ['south east', String.raw`$\boldsymbol{k}'$`]),
    label('G_label', '(O_star + G) / 2', ['west', String.raw`$\boldsymbol{G}$`]),
    label(
      'laue',
      [X_MIN + LABEL_MARGIN, -Y_END + LABEL_MARGIN],
      ['south west', String.raw`$\boldsymbol{k}'-\boldsymbol{k}=\boldsymbol{G}$`],
    ),
  ];
}

/** 右の図．回した結晶の格子と格子面の族，入射波と回折波． */
function crystalObjects(): JsonObject[] {
  return [
    crystalLines({ id: 'crystal_u', level: 'u', values: INDICES, style: LATTICE_STYLE }),
    crystalLines({ id: 'crystal_v', level: 'v', values: INDICES, style: LATTICE_STYLE }),
    crystalLines({
      id: 'planes',
      level: 'h*u + k*v',
      values: integersBetween(-FAMILY_END, FAMILY_END),
      style: FAMILY_STYLE,
    }),
    { id: 'P', type: 'point', at: [CRYSTAL_X, 0], dot: true },
    { id: 'source', type: 'point', at: [CRYSTAL_X - BEAM_LENGTH, 0] },
    {
      id: 'outgoing',
      type: 'point',
      at: [
        `${CRYSTAL_X} + ${BEAM_LENGTH}*(G_x - C_x)/${K_PRIME_LENGTH}`,
        `${BEAM_LENGTH}*(G_y - C_y)/${K_PRIME_LENGTH}`,
      ],
    },
    { id: 'incident', type: 'vector', from: 'source', to: 'P', style: { width: '1pt' } },
    { id: 'diffracted', type: 'vector', from: 'P', to: 'outgoing', style: RECIPROCAL_STYLE },
    label('incident_label', 'source', ['south west', String.raw`$\boldsymbol{k}$`]),
    label('diffracted_label', 'outgoing', ['west', String.raw`$\boldsymbol{k}'$`]),
    label(
      'bragg',
      [CRYSTAL_X, -Y_END + LABEL_MARGIN],
      ['south', String.raw`$2d\sin\theta_{\mathrm{B}}=\lambda$`],
    ),
  ];
}

const EWALD_BRAGG_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description: `Ewald球とBragg反射．左は逆格子とEwald球の断面，右は実格子と，指数(h, k)の格子面の族である．どちらも結晶の回転角θだけ回る．左で，逆格子点G = h b_1 + k b_2がEwald球に載ると，k' − k = Gが成り立ち，右では，入射波kと回折波k'が格子面について鏡映の位置になる(Braggの条件2d sin θ_B = λ)．Gは格子面に垂直で長さ1/dなので，2つの条件は同じことを言っている．格子定数はa = ${A}，b = ${B}，γ = ${GAMMA_DEGREES}°で，逆格子の点の間隔も同じ単位で描く．スライダーでθやλを動かすと，Gが球面を横切る所でだけ，右の回折波が格子面で鏡のように反射した向きになる．`,
  view: { x: [X_MIN, X_MAX], y: [-Y_END, Y_END], unit: { x: UNIT, y: UNIT } },
  objects: [
    parameter('theta', 0, [-THETA_END, THETA_END]),
    parameter('lambda', initialLambda(), LAMBDA_RANGE),
    indexParameter('h', INITIAL_H, H_RANGE),
    indexParameter('k', INITIAL_K, K_RANGE),
    ...ewaldObjects(),
    ...crystalObjects(),
  ],
};

const EWALD_BRAGG_SAMPLE: SceneTemplate = {
  id: 'ewaldBragg',
  label: 'Ewald球とBragg反射',
  scene: EWALD_BRAGG_SCENE,
};

export { EWALD_BRAGG_SAMPLE };
