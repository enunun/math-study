import { SCENE_VERSION } from './draft';
import type { SceneDraft } from './draft';
import type { JsonObject } from './json';
import {
  A_ANCHORS,
  A_NAMES,
  B_ANCHORS,
  B_NAMES,
  GAMMA,
  REAL,
  RECIPROCAL,
  RECIPROCAL_STYLE,
  REAL_STYLE,
  basisVectors,
  integersBetween,
  latticePoints,
  originPoint,
  parameter,
} from './reciprocal-parts';
import type { Lattice } from './reciprocal-parts';
import type { SceneTemplate } from './template-types';

/**
 * Wigner-Seitz胞と第1Brillouinゾーン．左の実格子には，原点のWigner-Seitz胞を，右の逆格子には，同じ作り方の
 * 胞(第1Brillouinゾーン)を描く．どちらも`wigner_seitz`オブジェクトに基本ベクトルの式を渡すだけで，
 * 格子定数のスライダーに従う．
 */

const X_END = 7.6;
const Y_END = 3.2;
const UNIT = '0.8cm';
/** 格子点の印を付ける添字の範囲．見える範囲の外の点は描かれない． */
const INDEX_END = 3;
const LENGTH_MIN = 1;
const LENGTH_MAX = 1.6;
const A_VALUE = 1.2;
const B_VALUE = 1.4;
const GAMMA_VALUE = 70;
const GAMMA_MIN = 60;
const GAMMA_MAX = 120;
const FILL_OPACITY = 0.2;

/** 格子の胞．基本ベクトルの式と，格子の原点を中心にする． */
function cell(id: string, { origin, basis }: Lattice, color: string): JsonObject {
  return {
    id,
    type: 'wigner_seitz',
    basis: basis.map((vector) => [...vector]),
    center: [origin, 0],
    fill: { color, opacity: FILL_OPACITY },
    style: { color, width: '1pt' },
  };
}

const BRILLOUIN_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    'Wigner-Seitz胞と第1Brillouinゾーン．Wigner-Seitz胞は，格子点のうち原点にいちばん近い点の集まりで，原点と近くの格子点を結ぶ線分の垂直二等分線で囲まれる．左は実格子のWigner-Seitz胞，右は逆格子のWigner-Seitz胞で，これが第1Brillouinゾーンである．どちらも面積は単位胞と等しく，格子を隙間なく埋める．正方格子(a = b，γ = 90°)では正方形，六方格子(a = b，γ = 60°か120°)では正六角形，一般の格子では六角形になる．',
  view: { x: [-X_END, X_END], y: [-Y_END, Y_END], unit: { x: UNIT, y: UNIT } },
  objects: [
    parameter('a', A_VALUE, [LENGTH_MIN, LENGTH_MAX]),
    parameter('b', B_VALUE, [LENGTH_MIN, LENGTH_MAX]),
    parameter(GAMMA, GAMMA_VALUE, [GAMMA_MIN, GAMMA_MAX]),
    cell('wigner_seitz', REAL, 'blue'),
    ...latticePoints('real', REAL, integersBetween(-INDEX_END, INDEX_END)),
    originPoint('O', REAL, 'O'),
    ...basisVectors({
      ids: ['O', 'A1', 'A2'],
      lattice: REAL,
      names: A_NAMES,
      style: REAL_STYLE,
      anchors: A_ANCHORS,
    }),
    cell('brillouin', RECIPROCAL, 'red'),
    ...latticePoints('reciprocal', RECIPROCAL, integersBetween(-INDEX_END, INDEX_END)),
    originPoint('O_star', RECIPROCAL, 'O^{*}'),
    ...basisVectors({
      ids: ['O_star', 'B1', 'B2'],
      lattice: RECIPROCAL,
      names: B_NAMES,
      style: RECIPROCAL_STYLE,
      anchors: B_ANCHORS,
    }),
  ],
};

const BRILLOUIN_SAMPLE: SceneTemplate = {
  id: 'brillouinZone',
  label: 'Wigner-Seitz胞とBrillouinゾーン',
  scene: BRILLOUIN_SCENE,
};

export { BRILLOUIN_SAMPLE };
