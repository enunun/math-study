import type { JsonObject } from './json';
import { indexName, integersBetween, withTransform } from './lattice-symmetry-parts';
import type { Look, Vector3 } from './lattice-symmetry-parts';

/**
 * 空間の格子の対称性の見本の部品．空間の図には見える範囲がないので，格子点は有限個しか置けない．原点を動かさない
 * 操作(点群の元)を見せる組は，原点を中心とする球の中の格子点を置く．球は原点を通るどの回転と反転でも変わらないので，
 * 格子の対称操作なら，有限個の格子点どうしもちょうど重なる．
 */

type Basis = readonly [Vector3, Vector3, Vector3];

/** 格子点の名前と，数学の座標． */
interface Site {
  key: string;
  at: Vector3;
}

interface Sites {
  sites: readonly Site[];
  /** 単位胞の辺．両端の格子点の`key`． */
  edges: readonly (readonly [string, string])[];
}

/** 格子点の組を，1組の図にする．辺は格子点を結ぶので，変換は点にだけ施せば，辺もついてくる． */
function siteCopy({ sites, edges }: Sites, look: Look): JsonObject[] {
  const id = (key: string): string => `${look.prefix}_${key}`;
  const points = sites.map(({ key, at }) =>
    withTransform(
      { id: id(key), type: 'point', at: [...at], dot: true, style: look.point },
      look.transform,
    ),
  );
  const segments = edges.map(([from, to], index) => ({
    id: `${look.prefix}_edge${index + 1}`,
    type: 'segment',
    from: id(from),
    to: id(to),
    style: look.cell,
  }));
  return [...points, ...segments];
}

/** 単位胞の頂点の添字(各成分が0か1)と，1成分だけが違う頂点の組(辺)． */
const CORNERS: readonly Vector3[] = [0, 1].flatMap((i) =>
  [0, 1].flatMap((j) => [0, 1].map((k): Vector3 => [i, j, k])),
);
const CELL_EDGES: readonly (readonly [Vector3, Vector3])[] = CORNERS.flatMap((from) =>
  CORNERS.filter(
    (to) =>
      from.join(',') < to.join(',') &&
      from.filter((value, axis) => value !== to[axis]).length === 1,
  ).map((to) => [from, to] as const),
);

function keyOf(indices: readonly number[]): string {
  return indices.map((value) => indexName(value)).join('_');
}

function combine([first, second, third]: Basis, [i, j, k]: Vector3): Vector3 {
  const [x, y, z] = first.map(
    (value, axis) => i * value + j * (second[axis] ?? 0) + k * (third[axis] ?? 0),
  );
  return [x ?? 0, y ?? 0, z ?? 0];
}

/** 添字を探す範囲．見本の格子では，球の中の格子点の添字は，この範囲に収まる． */
const INDEX_END = 4;
/** 球の半径に足す余裕．単位胞の頂点が，丸めの誤差で球の外に出ないようにする． */
const RADIUS_MARGIN = 1e-9;

/**
 * 原点を中心とする球の中の格子点と，原点の単位胞の辺．球の半径は，単位胞の頂点が全部入る最小の大きさである．
 */
function ballSites(basis: Basis): Sites {
  const radius = Math.max(...CORNERS.map((corner) => Math.hypot(...combine(basis, corner))));
  const indices = integersBetween(-INDEX_END, INDEX_END);
  const sites = indices.flatMap((i) =>
    indices.flatMap((j) =>
      indices.flatMap((k): Site[] => {
        const at = combine(basis, [i, j, k]);
        return Math.hypot(...at) <= radius + RADIUS_MARGIN ? [{ key: keyOf([i, j, k]), at }] : [];
      }),
    ),
  );
  const edges = CELL_EDGES.map(([from, to]) => [keyOf(from), keyOf(to)] as const);
  return { sites, edges };
}

export { CELL_EDGES, CORNERS, ballSites, keyOf, siteCopy };
export type { Basis, Site, Sites };
