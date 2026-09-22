import { evaluateConstant } from './constant-expr';
import { uniqueId } from './draft';
import type { SceneDraft } from './draft';
import { stringOf, withField } from './json';
import type { Json, JsonObject } from './json';

/**
 * 編集の補助として示す要素(格子，座標軸，ワイヤーフレーム)の見た目．実際の出力(プレビューやTikZ)は，
 * 元のオブジェクトのスタイルのままで，この見た目は，編集画面だけに使う．
 */
const REFERENCE_STYLE: JsonObject = { line: 'dotted', color: 'gray', width: '0.4pt' };
/** 補助のスタイルで描く，オブジェクトの種類．どちらも，パラメータを編集するときの目印になる． */
const REFERENCE_TYPES = new Set(['axis', 'grid']);

const PAIR = 2;
const TRIPLE = 3;
/** 曲面のワイヤーフレームの，1方向あたりの本数．境界は曲面自身の輪郭にすでにあるので，内側だけを引く． */
const WIREFRAME_LINES = 4;
const FIRST = 0;
const SECOND = 1;

function otherOf(at: number): number {
  return at === FIRST ? SECOND : FIRST;
}

/** 格子と座標軸を，補助のスタイルで描く． */
function withReferenceStyle(object: JsonObject): JsonObject {
  return withField(object, 'style', REFERENCE_STYLE);
}

/** 配列の要素のうち，文字列のものだけを残す．文字列でない要素があれば，そこで短くなる分だけ検査に引っかかる． */
function stringsOf(value: Json | undefined): string[] {
  return Array.isArray(value)
    ? value.filter((item): item is string => typeof item === 'string')
    : [];
}

/** 数か，`pi`・`e`だけを使った定数の式なら，その数．媒介変数を使った式は，値が決まらない． */
function numberOf(value: Json | undefined): number | undefined {
  if (typeof value === 'number') {
    return value;
  }
  return typeof value === 'string' ? evaluateConstant(value) : undefined;
}

/** 2つの数(か定数の式)の配列なら，その組．そうでなければ`undefined`． */
function numericPair(value: Json | undefined): [number, number] | undefined {
  if (!Array.isArray(value) || value.length !== PAIR) {
    return undefined;
  }
  const low = numberOf(value[0]);
  const high = numberOf(value[1]);
  return low === undefined || high === undefined ? undefined : [low, high];
}

/**
 * 曲面の`domain`が，どちらの変数も数(か定数の式)の範囲で書かれていれば，その組．
 * 媒介変数を使った範囲は，値が決まらないので`undefined`．
 */
function numericDomain(value: Json | undefined): [[number, number], [number, number]] | undefined {
  if (!Array.isArray(value) || value.length !== PAIR) {
    return undefined;
  }
  const first = numericPair(value[0]);
  const second = numericPair(value[1]);
  return first === undefined || second === undefined ? undefined : [first, second];
}

/** 式の中の変数`name`を，数値`value`に置き換える．変数名は，単語の境界で区切って探す． */
function substitute(expr: string, name: string, value: number): string {
  const pattern = new RegExp(`\\b${name}\\b`, 'gu');
  return expr.replace(pattern, `(${value})`);
}

/** `parametricLines`の入力． */
interface ParametricLinesInput {
  vars: readonly [string, string];
  expr: readonly string[];
  domain: readonly [[number, number], [number, number]];
  /** 一定にする変数(0か1)．もう一方が，曲線の媒介変数になる． */
  fixedAt: number;
  objects: readonly JsonObject[];
}

/**
 * 曲面の，変数`fixedAt`を一定にした断面の曲線を，`WIREFRAME_LINES`本作る．
 * 一定にする値は，範囲の内側を等間隔に分けた点で，両端(曲面自身の輪郭と重なる)は含めない．
 */
function parametricLines({
  vars,
  expr,
  domain,
  fixedAt,
  objects,
}: ParametricLinesInput): JsonObject[] {
  const freeAt = otherOf(fixedAt);
  const [low, high] = domain[fixedAt];
  const lines: JsonObject[] = [];
  for (let step = 1; step <= WIREFRAME_LINES; step += 1) {
    const t = step / (WIREFRAME_LINES + 1);
    const value = low + t * (high - low);
    lines.push({
      id: uniqueId('wire', [...objects, ...lines]),
      type: 'curve',
      var: vars[freeAt],
      expr: expr.map((one) => substitute(one, vars[fixedAt], value)),
      domain: domain[freeAt],
      style: REFERENCE_STYLE,
    });
  }
  return lines;
}

/**
 * 式で書いた曲面の，u一定・v一定の断面を，補助の曲線にする．
 * 範囲がどちらも数で書かれているときだけ作る．式で書かれていれば，値を補間できない．
 */
function formulaWireframe(surface: JsonObject, objects: readonly JsonObject[]): JsonObject[] {
  const vars = stringsOf(surface.vars);
  const expr = stringsOf(surface.expr);
  const domain = numericDomain(surface.domain);
  if (vars.length !== PAIR || expr.length !== TRIPLE || domain === undefined) {
    return [];
  }
  const names: [string, string] = [vars[0] ?? '', vars[1] ?? ''];
  const first = parametricLines({ vars: names, expr, domain, fixedAt: FIRST, objects });
  const second = parametricLines({
    vars: names,
    expr,
    domain,
    fixedAt: SECOND,
    objects: [...objects, ...first],
  });
  return [...first, ...second];
}

/** 値が，3個の数の配列(点の座標)か． */
function isTriple(value: Json): value is [number, number, number] {
  return (
    Array.isArray(value) && value.length === TRIPLE && value.every((one) => typeof one === 'number')
  );
}

/** 配列の要素が，すべて点の座標なら，その並び．1つでも式で書かれていれば，補間できないので`undefined`． */
function triplesOf(row: Json): [number, number, number][] | undefined {
  if (!Array.isArray(row)) {
    return undefined;
  }
  const triples: [number, number, number][] = [];
  for (const value of row) {
    if (!isTriple(value)) {
      return undefined;
    }
    triples.push(value);
  }
  return triples;
}

/** ベジエ曲面の制御点の網が，すべて数で書かれていれば，その並び．1つでも式で書かれていれば`undefined`． */
function bezierNet(surface: JsonObject): [number, number, number][][] | undefined {
  if (!Array.isArray(surface.bezier)) {
    return undefined;
  }
  const net: [number, number, number][][] = [];
  for (const row of surface.bezier) {
    const triples = triplesOf(row);
    if (triples === undefined) {
      return undefined;
    }
    net.push(triples);
  }
  return net;
}

/** 網の各点を，補助の点(印も名前もない)にする． */
function netPoints(
  net: readonly (readonly [number, number, number])[][],
  objects: readonly JsonObject[],
): { ids: string[][]; points: JsonObject[] } {
  const points: JsonObject[] = [];
  const ids = net.map((row) =>
    row.map((at) => {
      const id = uniqueId('wire', [...objects, ...points]);
      points.push({ id, type: 'point', at: [...at] });
      return id;
    }),
  );
  return { ids, points };
}

/** 隣り合う識別子どうしを結ぶ，補助の線分． */
function links(row: readonly string[], objects: readonly JsonObject[]): JsonObject[] {
  const segments: JsonObject[] = [];
  for (let at = 1; at < row.length; at += 1) {
    segments.push({
      id: uniqueId('wire', [...objects, ...segments]),
      type: 'segment',
      from: row[at - 1],
      to: row[at],
      style: REFERENCE_STYLE,
    });
  }
  return segments;
}

/** 行ごとに`links`を求める．前の行が作った線分も，次の行の識別子と重ならないよう，順に積み上げる． */
function allLinks(
  rows: readonly (readonly string[])[],
  objects: readonly JsonObject[],
): JsonObject[] {
  const segments: JsonObject[] = [];
  for (const row of rows) {
    segments.push(...links(row, [...objects, ...segments]));
  }
  return segments;
}

/** ベジエ曲面の制御点の網を，補助の折れ線(行ごと，列ごと)にする． */
function bezierWireframe(surface: JsonObject, objects: readonly JsonObject[]): JsonObject[] {
  const net = bezierNet(surface);
  if (net === undefined || net.length === 0) {
    return [];
  }
  const { ids, points } = netPoints(net, objects);
  const rows = allLinks(ids, [...objects, ...points]);
  const columnCount = ids[0]?.length ?? 0;
  const columns = allLinks(
    Array.from({ length: columnCount }, (_, column) => ids.map((row) => row[column] ?? '')),
    [...objects, ...points, ...rows],
  );
  return [...points, ...rows, ...columns];
}

/** 曲面オブジェクトを，補助のワイヤーフレームにする．曲面でなければ，何も作らない． */
function wireframeOf(object: JsonObject, objects: readonly JsonObject[]): JsonObject[] {
  if (stringOf(object, 'type') !== 'surface') {
    return [];
  }
  return 'bezier' in object ? bezierWireframe(object, objects) : formulaWireframe(object, objects);
}

interface EditSceneOptions {
  /** 曲面ごとに，範囲を目で追える補助の線を足すか． */
  wireframe: boolean;
}

/**
 * プレビューを，編集の補助を加えた図にする．格子と座標軸は，補助のスタイル(灰色の点線)で描き，
 * 実際に描かれるオブジェクトと見分けやすくする．`wireframe`が真なら，曲面ごとに，範囲を目で追える
 * 補助の線(式の曲面はu・v一定の断面，ベジエ曲面は制御点の網)を足す．どちらも，見た目だけを変える．
 * 実際の出力(プレビューやTikZ)は，元のシーンのままである．
 */
function buildEditScene(draft: SceneDraft, options: EditSceneOptions): SceneDraft {
  const objects = draft.objects.map((object) =>
    REFERENCE_TYPES.has(stringOf(object, 'type')) ? withReferenceStyle(object) : object,
  );
  if (!options.wireframe) {
    return { ...draft, objects };
  }
  const extra: JsonObject[] = [];
  for (const object of objects) {
    extra.push(...wireframeOf(object, [...objects, ...extra]));
  }
  return { ...draft, objects: [...objects, ...extra] };
}

export { buildEditScene };
export type { EditSceneOptions };
