import { SCENE_VERSION } from './draft';
import type { SceneDraft } from './draft';
import type { JsonObject } from './json';
import {
  foldAmount,
  guide,
  integersBetween,
  rotationAngle,
  stepped,
  twoCopies,
} from './lattice-symmetry-parts';
import type { Vector3 } from './lattice-symmetry-parts';
import { CELL_EDGES, siteCopy } from './space-lattice-parts';
import type { Site, Sites } from './space-lattice-parts';
import type { SceneTemplate } from './template-types';

/**
 * 空間の格子の，並進を含む対称操作の見本．らせん(回転と，軸に沿う並進を続けた操作)と，映進(鏡映と，鏡に沿う
 * 並進を続けた操作)である．どちらも，底心・体心・面心の格子点を持つ格子にだけ，回転や鏡映と格子の並進に
 * 分けられない形で現れる．並進を含むので，有限個の格子点は，端の1層がはみ出す．重なりは，内側の格子点で見る．
 */

const SPACE_AZIMUTH = -55;
const SPACE_ELEVATION = 22;
const UNIT = '1.5cm';
const HALF = 0.5;
/** 並進の量の刻み(立方体の辺の1/4)． */
const SHIFT_STEP = 0.25;
/** 補助の線(軸や鏡の面の縁)の，はみ出す長さ． */
const GUIDE_MARGIN = 0.3;

function view(): SceneDraft['view'] {
  return { azimuth: SPACE_AZIMUTH, elevation: SPACE_ELEVATION, unit: UNIT };
}

/** 座標の2倍(整数)で書いた格子点．識別子は，座標の2倍を並べる． */
function halfSite(doubled: Vector3): Site {
  const name = doubled.map((value) => (value < 0 ? `m${-value}` : String(value))).join('_');
  const [x, y, z] = doubled;
  return { key: name, at: [x * HALF, y * HALF, z * HALF] };
}

/** 座標の範囲(両端を含む)．端は1/2の倍数にする． */
type Range = readonly [number, number];

interface Box {
  x: Range;
  y: Range;
  z: Range;
  /** 座標の2倍で，格子点かどうか． */
  contains: (doubled: Vector3) => boolean;
}

/** 範囲の中の，1/2の倍数の座標を2倍した整数． */
function doubledBetween([low, high]: Range): number[] {
  return integersBetween(low / HALF, high / HALF);
}

/** 直方体の中の格子点と，原点の立方体(辺の長さ1)の辺． */
function boxSites({ x, y, z, contains }: Box): Sites {
  const sites = doubledBetween(x).flatMap((i) =>
    doubledBetween(y).flatMap((j) =>
      doubledBetween(z).flatMap((k): Site[] => (contains([i, j, k]) ? [halfSite([i, j, k])] : [])),
    ),
  );
  const cornerKey = ([i, j, k]: Vector3): string => halfSite([i / HALF, j / HALF, k / HALF]).key;
  const edges = CELL_EDGES.map(([from, to]) => [cornerKey(from), cornerKey(to)] as const);
  return { sites, edges };
}

/** 偶奇を見る割る数． */
const PARITY = 2;
const isOdd = (value: number): boolean => Math.abs(value) % PARITY === 1;

/** 体心立方格子：座標の2倍が，全部偶数(頂点)か，全部奇数(体心)． */
function bodyCentered(doubled: Vector3): boolean {
  const odd = doubled.filter((value) => isOdd(value)).length;
  return odd === 0 || odd === doubled.length;
}

/** 面心立方格子：座標の2倍のうち，奇数のものが0個(頂点)か2個(面心)． */
function faceCentered(doubled: Vector3): boolean {
  return doubled.filter((value) => isOdd(value)).length % PARITY === 0;
}

/**
 * らせん軸の見本の格子点．軸(x = 0，y = 1/2)を中心とする正方形の柱で，90°の回転で柱は変わらない．z方向の
 * 範囲は-1から1まで．
 */
const SCREW_SITES = boxSites({
  x: [-1, 1],
  y: [-HALF, 1 + HALF],
  z: [-1, 1],
  contains: bodyCentered,
});

const SCREW_AXIS_Y = HALF;
const SCREW_REACH = 1 + GUIDE_MARGIN;

const SCREW_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    'らせん軸．体心立方格子(立方体の頂点と中心に格子点)の青の組を，z軸に平行な直線x = 0，y = 1/2(赤の破線)のまわりに角θだけ回し，軸に沿ってsだけずらす．θ = 180°(s = 0)では，回すだけで重なる(2回軸)．θ = 90°では，回すだけでも，ずらすだけでも重ならないが，s = 1/2ずらすと重なる．これが4回らせん軸4_2である(4回続けると，格子の並進2 × 1/2 = 1になる)．z方向には有限個しか格子点を置いていないので，ずらすと，端の1層ははみ出す．',
  view: view(),
  objects: [
    rotationAngle(),
    stepped({ id: 's', value: 0, range: [0, 1], step: SHIFT_STEP }),
    ...twoCopies(
      (look) => siteCopy(SCREW_SITES, look),
      [
        { rotate: 'theta', axis: [0, 0, 1], center: [0, SCREW_AXIS_Y, 0] },
        { translate: [0, 0, 's'] },
      ],
    ),
    ...guide('axis', [
      [0, SCREW_AXIS_Y, -SCREW_REACH],
      [0, SCREW_AXIS_Y, SCREW_REACH],
    ]),
  ],
};

/**
 * 映進面の見本の格子点．鏡の面x = 1/4について対称な，xが-1/2から1までの範囲に置く．
 */
const GLIDE_SITES = boxSites({
  x: [-HALF, 1],
  y: [-HALF, 1],
  z: [0, 1],
  contains: faceCentered,
});

const GLIDE_PLANE_X = 0.25;

/** 鏡の面x = 1/4の縁(長方形)． */
function glidePlane(): JsonObject[] {
  const low = -HALF - GUIDE_MARGIN;
  const high = 1 + GUIDE_MARGIN;
  const corners: readonly (readonly number[])[] = [
    [GLIDE_PLANE_X, low, low],
    [GLIDE_PLANE_X, high, low],
    [GLIDE_PLANE_X, high, high],
    [GLIDE_PLANE_X, low, high],
  ];
  return corners.flatMap((from, index) =>
    guide(`plane${index + 1}`, [from, corners[(index + 1) % corners.length] ?? from]),
  );
}

const GLIDE_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '映進面．面心立方格子(立方体の頂点と面の中心に格子点)の青の組を，面x = 1/4(赤の破線の長方形)で鏡映し，面に沿ってy方向にsだけずらす．tは鏡映を施す割合で，t = 1で鏡映が済む．鏡映だけ(s = 0)でも，ずらすだけ(t = 0)でも重ならないが，t = 1でs = 1/2ずらすと重なる．これが映進面である．面x = 0なら，鏡映だけで重なる(鏡映面)．y方向には有限個しか格子点を置いていないので，ずらすと，端の1層ははみ出す．',
  view: view(),
  objects: [
    foldAmount(),
    stepped({ id: 's', value: 0, range: [0, 1], step: SHIFT_STEP }),
    ...twoCopies(
      (look) => siteCopy(GLIDE_SITES, look),
      [{ scale: ['cos(t*pi)', 1, 1], center: [GLIDE_PLANE_X, 0, 0] }, { translate: [0, 's', 0] }],
    ),
    ...glidePlane(),
  ],
};

const SPACE_SYMMETRY_OPERATION_SAMPLES: readonly SceneTemplate[] = [
  { id: 'latticeScrew', label: 'らせん軸(体心立方格子)', scene: SCREW_SCENE },
  { id: 'latticeGlidePlane', label: '映進面(面心立方格子)', scene: GLIDE_SCENE },
];

export { SPACE_SYMMETRY_OPERATION_SAMPLES };
