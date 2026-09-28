import { SCENE_VERSION } from './draft';
import type { SceneDraft } from './draft';
import type { JsonObject } from './json';
import type { SceneTemplate } from './template-types';

/**
 * Ewald球の見本．逆格子の格子点の間隔を1とし，波長λも同じ単位で書く．入射波の波数ベクトルkは
 * x軸の向きで，長さは1/λである．Ewald球は，中心C = O* − k，半径|k|の球(断面では円)で，原点O*を通る．
 * 結晶の回転角θ(度，z軸のまわり)と波長λは，範囲のある媒介変数なので，スライダーで動かせる．
 */

/** 回転角と波長の範囲と，最初の値．θ = 0，λ = 0.4(|k| = 2.5)では，逆格子点G = (−1, 2)が球面に載る． */
const THETA_END = 30;
const LAMBDA_MIN = 0.3;
const LAMBDA_MAX = 1;
const INITIAL_LAMBDA = 0.4;
/** 回折波を描く逆格子点G(の，回す前の座標)． */
const G_H = -1;
const G_K = 2;

const PARAMETERS: readonly JsonObject[] = [
  { id: 'theta', type: 'parameter', value: 0, range: [-THETA_END, THETA_END] },
  { id: 'lambda', type: 'parameter', value: INITIAL_LAMBDA, range: [LAMBDA_MIN, LAMBDA_MAX] },
];

const COS = 'cos(theta*pi/180)';
const SIN = 'sin(theta*pi/180)';

/** 逆格子点G = (h, k)を，原点のまわりにθ回した点の，x座標とy座標の式． */
const G_AT = [`${G_H}*${COS} - ${G_K}*${SIN}`, `${G_H}*${SIN} + ${G_K}*${COS}`];

/** 空間の図では，どの線も隠さない(Ewald球は透けた球として描く)． */
const SEEN: JsonObject = { hidden: 'visible' };
const LATTICE_STYLE: JsonObject = { color: 'gray', ...SEEN };
const G_STYLE: JsonObject = { color: 'red', ...SEEN };
const K_STYLE: JsonObject = { width: '1pt', ...SEEN };
const K_PRIME_STYLE: JsonObject = { color: 'red', width: '1pt', ...SEEN };
const RECIPROCAL_STYLE: JsonObject = { color: 'green', width: '1pt', ...SEEN };
const EWALD_STYLE: JsonObject = { color: 'blue', ...SEEN };

/** 断面の図の見える範囲．Ewald球の断面(円)と，その左の逆格子点までが入る． */
const PLANE_X_MIN = -6;
const PLANE_X_MAX = 1.5;
const PLANE_Y_END = 3.5;
/** 格子点の印を付ける範囲(回す前の添字)．断面の図では，回しても見える範囲に収まる点だけに付ける． */
const H_MIN = -6;
const H_MAX = 1;
const K_END = 3;
/** 半回転の角度(度)．度をラジアンに直すのに使う． */
const DEGREES_PER_HALF_TURN = 180;
/** 回しても見える範囲に収まるかを確かめる角度の刻み(度)． */
const THETA_CHECK_STEP = 5;
/** 断面の図の格子の線を引く範囲．回しても見える範囲を覆う大きさにする(はみ出た所は切れる)． */
const GRID_END = 8;
/** 条件の式のラベルを，見える範囲の左下の角から離す距離． */
const LABEL_MARGIN = 0.2;

function integersBetween(low: number, high: number): number[] {
  return Array.from({ length: high - low + 1 }, (_, index) => low + index);
}

/** 点(h, k)をθ(度)回しても，断面の図の見える範囲に収まるか． */
function staysInView(h: number, k: number): boolean {
  const steps = integersBetween(-THETA_END / THETA_CHECK_STEP, THETA_END / THETA_CHECK_STEP);
  return steps.every((step) => {
    const radians = (step * THETA_CHECK_STEP * Math.PI) / DEGREES_PER_HALF_TURN;
    const x = h * Math.cos(radians) - k * Math.sin(radians);
    const y = h * Math.sin(radians) + k * Math.cos(radians);
    return x >= PLANE_X_MIN && x <= PLANE_X_MAX && Math.abs(y) <= PLANE_Y_END;
  });
}

/** 添字を識別子に使う形(負の数は`m`で書く)． */
function indexName(value: number): string {
  return value < 0 ? `m${-value}` : String(value);
}

/** 逆格子点の印．`space`なら，z = 0の層の点で，z軸のまわりに回す． */
function latticePoints(space: boolean): JsonObject[] {
  const rotate: JsonObject = space ? { rotate: 'theta', axis: [0, 0, 1] } : { rotate: 'theta' };
  return integersBetween(H_MIN, H_MAX).flatMap((h) =>
    integersBetween(-K_END, K_END)
      .filter((k) => space || staysInView(h, k))
      .map((k) => ({
        id: `g_${indexName(h)}_${indexName(k)}`,
        type: 'point',
        at: space ? [h, k, 0] : [h, k],
        dot: true,
        style: LATTICE_STYLE,
        transform: [rotate],
      })),
  );
}

const K_TEX = String.raw`$\boldsymbol{k}$`;
const K_PRIME_TEX = String.raw`$\boldsymbol{k}'$`;
const G_TEX = String.raw`$\boldsymbol{G}$`;
const CONDITION_TEX = String.raw`$\boldsymbol{k}'-\boldsymbol{k}=\boldsymbol{G}$`;

/** 原点O*，球の中心C，逆格子点Gと，3つのベクトルk，k′，G．`space`なら，z座標0を足す． */
function vectorObjects(space: boolean): JsonObject[] {
  const z = space ? [0] : [];
  return [
    { id: 'O', type: 'point', at: [0, 0, ...z], label: 'O^{*}', anchor: 'north west', dot: true },
    {
      id: 'C',
      type: 'point',
      at: ['-1/lambda', 0, ...z],
      label: 'C',
      anchor: 'north east',
      dot: true,
    },
    { id: 'G', type: 'point', at: [...G_AT, ...z], dot: true, style: G_STYLE },
    { id: 'k', type: 'vector', from: 'C', to: 'O', style: K_STYLE },
    { id: 'k_prime', type: 'vector', from: 'C', to: 'G', style: K_PRIME_STYLE },
    { id: 'reciprocal', type: 'vector', from: 'O', to: 'G', style: RECIPROCAL_STYLE },
  ];
}

const DESCRIPTION_BODY =
  '結晶に波数ベクトルkの波(X線など)を当てると，回折波の波数ベクトルk′は，k′ − kが逆格子ベクトルGに等しい向きにだけ強く出る(Laueの条件で，Braggの条件2d sin θ = λと同じ)．|k′| = |k|なので，k′の終点は，中心C = O* − k，半径1/λの球(Ewald球)の上にある．したがって，回折が起きるのは，逆格子点がEwald球の上に載るときである．逆格子点Gの1つ1つは結晶の格子面の1組に当たり，Gは格子面に垂直で，長さは面の間隔dの逆数である．逆格子を描けば，どの格子面がどの向きに回折を起こすかが，点と球の位置関係として読める．スライダーで結晶の回転角θや波長λを変えると，格子点が球面を横切るときにだけ条件が満たされることがわかる．';

const EWALD_SECTION_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description: `Ewald球の断面(原点O*を通る逆格子の面)．${DESCRIPTION_BODY}`,
  view: {
    x: [PLANE_X_MIN, PLANE_X_MAX],
    y: [-PLANE_Y_END, PLANE_Y_END],
    unit: { x: '1cm', y: '1cm' },
  },
  objects: [
    ...PARAMETERS,
    {
      id: 'lattice',
      type: 'grid',
      x_step: 1,
      y_step: 1,
      x_range: [-GRID_END, GRID_END],
      y_range: [-GRID_END, GRID_END],
      style: { color: 'gray' },
      transform: [{ rotate: 'theta' }],
    },
    ...latticePoints(false),
    {
      id: 'ewald',
      type: 'curve',
      var: 's',
      expr: ['-1/lambda + cos(s)/lambda', 'sin(s)/lambda'],
      domain: [0, '2*pi'],
      style: { color: 'blue' },
    },
    ...vectorObjects(false),
    { id: 'k_label', type: 'label', at: '(C + O) / 2', anchor: 'north', tex: K_TEX },
    {
      id: 'k_prime_label',
      type: 'label',
      at: '(C + G) / 2',
      anchor: 'south east',
      tex: K_PRIME_TEX,
    },
    { id: 'reciprocal_label', type: 'label', at: '(O + G) / 2', anchor: 'west', tex: G_TEX },
    {
      id: 'condition',
      type: 'label',
      at: [PLANE_X_MIN + LABEL_MARGIN, -PLANE_Y_END + LABEL_MARGIN],
      anchor: 'south west',
      tex: CONDITION_TEX,
    },
  ],
};

/** 空間の図の見る向き．入射波が左から右へ進むように見る． */
const SPACE_AZIMUTH = -60;
const SPACE_ELEVATION = 25;

const EWALD_SPHERE_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description: `Ewald球．逆格子のうち，原点O*を通る層(z = 0)だけを描き，その層と球が交わる円を太く描く．${DESCRIPTION_BODY}`,
  view: { azimuth: SPACE_AZIMUTH, elevation: SPACE_ELEVATION, unit: '1cm' },
  objects: [
    ...PARAMETERS,
    {
      id: 'layer',
      type: 'grid',
      x_step: 1,
      y_step: 1,
      x_range: [H_MIN, H_MAX],
      y_range: [-K_END, K_END],
      style: LATTICE_STYLE,
      transform: [{ rotate: 'theta', axis: [0, 0, 1] }],
    },
    ...latticePoints(true),
    {
      id: 'ewald',
      type: 'surface',
      vars: ['a', 'b'],
      expr: ['-1/lambda + sin(a)*cos(b)/lambda', 'sin(a)*sin(b)/lambda', 'cos(a)/lambda'],
      domain: [
        [0, 'pi'],
        ['-pi', 'pi'],
      ],
      boundary: true,
      wireframe: { color: 'blue', hidden: 'none' },
      wireframe_step: ['pi/4', 'pi/4'],
      style: EWALD_STYLE,
    },
    {
      id: 'layer_circle',
      type: 'curve',
      var: 's',
      expr: ['-1/lambda + cos(s)/lambda', 'sin(s)/lambda', '0'],
      domain: [0, '2*pi'],
      style: { ...EWALD_STYLE, width: '1pt' },
    },
    ...vectorObjects(true),
    { id: 'k_label', type: 'label', at: ['-1/(2*lambda)', 0, 0], anchor: 'north', tex: K_TEX },
    {
      id: 'k_prime_label',
      type: 'label',
      at: ['(C_x + G_x) / 2', '(C_y + G_y) / 2', 0],
      anchor: 'south',
      tex: K_PRIME_TEX,
    },
    {
      id: 'reciprocal_label',
      type: 'label',
      at: ['G_x / 2', 'G_y / 2', 0],
      anchor: 'west',
      tex: G_TEX,
    },
    {
      id: 'condition',
      type: 'label',
      at: ['-1/lambda', 0, `1/lambda + ${LABEL_MARGIN}`],
      anchor: 'south',
      tex: CONDITION_TEX,
    },
  ],
};

const EWALD_SECTION_SAMPLE: SceneTemplate = {
  id: 'ewaldSection',
  label: 'Ewald球(断面)',
  scene: EWALD_SECTION_SCENE,
};
const EWALD_SPHERE_SAMPLE: SceneTemplate = {
  id: 'ewaldSphere',
  label: 'Ewald球',
  scene: EWALD_SPHERE_SCENE,
};

export { EWALD_SECTION_SAMPLE, EWALD_SPHERE_SAMPLE };
