import { BEZIER_SPLINE_TEMPLATES, BEZIER_SURFACE_TEMPLATES } from './bezier-templates';
import { CONIC_TEMPLATES, PLANE_CURVE_TEMPLATES, SPACE_CURVE_TEMPLATES } from './curve-templates';
import { FRACTAL_TEMPLATES } from './fractal-templates';
import { FUNCTION_TEMPLATES } from './function-templates';
import { OTHER_SURFACE_TEMPLATES, QUADRIC_TEMPLATES } from './surface-templates';
import type { ObjectTemplate, ObjectTemplateGroup } from './template-types';

const TRIANGLE_SIDES = 3;
const SQUARE_SIDES = 4;
const PENTAGON_SIDES = 5;
const HEXAGON_SIDES = 6;
const OCTAGON_SIDES = 8;
const DECAGON_SIDES = 10;
const DODECAGON_SIDES = 12;

const POLYGON_SPECS: readonly { n: number; label: string }[] = [
  { n: TRIANGLE_SIDES, label: '正三角形' },
  { n: SQUARE_SIDES, label: '正方形' },
  { n: PENTAGON_SIDES, label: '正五角形' },
  { n: HEXAGON_SIDES, label: '正六角形' },
  { n: OCTAGON_SIDES, label: '正八角形' },
  { n: DECAGON_SIDES, label: '正十角形' },
  { n: DODECAGON_SIDES, label: '正十二角形' },
];

/** 正多角形の既定の半径(中心から頂点までの距離)． */
const POLYGON_RADIUS = 2;

/**
 * 正多角形は，辺の数と中心と半径だけを持つ`polygon`で書く．頂点はエンジンが決め，底辺を水平に置く．
 * 向きを変えるときは，変換(`transform`)の回転を足す．
 */
const POLYGON_TEMPLATES: readonly ObjectTemplate[] = POLYGON_SPECS.map(({ n, label }) => ({
  id: `polygon${n}`,
  label,
  kind: 'plane',
  objects: [{ id: 'p', type: 'polygon', sides: n, center: [0, 0], radius: POLYGON_RADIUS }],
}));

/** 正多面体の既定の半径(中心から頂点までの距離)． */
const POLYHEDRON_RADIUS = 2;

const POLYHEDRON_SPECS: readonly { solid: string; label: string }[] = [
  { solid: 'tetrahedron', label: '正4面体' },
  { solid: 'cube', label: '正6面体(立方体)' },
  { solid: 'octahedron', label: '正8面体' },
  { solid: 'dodecahedron', label: '正12面体' },
  { solid: 'icosahedron', label: '正20面体' },
];

/** 正多面体は，種類と中心と半径だけを持つ`polyhedron`で書く．頂点と面はエンジンが決める． */
const POLYHEDRON_TEMPLATES: readonly ObjectTemplate[] = POLYHEDRON_SPECS.map(
  ({ solid, label }) => ({
    id: solid,
    label,
    kind: 'space',
    objects: [{ id: 'p', type: 'polyhedron', solid, center: [0, 0, 0], radius: POLYHEDRON_RADIUS }],
  }),
);

/**
 * 部品のテンプレートのまとまり．今編集している図に，オブジェクトとして挿入する．ツールバーは，
 * 今の図の種類(平面・空間)で使えるものだけを出すので，平面と空間の両方を持つまとまりもある．
 * 図全体を置き換える図のテンプレート(`scene-templates.ts`)は，座標軸だけの出発点である．
 * そのまま使える完成した図は，見本(`samples.ts`)に置く．
 */
const OBJECT_TEMPLATE_GROUPS: readonly ObjectTemplateGroup[] = [
  { label: '正多角形', templates: POLYGON_TEMPLATES },
  { label: '2次曲線', templates: CONIC_TEMPLATES },
  { label: '平面曲線', templates: PLANE_CURVE_TEMPLATES },
  { label: '関数のグラフ', templates: FUNCTION_TEMPLATES },
  { label: 'フラクタル', templates: FRACTAL_TEMPLATES },
  { label: 'ベジエ曲線・スプライン曲線', templates: BEZIER_SPLINE_TEMPLATES },
  { label: '空間曲線', templates: SPACE_CURVE_TEMPLATES },
  { label: '正多面体', templates: POLYHEDRON_TEMPLATES },
  { label: '2次曲面', templates: QUADRIC_TEMPLATES },
  { label: 'ベジエ曲面', templates: BEZIER_SURFACE_TEMPLATES },
  { label: 'いろいろな曲面', templates: OTHER_SURFACE_TEMPLATES },
];

export { OBJECT_TEMPLATE_GROUPS };
export { SCENE_TEMPLATES } from './scene-templates';
export type { ObjectTemplate, ObjectTemplateGroup, SceneTemplate } from './template-types';
