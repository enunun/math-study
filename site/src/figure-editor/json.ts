/** JSONの値． */
type Json = null | boolean | number | string | Json[] | JsonObject;

/** JSONのオブジェクト．型の別名では，自分自身を含む定義が循環するので，インターフェースで書く． */
interface JsonObject {
  [key: string]: Json;
}

const PRIMITIVES = new Set(['boolean', 'number', 'string']);

function isJsonObject(value: unknown): value is JsonObject {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

/** 値がJSONで書ける形か．関数や`undefined`，循環する値は，JSONではない． */
function isJson(value: unknown): value is Json {
  if (value === null || PRIMITIVES.has(typeof value)) {
    return true;
  }
  if (Array.isArray(value)) {
    return value.every((item) => isJson(item));
  }
  return isJsonObject(value) && Object.values(value).every((item) => isJson(item));
}

/** 文字列を読んでJSONの値にする．読めないときは，理由を返す． */
function parseJson(text: string): { ok: true; value: Json } | { ok: false; message: string } {
  try {
    const value: unknown = JSON.parse(text);
    return isJson(value) ? { ok: true, value } : { ok: false, message: 'JSONの値ではない．' };
  } catch (error) {
    return { ok: false, message: error instanceof Error ? error.message : String(error) };
  }
}

/** オブジェクトの項目のうち，文字列のもの．なければ空文字列． */
function stringOf(object: JsonObject, key: string): string {
  const value = object[key];
  return typeof value === 'string' ? value : '';
}

/** オブジェクトの項目のうち，配列のもの．なければ空の配列． */
function arrayOf(object: JsonObject, key: string): Json[] {
  const value = object[key];
  return Array.isArray(value) ? value : [];
}

/** オブジェクトの項目のうち，オブジェクトのもの．なければ空のオブジェクト． */
function objectOf(object: JsonObject, key: string): JsonObject {
  const value = object[key];
  return isJsonObject(value) ? value : {};
}

/** 値がオブジェクトなら，そのオブジェクト．そうでなければ，空のオブジェクト． */
function toJsonObject(value: unknown): JsonObject {
  return isJsonObject(value) ? value : {};
}

/** 項目を書き換えた，新しいオブジェクト．値が`undefined`なら，その項目を取り除く． */
function withField(object: JsonObject, key: string, value: Json | undefined): JsonObject {
  const { [key]: _removed, ...rest } = object;
  return value === undefined ? rest : { ...rest, [key]: value };
}

export { arrayOf, isJson, isJsonObject, objectOf, parseJson, stringOf, toJsonObject, withField };
export type { Json, JsonObject };
