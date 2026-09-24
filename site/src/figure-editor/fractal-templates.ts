import {
  fractal,
  HALF,
  INV_SQRT2,
  placement,
  similarity,
  THIRD,
  UNIT_SEGMENT,
  UNIT_SQUARE,
  UNIT_TRIANGLE,
} from './fractal-parts';
import type { JsonObject } from './json';
import type { ObjectTemplate } from './template-types';

/** 図の大きさ．基本図形(1辺1)を，この倍率に広げて置く． */
const SIZE = 4;
/** ピタゴラスの木の幹(正方形)の1辺と，根元を置く高さ．木の高さは，幹の約4倍になる． */
const TREE_SIZE = 1.2;
const TREE_ROOT_Y = -3;

/** 長さ`SIZE`の線分を，中点を原点にして置く変換． */
const CENTERED_SEGMENT = placement(SIZE, [HALF, 0]);
/** 1辺`SIZE`の正方形を，原点を中心にして置く変換． */
const CENTERED_SQUARE = placement(SIZE, [HALF, HALF]);

const KOCH_ANGLE = 60;
const DRAGON_ANGLE = 45;
const DRAGON_BACK_ANGLE = 135;
/** レヴィのC曲線とドラゴン曲線の深さ．線分の数は，2のこの乗になる． */
const CURVE_DEPTH = 10;
/** レヴィのC曲線の大きさ．曲線は，線分の両端より外へ広がるので，ほかより小さくする． */
const LEVY_SIZE = 3;
/** カントール集合の段の数と，段の間隔(長さ1の線分に対する割合)． */
const CANTOR_DEPTH = 5;
const CANTOR_ROW = 0.1;
/** カントール集合の段の真ん中の高さ(上の段0から下の段-0.5までの中点)． */
const CANTOR_CENTER_Y = -0.25;
const CARPET_DIVISIONS = 3;
const MIDDLE = 1;

/**
 * コッホ曲線(長さ1の線分)．3等分した真ん中を，正三角形の2辺で置き換える．コッホ雪片は，
 * これを正三角形の各辺に置く．
 */
const KOCH_TRANSFORMS: JsonObject[][] = [
  similarity(THIRD, 0, [0, 0]),
  similarity(THIRD, KOCH_ANGLE, [THIRD, 0]),
  similarity(THIRD, -KOCH_ANGLE, [HALF, 'sqrt(3)/6']),
  similarity(THIRD, 0, ['2/3', 0]),
];
const KOCH_DEPTH = 5;
/** コッホ雪片の2辺目，3辺目を作る回転(度)．雪片の中心(原点)のまわりに回す． */
const FULL_TURN_DEGREES = 360;
const SNOWFLAKE_SIDES = 3;
const SNOWFLAKE_TURNS = Array.from(
  { length: SNOWFLAKE_SIDES - 1 },
  (_, index) => ((index + 1) * FULL_TURN_DEGREES) / SNOWFLAKE_SIDES,
);

/** シェルピンスキーのカーペットの8つの変換．3×3に分けた，真ん中以外の正方形へ縮める． */
const CARPET_TRANSFORMS: JsonObject[][] = Array.from(
  { length: CARPET_DIVISIONS * CARPET_DIVISIONS },
  (_, index) => [index % CARPET_DIVISIONS, Math.floor(index / CARPET_DIVISIONS)] as const,
)
  .filter(([column, row]) => column !== MIDDLE || row !== MIDDLE)
  .map(([column, row]) =>
    similarity(THIRD, 0, [column === 0 ? 0 : `${column}/3`, row === 0 ? 0 : `${row}/3`]),
  );

/** ヴィチェックのフラクタルの5つの変換．3×3に分けた，4隅と真ん中の正方形へ縮める． */
const VICSEK_TRANSFORMS: JsonObject[][] = [
  similarity(THIRD, 0, [0, 0]),
  similarity(THIRD, 0, ['2/3', 0]),
  similarity(THIRD, 0, [THIRD, THIRD]),
  similarity(THIRD, 0, [0, '2/3']),
  similarity(THIRD, 0, ['2/3', '2/3']),
];

/**
 * フラクタル．どれも，1辺1の基本図形の上で，正確な分数で反復関数系を書き，全体の変換(`transform`)で
 * 大きさと位置を決める．大きさを変えるときは，`transform`の`scale`だけを直す．
 */
const FRACTAL_TEMPLATES: readonly ObjectTemplate[] = [
  {
    id: 'cantor',
    label: 'カントール集合',
    kind: 'plane',
    // 各段を1行ずつ下にずらし(xだけを縮める)，すべての深さを重ねて，作り方の段を上から並べる．
    objects: [
      fractal({
        base: UNIT_SEGMENT,
        transforms: [
          [{ scale: [THIRD, 1] }, { translate: [0, -CANTOR_ROW] }],
          [{ scale: [THIRD, 1] }, { translate: ['2/3', -CANTOR_ROW] }],
        ],
        depth: CANTOR_DEPTH,
        allDepths: true,
        transform: placement(SIZE, [HALF, CANTOR_CENTER_Y]),
      }),
    ],
  },
  {
    id: 'koch',
    label: 'コッホ曲線',
    kind: 'plane',
    objects: [
      fractal({
        base: UNIT_SEGMENT,
        transforms: KOCH_TRANSFORMS,
        depth: KOCH_DEPTH,
        transform: CENTERED_SEGMENT,
      }),
    ],
  },
  {
    id: 'kochSnowflake',
    label: 'コッホ雪片',
    kind: 'plane',
    // 上の辺(左から右へ，突起が外側)を1つ置き，その像を，中心のまわりに120度ずつ回して3辺にする．
    // 雪片の中心(正三角形の重心)が原点に来るように，辺を置く．大きさを変えても，中心は動かない．
    objects: [
      fractal({
        base: UNIT_SEGMENT,
        transforms: KOCH_TRANSFORMS,
        depth: KOCH_DEPTH,
        transform: placement(SIZE, [HALF, '-sqrt(3)/6']),
      }),
      ...SNOWFLAKE_TURNS.map((angle) => ({
        id: `side${angle}`,
        type: 'image',
        of: 'fractal',
        transform: [{ rotate: angle }],
      })),
    ],
  },
  {
    id: 'sierpinskiTriangle',
    label: 'シェルピンスキーの三角形',
    kind: 'plane',
    objects: [
      fractal({
        base: UNIT_TRIANGLE,
        closed: true,
        transforms: [
          similarity(HALF, 0, [0, 0]),
          similarity(HALF, 0, [HALF, 0]),
          similarity(HALF, 0, ['1/4', 'sqrt(3)/4']),
        ],
        depth: 5,
        transform: placement(SIZE, [HALF, 'sqrt(3)/4']),
      }),
    ],
  },
  {
    id: 'sierpinskiCarpet',
    label: 'シェルピンスキーのカーペット',
    kind: 'plane',
    objects: [
      fractal({
        base: UNIT_SQUARE,
        closed: true,
        transforms: CARPET_TRANSFORMS,
        depth: 3,
        transform: CENTERED_SQUARE,
      }),
    ],
  },
  {
    id: 'vicsek',
    label: 'ヴィチェックのフラクタル',
    kind: 'plane',
    objects: [
      fractal({
        base: UNIT_SQUARE,
        closed: true,
        transforms: VICSEK_TRANSFORMS,
        depth: 3,
        transform: CENTERED_SQUARE,
      }),
    ],
  },
  {
    id: 'levy',
    label: 'レヴィのC曲線',
    kind: 'plane',
    objects: [
      fractal({
        base: UNIT_SEGMENT,
        transforms: [
          similarity(INV_SQRT2, DRAGON_ANGLE, [0, 0]),
          similarity(INV_SQRT2, -DRAGON_ANGLE, [HALF, HALF]),
        ],
        depth: CURVE_DEPTH,
        transform: placement(LEVY_SIZE, [HALF, HALF]),
      }),
    ],
  },
  {
    id: 'dragon',
    label: 'ドラゴン曲線',
    kind: 'plane',
    objects: [
      fractal({
        base: UNIT_SEGMENT,
        transforms: [
          similarity(INV_SQRT2, DRAGON_ANGLE, [0, 0]),
          similarity(INV_SQRT2, DRAGON_BACK_ANGLE, [1, 0]),
        ],
        depth: CURVE_DEPTH,
        transform: CENTERED_SEGMENT,
      }),
    ],
  },
  {
    id: 'pythagorasTree',
    label: 'ピタゴラスの木',
    kind: 'plane',
    // 正方形の上に直角二等辺三角形を置き，その2本の辺に，縮めた正方形を置くことを繰り返す．
    // 途中の段の正方形も木の一部なので，すべての深さを重ねて描く．
    objects: [
      fractal({
        base: UNIT_SQUARE,
        closed: true,
        transforms: [
          similarity(INV_SQRT2, DRAGON_ANGLE, [0, 1]),
          similarity(INV_SQRT2, -DRAGON_ANGLE, [HALF, '3/2']),
        ],
        depth: 8,
        allDepths: true,
        transform: [...placement(TREE_SIZE, [HALF, 0]), { translate: [0, TREE_ROOT_Y] }],
      }),
    ],
  },
];

export { FRACTAL_TEMPLATES };
