import type { JsonObject } from './json';

/**
 * 格子の対称性の見本の部品．どの見本も，同じ格子を2組置き，灰色の組はそのまま残して，青の組にだけ，媒介変数を
 * 使う変換を施す．媒介変数が格子の対称操作の値になると，2組が重なる．ここには，格子定数を数で決めた格子(平面と
 * 空間)の組と，媒介変数，補助の線を置く．格子定数を媒介変数にした平面の格子は，`symmetry-sample-scenes.ts`にある．
 */

type Vector2 = readonly [number, number];
type Vector3 = readonly [number, number, number];

const GHOST_STYLE: JsonObject = { color: 'gray' };
const MOVED_STYLE: JsonObject = { color: 'blue' };
const CELL_GHOST_STYLE: JsonObject = { color: 'gray', line: 'dashed' };
const CELL_MOVED_STYLE: JsonObject = { color: 'blue', width: '1pt' };
const GUIDE_STYLE: JsonObject = { color: 'red', line: 'dashed' };

const FULL_TURN = 360;
const HALF_TURN = 180;
/** 回転の角の刻み．30°の倍数(2回・3回・4回・6回の回転)と，その間の角を選べる． */
const ROTATION_STEP = 15;
/** 鏡映や反転を，途中の姿を見せながら施す媒介変数の刻み． */
const FOLD_STEP = 0.05;

interface Stepped {
  id: string;
  value: number;
  range: readonly [number, number];
  step: number;
}

/** 刻みのある媒介変数．刻みを対称操作の値に合わせ，スライダーでちょうどその値にできるようにする． */
function stepped({ id, value, range, step }: Stepped): JsonObject {
  return { id, type: 'parameter', value, range: [...range], step };
}

/** 回転の角θ(0°から360°まで，15°刻み)． */
function rotationAngle(id = 'theta'): JsonObject {
  return stepped({ id, value: 0, range: [0, FULL_TURN], step: ROTATION_STEP });
}

/** 鏡映や反転を施す割合t(0で元のまま，1で操作が済む)． */
function foldAmount(id = 't'): JsonObject {
  return stepped({ id, value: 0, range: [0, 1], step: FOLD_STEP });
}

/** 添字を識別子に使う形(負の数は`m`で書く)． */
function indexName(value: number): string {
  return value < 0 ? `m${-value}` : String(value);
}

function integersBetween(low: number, high: number): number[] {
  return Array.from({ length: high - low + 1 }, (_, index) => low + index);
}

/**
 * 整数の媒介変数kで，値の並びから1つを選ぶ式．k = jのとき，j番目の値になる．`max(0, 1 - |k - j|)`を
 * `abs`だけで書いた山形の関数を重ねるので，kが整数なら，ほかの値は厳密に0倍になる．
 */
function selectBy(parameter: string, values: readonly number[]): string {
  const terms = values.flatMap((value, index) => {
    if (value === 0) {
      return [];
    }
    const distance = `abs(${parameter} - ${index})`;
    return [`(${value})*(1 - ${distance} + abs(1 - ${distance}))/2`];
  });
  return terms.length === 0 ? '0' : terms.join(' + ');
}

interface Look {
  /** 識別子の頭． */
  prefix: string;
  point: JsonObject;
  cell: JsonObject;
  /** 変換の手順．元の組では空である． */
  transform: readonly JsonObject[];
}

const FIXED: Omit<Look, 'transform'> = {
  prefix: 'fixed',
  point: GHOST_STYLE,
  cell: CELL_GHOST_STYLE,
};
const MOVED: Omit<Look, 'transform'> = {
  prefix: 'moved',
  point: MOVED_STYLE,
  cell: CELL_MOVED_STYLE,
};

/** 元の組(灰色)と，変換を施す組(青)．`copy`は，見た目と変換から，格子の1組を作る． */
function twoCopies(
  copy: (look: Look) => JsonObject[],
  transform: readonly JsonObject[],
): JsonObject[] {
  return [...copy({ ...FIXED, transform: [] }), ...copy({ ...MOVED, transform })];
}

function withTransform(object: JsonObject, transform: readonly JsonObject[]): JsonObject {
  return transform.length > 0 ? { ...object, transform: [...transform] } : object;
}

/** 平面の格子点を置く添字の範囲．見える範囲を回しても，格子点が見える範囲を覆う大きさにする． */
const PLANE_INDEX_END = 6;

interface PlaneLattice {
  basis: readonly [Vector2, Vector2];
  /** 単位胞の頂点(数学の座標)．周に沿って並べる． */
  cell: readonly Vector2[];
}

/** 数で決めた平面の格子の1組．格子点と単位胞である．見える範囲の外の格子点は描かれない． */
function planeLatticeCopy({ basis, cell }: PlaneLattice, look: Look): JsonObject[] {
  const [first, second] = basis;
  const indices = integersBetween(-PLANE_INDEX_END, PLANE_INDEX_END);
  const points = indices.flatMap((u) =>
    indices.map((v) =>
      withTransform(
        {
          id: `${look.prefix}_${indexName(u)}_${indexName(v)}`,
          type: 'point',
          at: [u * first[0] + v * second[0], u * first[1] + v * second[1]],
          dot: true,
          style: look.point,
        },
        look.transform,
      ),
    ),
  );
  const polygon = {
    id: `${look.prefix}_cell`,
    type: 'polygon',
    vertices: cell.map((vertex) => [...vertex]),
    style: look.cell,
  };
  return [...points, withTransform(polygon, look.transform)];
}

/** 2点を結ぶ補助の線(回転軸，鏡の直線や面の縁)．両端の点と線分． */
function guide(
  id: string,
  [from, to]: readonly [readonly (number | string)[], readonly (number | string)[]],
): JsonObject[] {
  return [
    { id: `${id}_from`, type: 'point', at: [...from] },
    { id: `${id}_to`, type: 'point', at: [...to] },
    { id, type: 'segment', from: `${id}_from`, to: `${id}_to`, style: GUIDE_STYLE },
  ];
}

export {
  FOLD_STEP,
  FULL_TURN,
  GUIDE_STYLE,
  HALF_TURN,
  ROTATION_STEP,
  foldAmount,
  guide,
  indexName,
  integersBetween,
  planeLatticeCopy,
  rotationAngle,
  selectBy,
  stepped,
  twoCopies,
  withTransform,
};
export type { Look, PlaneLattice, Vector2, Vector3 };
