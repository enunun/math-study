import { describe, expect, it } from 'vitest';

import { allowedIn, changeKind, createObject, typesFor } from './create';
import {
  NO_SELECTION,
  addObject,
  emptyDraft,
  indexOfId,
  moveObject,
  parseDraft,
  removeObject,
  replaceObject,
  stringifyDraft,
  uniqueId,
  viewKind,
} from './draft';
import { fieldsFor, listNames } from './fields';
import {
  boundFromText,
  boundToText,
  commitField,
  parseItem,
  readJsonField,
  setItem,
} from './values';

const OBJECTS = [
  { id: 'a', type: 'point' },
  { id: 'b', type: 'point' },
  { id: 'c', type: 'point' },
];

describe('図のシーンの操作', () => {
  it('新しい図は，空で，平面か空間かが，viewの形で決まる', () => {
    expect(emptyDraft().objects).toEqual([]);
    expect(viewKind(emptyDraft('plane'))).toBe('plane');
    expect(viewKind(emptyDraft('space'))).toBe('space');
  });

  it('書き出して読み直すと，同じ図になる', () => {
    const draft = addObject(emptyDraft(), { id: 'p', type: 'point', at: [1, 2] });
    const parsed = parseDraft(stringifyDraft(draft));
    expect(parsed).toEqual({ ok: true, draft });
  });

  it('JSONでない文字列と，オブジェクトでないJSONは，理由つきで断る', () => {
    expect(parseDraft('{')).toMatchObject({ ok: false });
    expect(parseDraft('[1]')).toMatchObject({ ok: false });
    expect(parseDraft('{"objects": [1]}')).toMatchObject({ ok: false });
  });

  it('項目がなくても，読める．版は，既定の版になる', () => {
    const parsed = parseDraft('{}');
    expect(parsed.ok && parsed.draft.version).toBe('0.1.0');
  });

  it('識別子は，重ならない連番になる', () => {
    expect(uniqueId('point', [])).toBe('point1');
    expect(
      uniqueId('point', [
        { id: 'point1', type: 'point' },
        { id: 'point2', type: 'point' },
      ]),
    ).toBe('point3');
    expect(uniqueId('', [])).toBe('object1');
  });

  it('オブジェクトを，足し，置き換え，取り除く', () => {
    let draft = emptyDraft();
    for (const object of OBJECTS) {
      draft = addObject(draft, object);
    }
    draft = replaceObject(draft, 1, { id: 'B', type: 'point' });
    draft = removeObject(draft, 0);
    expect(draft.objects.map((object) => object.id)).toEqual(['B', 'c']);
  });

  it('オブジェクトを動かす．端を越えるときは，そのままにする', () => {
    const draft = { ...emptyDraft(), objects: OBJECTS };
    expect(moveObject(draft, 0, 1).objects.map((object) => object.id)).toEqual(['b', 'a', 'c']);
    expect(moveObject(draft, 2, -2).objects.map((object) => object.id)).toEqual(['c', 'a', 'b']);
    expect(moveObject(draft, 0, -1)).toBe(draft);
    expect(moveObject(draft, 2, 1)).toBe(draft);
  });
});

describe('新しいオブジェクト', () => {
  it('使える種類は，図の種類で変わる', () => {
    const plane = typesFor('plane').map((entry) => entry.type);
    const space = typesFor('space').map((entry) => entry.type);
    expect(plane).toContain('graph');
    expect(plane).not.toContain('sphere');
    expect(space).toContain('sphere');
    expect(space).not.toContain('graph');
    expect(allowedIn({ id: 's', type: 'surface' }, 'space')).toBe(true);
    expect(allowedIn({ id: 'g', type: 'graph' }, 'space')).toBe(false);
  });

  it('参照する項目は，追加済みのオブジェクトの，先頭のものを使う', () => {
    let draft = emptyDraft();
    draft = addObject(draft, createObject('point', draft, 'plane'));
    draft = addObject(draft, createObject('point', draft, 'plane'));
    const segment = createObject('segment', draft, 'plane');
    expect(segment).toMatchObject({ from: 'point1', to: 'point2' });
    expect(segment.id).toBe('segment1');
  });

  it('曲面の項目は，式かベジエ曲面かで変わる', () => {
    const draft = emptyDraft('space');
    const formula = fieldsFor(createObject('surface', draft, 'space'), 'space');
    const bezier = fieldsFor(createObject('bezier', draft, 'space'), 'space');
    expect(formula.some((field) => 'key' in field && field.key === 'expr')).toBe(true);
    expect(bezier.some((field) => 'key' in field && field.key === 'bezier')).toBe(true);
  });
});

describe('図の種類の切り替え', () => {
  it('切り替えると，viewが初期値になり，使えないオブジェクトが消える', () => {
    let draft = emptyDraft('space');
    draft = addObject(draft, createObject('sphere', draft, 'space'));
    draft = addObject(draft, createObject('axis', draft, 'space'));
    const plane = changeKind(draft, 'plane');
    expect(viewKind(plane)).toBe('plane');
    expect(plane.objects.map((object) => object.type)).toEqual(['axis']);
  });

  it('同じ種類への切り替えは，何も変えない', () => {
    const draft = emptyDraft('plane');
    expect(changeKind(draft, 'plane')).toBe(draft);
  });
});

describe('入力欄の値', () => {
  it('数の形の文字列は数に，それ以外は式(文字列)にする', () => {
    expect(boundFromText(' 1.5 ')).toBe(1.5);
    expect(boundFromText('-2')).toBe(-2);
    expect(boundFromText('2*pi')).toBe('2*pi');
    expect(boundFromText('')).toBe('');
  });

  it('数か式を，入力欄の文字列にする', () => {
    expect(boundToText(3)).toBe('3');
    expect(boundToText('pi')).toBe('pi');
    expect(boundToText([])).toBe('');
  });

  it('配列の1つの要素だけを書き換える', () => {
    expect(setItem({ list: [1, 2], index: 1, item: 5, count: 3 })).toEqual([1, 5, '']);
    expect(setItem({ list: [], index: 0, item: 'x', count: 2 })).toEqual(['x', '']);
  });
});

describe('選択の操作', () => {
  it('識別子から添字を探す．なければ選択なしを返す', () => {
    const draft = addObject(addObject(emptyDraft(), { id: 'a', type: 'point' }), {
      id: 'b',
      type: 'point',
    });
    expect(indexOfId(draft, 'b')).toBe(1);
    expect(indexOfId(draft, 'c')).toBe(NO_SELECTION);
  });
});

describe('入力欄の読み書き', () => {
  it('種類に応じて，文字列を値にする', () => {
    expect(parseItem('number', '1.5')).toBe(1.5);
    expect(parseItem('bound', 'pi')).toBe('pi');
    expect(parseItem('text', '1')).toBe('1');
  });

  it('省略できる項目が空なら取り除き，省略できない項目は空のまま持つ', () => {
    const object = { id: 'a', label: 'A' };
    expect(commitField({ object, key: 'label', value: '', optional: true })).toEqual({ id: 'a' });
    expect(commitField({ object, key: 'label', value: '' })).toEqual({ id: 'a', label: '' });
    expect(commitField({ object, key: 'label', value: ['', ''], optional: true })).toEqual({
      id: 'a',
    });
  });

  it('JSONを直接書く欄は，読めない間は直前の値を保つ', () => {
    expect(readJsonField('[1, 2]', false)).toEqual({ kind: 'set', value: [1, 2] });
    expect(readJsonField('  ', true)).toEqual({ kind: 'set', value: undefined });
    expect(readJsonField('', false)).toEqual({ kind: 'keep', message: '' });
    expect(readJsonField('[1,', false)).toMatchObject({ kind: 'keep' });
  });

  it('並びの入力欄の名前は，空間の座標なら成分の名前になる', () => {
    const point = fieldsFor({ type: 'sphere' }, 'space').find((spec) => spec.kind === 'list');
    expect(point?.kind === 'list' && listNames(point, 'space')).toEqual(['x', 'y', 'z']);
    expect(point?.kind === 'list' && listNames(point, 'plane')).toEqual([
      '1番目',
      '2番目',
      '3番目',
    ]);
  });
});
