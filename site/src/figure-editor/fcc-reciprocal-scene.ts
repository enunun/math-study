import { SCENE_VERSION } from './draft';
import type { SceneDraft } from './draft';
import type { JsonObject } from './json';
import { parameter } from './reciprocal-parts';
import {
  EDGE_STYLE,
  RECIPROCAL_LOOK,
  REAL_LOOK,
  SCREEN_RIGHT,
  SPACE_AZIMUTH,
  SPACE_ELEVATION,
  basisVectors,
  toRight,
} from './reciprocal-space-scenes';
import type { Triple } from './reciprocal-space-scenes';
import type { SceneTemplate } from './template-types';

/**
 * 面心立方格子の逆格子．立方体の辺aの面心立方格子の基本ベクトルは，a_1 = (0, a/2, a/2)などで，その逆格子の
 * 基本ベクトルb_1 = (−1, 1, 1)/aなどは，辺2/aの立方体の中心から3つの頂点へ向かう．つまり逆格子は体心立方
 * 格子である．
 */

type Offset = readonly [number, number, number];

const HALF = 'a/2';
const A_VALUE = 2;
const A_MIN = 1.5;
const A_MAX = 3;
const A_RANGE = [A_MIN, A_MAX] as const;
const MIDDLE = 0.5;
/** 逆格子の立方体と実格子の立方体の間の，画面の右向きの隙間． */
const GAP = 0.8;
/**
 * 逆格子の立方体の中心O*．実格子の立方体(画面の右向きの幅は，x成分とy成分の和のa倍)の右に，逆格子の
 * 立方体(中心から同じ和の1/a倍)を，隙間を空けて並べ，高さは実格子の立方体の中心にそろえる．
 */
const SCREEN_WIDTH = SCREEN_RIGHT[0] + SCREEN_RIGHT[1];
const CENTER: Triple = [...toRight(`${SCREEN_WIDTH}*(a + 1/a) + ${GAP}`), HALF];

const FCC_TIPS: readonly Triple[] = [
  ['0', HALF, HALF],
  [HALF, '0', HALF],
  [HALF, HALF, '0'],
];
const BCC_TIPS: readonly Offset[] = [
  [-1, 1, 1],
  [1, -1, 1],
  [1, 1, -1],
];
const FACE_CENTERS: readonly Offset[] = [
  [MIDDLE, MIDDLE, 0],
  [MIDDLE, MIDDLE, 1],
  [MIDDLE, 0, MIDDLE],
  [MIDDLE, 1, MIDDLE],
  [0, MIDDLE, MIDDLE],
  [1, MIDDLE, MIDDLE],
];
const BODY_CENTER: readonly Offset[] = [[MIDDLE, MIDDLE, MIDDLE]];

/** 立方体の頂点の添字(ijkの各桁が0か1)． */
const VERTICES: readonly string[] = ['0', '1'].flatMap((i) =>
  ['0', '1'].flatMap((j) => ['0', '1'].map((k) => `${i}${j}${k}`)),
);

interface Cube {
  prefix: string;
  /** 頂点000の座標の式(3つ)． */
  corner: readonly string[];
  /** 辺の長さの式． */
  side: string;
  /** 頂点のほかの格子点(辺を1とした位置)． */
  centers: readonly Offset[];
  style: JsonObject;
}

function pointAt(cube: Cube, id: string, offset: Offset): JsonObject {
  return {
    id,
    type: 'point',
    at: cube.corner.map((value, axis) => `${value} + ${offset[axis] ?? 0}*${cube.side}`),
    dot: true,
    style: cube.style,
  };
}

/** 立方体の頂点と心の格子点と，12本の辺(添字が1桁だけ違う頂点の組)． */
function cubeObjects(cube: Cube): JsonObject[] {
  const { prefix } = cube;
  const vertices = VERTICES.map((name) => {
    const [i = 0, j = 0, k = 0] = [...name].map(Number);
    return pointAt(cube, `${prefix}_${name}`, [i, j, k]);
  });
  const centers = cube.centers.map((offset, index) =>
    pointAt(cube, `${prefix}_center${index + 1}`, offset),
  );
  const pairs = VERTICES.flatMap((from) =>
    VERTICES.filter(
      (to) => from < to && [...from].filter((digit, n) => digit !== to[n]).length === 1,
    ).map((to) => [from, to] as const),
  );
  const edges = pairs.map(([from, to], index) => ({
    id: `${prefix}_edge${index + 1}`,
    type: 'segment',
    from: `${prefix}_${from}`,
    to: `${prefix}_${to}`,
    style: EDGE_STYLE,
  }));
  return [...vertices, ...centers, ...edges];
}

const FCC_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '面心立方格子の逆格子．左は辺aの面心立方格子で，基本ベクトルa_1 = (0, a/2, a/2)，a_2 = (a/2, 0, a/2)，a_3 = (a/2, a/2, 0)を描く．右は，b_1 = (a_2 × a_3)/Vなどで作った逆格子で，b_1 = (−1, 1, 1)/a，b_2 = (1, −1, 1)/a，b_3 = (1, 1, −1)/aは，辺2/aの立方体の中心O*から頂点へ向かう．逆格子は体心立方格子になる(逆に，体心立方格子の逆格子は面心立方格子である)．スライダーでaを大きくすると，逆格子の立方体は小さくなる．',
  view: { azimuth: SPACE_AZIMUTH, elevation: SPACE_ELEVATION, unit: '1cm' },
  objects: [
    parameter('a', A_VALUE, A_RANGE),
    ...cubeObjects({
      prefix: 'fcc',
      corner: ['0', '0', '0'],
      side: 'a',
      centers: FACE_CENTERS,
      style: { color: 'blue' },
    }),
    { id: 'O', type: 'point', at: [0, 0, 0], label: 'O', anchor: 'north east' },
    ...FCC_TIPS.map((at, index) => ({ id: `A${index + 1}`, type: 'point', at: [...at] })),
    ...basisVectors('O', ['A1', 'A2', 'A3'], REAL_LOOK),
    ...cubeObjects({
      prefix: 'bcc',
      corner: CENTER.map((center) => `${center} - 1/a`),
      side: '2/a',
      centers: BODY_CENTER,
      style: { color: 'red' },
    }),
    { id: 'O_star', type: 'point', at: [...CENTER], label: 'O^{*}', anchor: 'north east' },
    ...BCC_TIPS.map((tip, index) => ({
      id: `B${index + 1}`,
      type: 'point',
      at: CENTER.map((center, axis) => `${center} + ${tip[axis] ?? 0}/a`),
    })),
    ...basisVectors('O_star', ['B1', 'B2', 'B3'], RECIPROCAL_LOOK),
  ],
};

const RECIPROCAL_FCC_SAMPLE: SceneTemplate = {
  id: 'reciprocalFcc',
  label: '面心立方格子の逆格子',
  scene: FCC_SCENE,
};

export { RECIPROCAL_FCC_SAMPLE };
