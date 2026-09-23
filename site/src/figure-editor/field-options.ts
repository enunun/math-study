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
  ['tetrahedron', '正4面体'],
  ['cube', '正6面体(立方体)'],
  ['octahedron', '正8面体'],
  ['dodecahedron', '正12面体'],
  ['icosahedron', '正20面体'],
];

export { ANCHORS, ARROWS, PLANE_DIRECTIONS, SOLIDS, SPACE_DIRECTIONS };
export type { Option };
