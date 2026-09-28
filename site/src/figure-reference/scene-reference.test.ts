import { describe, expect, it } from 'vitest';

import schema from '@/../public/schema/scene.schema.json';

import {
  commonSections,
  expressionEntries,
  objectSections,
  objectTypes,
  sceneFields,
  viewSections,
} from './scene-reference';
import type { FieldEntry, ReferenceSection } from './scene-reference';

const allSections = (): ReferenceSection[] => [
  ...objectSections(),
  ...commonSections(),
  ...viewSections(),
];

const allFields = (): FieldEntry[] => [
  ...sceneFields(),
  ...allSections().flatMap((section) => section.fields),
];

/** スキーマの部品`name`の項目の名前． */
function properties(name: string): string[] {
  const node = Object.entries(schema.$defs).find(([key]) => key === name)?.[1];
  return node !== undefined && 'properties' in node ? Object.keys(node.properties) : [];
}

describe('図のシーンのリファレンス', () => {
  it('スキーマのオブジェクトの種類を，すべて節にする', () => {
    const sections = objectSections();
    expect(sections.map((section) => section.title)).toEqual(objectTypes());
    expect(objectTypes()).toContain('implicit_curve');
    for (const section of sections) {
      expect(section.fields.map((field) => field.name)).toEqual(properties(section.title));
    }
  });

  it('どの種類も，編集画面で追加でき，使える図の種類を示す', () => {
    for (const section of objectSections()) {
      expect(section.note, section.title).toMatch(/(?:平面|空間)(?:と空間)?の図で使える．$/u);
    }
  });

  it('型の中のリンクは，どれもページの中の節を指す', () => {
    const anchors = new Set(allSections().map((section) => `#${section.anchor}`));
    const parts = [
      ...allFields().flatMap((field) => field.type),
      ...allSections().flatMap((section) => section.shape),
    ];
    for (const part of parts) {
      if (part.href !== undefined) {
        expect(anchors, part.href).toContain(part.href);
      }
    }
  });

  it('識別子と種類のほかの項目は，どれも説明を持ち，スキーマの書き方の注記を含まない', () => {
    for (const field of allFields()) {
      const text = field.description.map((part) => part.text).join('');
      expect(text.length, field.name).toBeGreaterThan(0);
      expect(text).not.toContain('スキーマでは表せない');
    }
  });

  it('編集画面の欄の名前を添える', () => {
    const byType = new Map(objectSections().map((section) => [section.title, section]));
    const label = (type: string, name: string): string | undefined =>
      byType.get(type)?.fields.find((field) => field.name === name)?.editorLabel;
    expect(label('surface', 'expr')).toBe('x，y，zの式');
    expect(label('surface', 'bezier')).toBe('制御点の網');
    expect(label('grid', 'z_step')).toBe('z方向の間隔');
    expect(label('implicit_curve', 'values')).toBe('線を引く値');
  });

  it('式の関数は，別名を1つの行にまとめる', () => {
    const { constants, functions } = expressionEntries();
    expect(constants.map((entry) => entry.names)).toEqual([['pi'], ['e']]);
    expect(functions.find((entry) => entry.names.includes('ln'))?.names).toEqual(['log', 'ln']);
  });
});
