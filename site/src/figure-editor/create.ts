import defaults from './defaults.json';
import { defaultView, uniqueId, viewKind } from './draft';
import type { SceneDraft, ViewKind } from './draft';
import { stringOf, toJsonObject } from './json';
import type { Json, JsonObject } from './json';

/** 追加できるオブジェクトの種類．`kinds`は，使える図の種類である． */
interface ObjectType {
  type: string;
  label: string;
  kinds: readonly ViewKind[];
}

const BOTH: readonly ViewKind[] = ['plane', 'space'];
const PLANE: readonly ViewKind[] = ['plane'];
const SPACE: readonly ViewKind[] = ['space'];

/** ベジエ曲面は，`type`が`surface`で，`bezier`の項目を持つ．追加の一覧では，別の種類として扱う． */
const BEZIER = 'bezier';
/** ベジエ曲線は，`type`が`curve`で，`bezier`の項目を持つ．追加の一覧では，別の種類として扱う． */
const BEZIER_CURVE = 'bezierCurve';
/** スプライン曲線は，`type`が`curve`で，`spline`の項目を持つ．追加の一覧では，別の種類として扱う． */
const SPLINE_CURVE = 'splineCurve';
/** 頂点で書く多角形は，`type`が`polygon`で，`vertices`の項目を持つ．正多角形とは別の種類として扱う． */
const VERTEX_POLYGON = 'vertexPolygon';
const PAIR = 2;

/** 変換(`transform`)を持てる種類．像(`image`)の元にできる． */
const TRANSFORMABLE: readonly string[] = [
  'point',
  'segment',
  'vector',
  'graph',
  'curve',
  'polygon',
  'fractal',
  'grid',
  'surface',
  'complex',
  'polyhedron',
];

const OBJECT_TYPES: readonly ObjectType[] = [
  { type: 'axis', label: '座標軸', kinds: BOTH },
  { type: 'label', label: 'ラベル', kinds: BOTH },
  { type: 'parameter', label: '媒介変数', kinds: BOTH },
  { type: 'graph', label: '関数のグラフ', kinds: PLANE },
  { type: 'curve', label: '曲線(式)', kinds: BOTH },
  { type: BEZIER_CURVE, label: '曲線(ベジエ)', kinds: BOTH },
  { type: SPLINE_CURVE, label: '曲線(スプライン)', kinds: BOTH },
  { type: 'tangent_line', label: '接線', kinds: PLANE },
  { type: 'grid', label: '格子', kinds: BOTH },
  { type: 'point', label: '点', kinds: BOTH },
  { type: 'vector', label: 'ベクトル', kinds: BOTH },
  { type: 'segment', label: '線分', kinds: BOTH },
  { type: 'polygon', label: '正多角形', kinds: PLANE },
  { type: VERTEX_POLYGON, label: '多角形(頂点)', kinds: PLANE },
  { type: 'region', label: '領域', kinds: PLANE },
  { type: 'fractal', label: 'フラクタル', kinds: PLANE },
  { type: 'taylor', label: 'テイラー展開', kinds: PLANE },
  { type: 'function', label: '関数', kinds: BOTH },
  { type: 'map', label: '写像', kinds: BOTH },
  { type: 'image', label: '像(変換した図形)', kinds: BOTH },
  { type: 'sphere', label: '球', kinds: SPACE },
  { type: 'surface', label: '曲面(式)', kinds: SPACE },
  { type: BEZIER, label: '曲面(ベジエ)', kinds: SPACE },
  { type: 'cut', label: '曲面の切り口', kinds: SPACE },
  { type: 'intersection', label: '曲面の交線', kinds: SPACE },
  { type: 'tangent_plane', label: '接平面', kinds: SPACE },
  { type: 'polyhedron', label: '正多面体', kinds: SPACE },
  { type: 'complex', label: '複体', kinds: SPACE },
];

function typesFor(kind: ViewKind): readonly ObjectType[] {
  return OBJECT_TYPES.filter((entry) => entry.kinds.includes(kind));
}

/** 追加の一覧での種類の名前．ベジエ曲面・曲線とスプライン曲線は，`surface`・`curve`ではなく，
 * 別の名前である． */
function listedType(object: JsonObject): string {
  const type = stringOf(object, 'type');
  if (type === 'surface' && BEZIER in object) {
    return BEZIER;
  }
  if (type === 'polygon') {
    return 'vertices' in object ? VERTEX_POLYGON : type;
  }
  if (type !== 'curve') {
    return type;
  }
  if (BEZIER in object) {
    return BEZIER_CURVE;
  }
  return 'spline' in object ? SPLINE_CURVE : type;
}

/** そのオブジェクトが，この図で使えるか． */
function allowedIn(object: JsonObject, kind: ViewKind): boolean {
  return typesFor(kind).some((entry) => entry.type === listedType(object));
}

/**
 * 識別子の元になる名前．ベジエ曲面は`surface`，ベジエ曲線とスプライン曲線は`curve`，頂点で書く多角形は
 * `polygon`から始める．関数と写像は，式の中で呼ぶ名前になるので，短い`f`と`F`から始める．
 */
const STEMS: Readonly<Record<string, string>> = {
  [BEZIER]: 'surface',
  [BEZIER_CURVE]: 'curve',
  [SPLINE_CURVE]: 'curve',
  [VERTEX_POLYGON]: 'polygon',
  function: 'f',
  map: 'F',
};

function stemOf(type: string): string {
  return STEMS[type] ?? type;
}

/** 追加済みのオブジェクトのうち，指定の種類の識別子を，並びの順に返す． */
function idsOfType(draft: SceneDraft, type: string): string[] {
  return draft.objects
    .filter((object) => object.type === type)
    .map((object) => stringOf(object, 'id'));
}

/** 参照する項目に，追加済みのオブジェクトの，先頭のものを入れる． */
function withReferences(content: JsonObject, draft: SceneDraft): JsonObject {
  const points = idsOfType(draft, 'point');
  const surfaces = idsOfType(draft, 'surface');
  const graphs = idsOfType(draft, 'graph');
  const curves = idsOfType(draft, 'curve');
  const transformable = draft.objects
    .filter((object) => TRANSFORMABLE.includes(stringOf(object, 'type')))
    .map((object) => stringOf(object, 'id'));
  const references: Record<string, Record<string, Json>> = {
    vector: { from: points[0] ?? '', to: points[1] ?? '' },
    segment: { from: points[0] ?? '', to: points[1] ?? '' },
    region: { between: graphs.slice(0, PAIR) },
    cut: { surface: surfaces[0] ?? '' },
    intersection: { surfaces: surfaces.slice(0, PAIR) },
    tangent_line: { of: graphs[0] ?? curves[0] ?? '' },
    tangent_plane: { of: surfaces[0] ?? '' },
    taylor: { of: graphs[0] ?? '' },
    image: { of: transformable.at(-1) ?? '' },
  };
  return { ...content, ...references[stringOf(content, 'type')] };
}

/** 新しいオブジェクト．識別子は，種類の名前に連番を付けたもので，ほかと重ならない． */
function createObject(type: string, draft: SceneDraft, kind: ViewKind): JsonObject {
  const byType = toJsonObject(toJsonObject(defaults.objects)[type]);
  const content = withReferences(toJsonObject(byType[kind]), draft);
  return { id: uniqueId(stemOf(type), draft.objects), ...content };
}

/**
 * 図の種類を切り替える．`view`は，その種類の初期値になり，その種類で使えないオブジェクトは取り除く．
 * 同じ種類のままなら，何も変えない．
 */
function changeKind(draft: SceneDraft, kind: ViewKind): SceneDraft {
  if (viewKind(draft) === kind) {
    return draft;
  }
  return {
    ...draft,
    view: defaultView(kind),
    objects: draft.objects.filter((object) => allowedIn(object, kind)),
  };
}

export { allowedIn, changeKind, createObject, listedType, OBJECT_TYPES, TRANSFORMABLE, typesFor };
export type { ObjectType };
