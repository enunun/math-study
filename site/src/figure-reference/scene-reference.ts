import schemaJson from '@/../public/schema/scene.schema.json';
import { createObject, OBJECT_TYPES } from '@/figure-editor/create';
import { emptyDraft } from '@/figure-editor/draft';
import type { ViewKind } from '@/figure-editor/draft';
import { fieldsFor } from '@/figure-editor/fields';
import { stringOf } from '@/figure-editor/json';

import expressionNames from './expression-functions.json';
import {
  asSchemaNode,
  COMMON_DEFS,
  defAnchor,
  inlineParts,
  objectAnchor,
  refName,
  typeParts,
} from './schema-text';
import type { SchemaNode, TextPart } from './schema-text';

/**
 * 図のシーンのリファレンスの中身．JSON Schema(`site/public/schema/scene.schema.json`)から作るので，
 * スキーマに種類や項目を足せば，リファレンスにも載る．スキーマはRustの型と突き合わせてある
 * (`crates/figure/tests/schema_sync.rs`)．編集画面の欄の名前は，図の作成のフォームの定義から取る．
 */

/** シーン全体(トップレベル)と，`$defs`の部品． */
const scene: SchemaNode = asSchemaNode(schemaJson);
const definitions: ReadonlyMap<string, SchemaNode> = new Map(
  Object.entries(schemaJson.$defs).map(([name, node]) => [name, asSchemaNode(node)]),
);

/** 表の1行．項目の名前，編集画面の欄の名前(なければ空)，型，必須か，説明． */
interface FieldEntry {
  name: string;
  editorLabel: string;
  type: TextPart[];
  required: boolean;
  description: TextPart[];
}

/** 種類や部品の1つの節． */
interface ReferenceSection {
  /** ページの中のリンク先(見出しの`id`)． */
  anchor: string;
  /** 見出し．オブジェクトでは`type`の値，共通の部品では名前である． */
  title: string;
  /** 見出しをコードとして示すか(オブジェクトの`type`の値)． */
  codeTitle: boolean;
  /** 見出しに添える語(編集画面の種類の名前や，使える図の種類)． */
  note: string;
  description: TextPart[];
  /** 項目の表．項目を持たない部品(列挙や，数か式など)では空である． */
  fields: FieldEntry[];
  /** 項目を持たない部品の，値の形． */
  shape: TextPart[];
}

function definition(name: string): SchemaNode {
  return definitions.get(name) ?? {};
}

/** 項目の説明．項目自身の説明がなければ，参照先の部品の説明を使う． */
function descriptionOf(node: SchemaNode): string {
  if (node.description !== undefined) {
    return node.description;
  }
  return node.$ref === undefined ? '' : (definition(refName(node.$ref)).description ?? '');
}

function fieldEntries(node: SchemaNode, labels: ReadonlyMap<string, string>): FieldEntry[] {
  const required = new Set(node.required);
  return Object.entries(node.properties ?? {}).map(([name, property]) => ({
    name,
    editorLabel: labels.get(name) ?? '',
    type: typeParts(property),
    required: required.has(name),
    description: inlineParts(name === 'type' ? 'オブジェクトの種類．' : descriptionOf(property)),
  }));
}

/** 追加の一覧の種類(Bézier曲線などを含む)のうち，本当の`type`が`type`であるもの． */
function listedTypesOf(type: string): { label: string; kind: ViewKind; listed: string }[] {
  return OBJECT_TYPES.flatMap(({ type: listed, label, kinds }) =>
    kinds
      .filter((kind) => stringOf(createObject(listed, emptyDraft(kind), kind), 'type') === type)
      .map((kind) => ({ label, kind, listed })),
  );
}

/** 種類の各項目の，編集画面の欄の名前．Bézier曲面のように欄が変わる種類は，すべての欄を集める． */
function editorLabels(type: string): Map<string, string> {
  const labels = new Map<string, string>([['style', 'スタイル']]);
  for (const { kind, listed } of listedTypesOf(type)) {
    for (const spec of fieldsFor(createObject(listed, emptyDraft(kind), kind), kind)) {
      if ('key' in spec && 'label' in spec && !labels.has(spec.key)) {
        labels.set(spec.key, spec.label);
      }
    }
  }
  return labels;
}

const VIEW_KINDS: readonly ViewKind[] = ['plane', 'space'];
const KIND_NAMES: Readonly<Record<ViewKind, string>> = { plane: '平面', space: '空間' };

/** 見出しに添える語．編集画面での種類の名前と，使える図の種類． */
function objectNote(type: string): string {
  const listed = listedTypesOf(type);
  const labels = [...new Set(listed.map(({ label }) => label))];
  const kinds = VIEW_KINDS.filter((kind) => listed.some((entry) => entry.kind === kind));
  const where =
    kinds.length === VIEW_KINDS.length
      ? '平面と空間'
      : kinds.map((kind) => KIND_NAMES[kind]).join('');
  return `${labels.join('，')}．${where}の図で使える．`;
}

/** オブジェクトの種類．スキーマの`object`に並ぶ順である． */
function objectTypes(): string[] {
  return (definition('object').oneOf ?? []).flatMap((choice) =>
    choice.$ref === undefined ? [] : [refName(choice.$ref)],
  );
}

function objectSections(): ReferenceSection[] {
  return objectTypes().map((type) => {
    const node = definition(type);
    return {
      anchor: objectAnchor(type),
      title: type,
      codeTitle: true,
      note: objectNote(type),
      description: inlineParts(node.description ?? ''),
      fields: fieldEntries(node, editorLabels(type)),
      shape: [],
    };
  });
}

function defSection(name: string, title: string): ReferenceSection {
  const node = definition(name);
  const hasFields = node.properties !== undefined;
  return {
    anchor: defAnchor(name),
    title,
    codeTitle: false,
    note: '',
    description: inlineParts(node.description ?? ''),
    fields: hasFields ? fieldEntries(node, new Map()) : [],
    shape: hasFields ? [] : typeParts(node),
  };
}

/** 共通の部品(項目の型として参照するもの)． */
function commonSections(): ReferenceSection[] {
  return COMMON_DEFS.map(([name, title]) => defSection(name, title));
}

/** シーン全体の項目の表と，見える範囲(`view`)の2つの形． */
function sceneFields(): FieldEntry[] {
  return fieldEntries(
    scene,
    new Map([
      ['description', '説明'],
      ['view', '図の種類と範囲'],
    ]),
  );
}

function viewSections(): ReferenceSection[] {
  return [defSection('planeView', '平面の図の範囲'), defSection('spaceView', '空間の図の見る向き')];
}

interface NamedEntry {
  names: string[];
  meaning: string;
}

/** 式で使える定数と関数．エンジンのものと一致することは，`crates/figure/tests/function_list.rs`が確かめる． */
function expressionEntries(): { constants: NamedEntry[]; functions: NamedEntry[] } {
  return {
    constants: expressionNames.constants.map(({ name, meaning }) => ({ names: [name], meaning })),
    functions: expressionNames.functions,
  };
}

export {
  commonSections,
  expressionEntries,
  objectSections,
  objectTypes,
  sceneFields,
  viewSections,
};
export type { FieldEntry, NamedEntry, ReferenceSection };
