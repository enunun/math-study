/** 選択肢の入力欄の，1つの選択肢(書き出す値と，表示する名前)． */
type Option = readonly [value: string, label: string];

const ANCHORS: readonly Option[] = [
  ['center', '中央'],
  ['north', '上'],
  ['south', '下'],
  ['east', '右'],
  ['west', '左'],
  ['north east', '右上'],
  ['north west', '左上'],
  ['south east', '右下'],
  ['south west', '左下'],
];

const ARROWS: readonly Option[] = [
  ['stealth', '矢じりあり'],
  ['none', '矢じりなし'],
];

const PLANE_DIRECTIONS: readonly Option[] = [
  ['x', 'x軸'],
  ['y', 'y軸'],
];

const SPACE_DIRECTIONS: readonly Option[] = [...PLANE_DIRECTIONS, ['z', 'z軸']];

const SOLIDS: readonly Option[] = [
  ['tetrahedron', '正四面体'],
  ['cube', '正六面体(立方体)'],
  ['octahedron', '正八面体'],
  ['dodecahedron', '正十二面体'],
  ['icosahedron', '正二十面体'],
];

/** ベクトル場の矢印の長さの決め方． */
const ARROW_LENGTHS: readonly Option[] = [
  ['scaled', '場の値×倍率'],
  ['normalized', 'そろえる(向きだけ)'],
  ['clamped', '場の値×倍率(上限で切る)'],
];

/** ベクトル場の矢印を置く位置． */
const PIVOTS: readonly Option[] = [
  ['middle', '中点を格子点に'],
  ['tail', '根元を格子点に'],
];

/** 流線を伸ばす向き． */
const LINE_DIRECTIONS: readonly Option[] = [
  ['both', '両方'],
  ['forward', '場の向き'],
  ['backward', '場と逆向き'],
];

/** 値を色で表す図の，値と色の対応． */
const COLORMAPS: readonly Option[] = [
  ['viridis', 'viridis(紫から黄)'],
  ['gray', '黒から白'],
  ['gray_inverse', '白から黒'],
  ['coolwarm', '青・白・赤(正負)'],
];

/** 値を色で表す図の，値の目盛． */
const VALUE_SCALES: readonly Option[] = [
  ['linear', 'そのまま'],
  ['log', '対数'],
];

/** 複素関数の色塗りの，明るさの陰影． */
const SHADINGS: readonly Option[] = [
  ['modulus', '絶対値の縞'],
  ['none', 'なし(色相だけ)'],
];

export {
  ANCHORS,
  COLORMAPS,
  SHADINGS,
  VALUE_SCALES,
  ARROWS,
  ARROW_LENGTHS,
  LINE_DIRECTIONS,
  PIVOTS,
  PLANE_DIRECTIONS,
  SOLIDS,
  SPACE_DIRECTIONS,
};
export type { Option };
