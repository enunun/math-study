import type { ViewKind } from './draft';
import { FUNCTION_TEMPLATES } from './function-templates';
import type { JsonObject } from './json';
import { NAMED_SCENE_TEMPLATES } from './named-scenes';
import {
  CUBE_VERTICES,
  DODECAHEDRON_VERTICES,
  ICOSAHEDRON_VERTICES,
  OCTAHEDRON_VERTICES,
  TETRAHEDRON_VERTICES,
  polyhedronObjects,
  regularPolygonObjects,
} from './regular-shapes';
import type { Vertex3 } from './regular-shapes';

/**
 * 記事に紐づく「見本」(`samples.ts`)とは別の，部品として組み合わせて使うテンプレート．
 * 正多角形・正多面体・関数のグラフは，今編集している図にオブジェクトとして挿入する．
 * メビウスの帯とコッホ曲線は，それ自体が図なので，新しい図として読み込む(`named-scenes.ts`)．
 */
interface ObjectTemplate {
  id: string;
  label: string;
  kind: ViewKind;
  objects: readonly JsonObject[];
}

interface ObjectTemplateGroup {
  label: string;
  templates: readonly ObjectTemplate[];
}

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

const POLYGON_TEMPLATES: readonly ObjectTemplate[] = POLYGON_SPECS.map(({ n, label }) => ({
  id: `polygon${n}`,
  label,
  kind: 'plane',
  objects: regularPolygonObjects(n),
}));

const POLYHEDRON_SPECS: readonly { id: string; label: string; vertices: readonly Vertex3[] }[] = [
  { id: 'tetrahedron', label: '正4面体', vertices: TETRAHEDRON_VERTICES },
  { id: 'cube', label: '正6面体(立方体)', vertices: CUBE_VERTICES },
  { id: 'octahedron', label: '正8面体', vertices: OCTAHEDRON_VERTICES },
  { id: 'dodecahedron', label: '正12面体', vertices: DODECAHEDRON_VERTICES },
  { id: 'icosahedron', label: '正20面体', vertices: ICOSAHEDRON_VERTICES },
];

const POLYHEDRON_TEMPLATES: readonly ObjectTemplate[] = POLYHEDRON_SPECS.map(
  ({ id, label, vertices }) => ({
    id,
    label,
    kind: 'space',
    objects: polyhedronObjects(vertices),
  }),
);

const OBJECT_TEMPLATE_GROUPS: readonly ObjectTemplateGroup[] = [
  { label: '正多角形', templates: POLYGON_TEMPLATES },
  { label: '正多面体', templates: POLYHEDRON_TEMPLATES },
  { label: '関数のグラフ', templates: FUNCTION_TEMPLATES },
];

const SCENE_TEMPLATES = NAMED_SCENE_TEMPLATES;

export { OBJECT_TEMPLATE_GROUPS, SCENE_TEMPLATES };
export type { ObjectTemplate, ObjectTemplateGroup };
export type { SceneTemplate } from './named-scenes';
