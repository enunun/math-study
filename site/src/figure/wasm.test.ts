import { readFile } from 'node:fs/promises';

import { beforeAll, describe, expect, it } from 'vitest';

import { SCENE_SAMPLES } from '@/figure/samples';
import { initSync, parseScene, renderScene } from '@/wasm/figure';
import type { RenderOutcome, SceneOutcome } from '@/wasm/figure';

/**
 * Rustで書いたシーンの読み込みを，Wasmにしたものを，そのまま呼んで，JavaScriptとの約束を確かめる．
 * 検査の網羅は，Rust側のテストが担う．ここでは，値の形と，確認のページに置く見本の結果を確かめる．
 */
beforeAll(async () => {
  const module = await readFile(new URL('../wasm/figure_bg.wasm', import.meta.url));
  initSync({ module });
});

function sampleJson(id: string): string {
  const found = SCENE_SAMPLES.find((candidate) => candidate.id === id);
  if (found === undefined) {
    throw new Error(`見本「${id}」がない`);
  }
  return found.json;
}

function sample(id: string): SceneOutcome {
  return parseScene(sampleJson(id));
}

function render(id: string): RenderOutcome {
  return renderScene(sampleJson(id));
}

/** ネイティブのRustのテストが確かめる，期待するTikZの出力． */
const GOLDEN_TIKZ = new URL(
  '../../../crates/figure/tests/golden/sine-and-shifted-sine.tikz',
  import.meta.url,
);
const GOLDEN_SPACE_TIKZ = new URL(
  '../../../crates/figure/tests/golden/sphere-with-axes.tikz',
  import.meta.url,
);

describe('Wasmのシーンの読み込み', () => {
  it('最初の図を読み，版とオブジェクトの一覧を返す', () => {
    const outcome = sample('first');
    expect(outcome.status).toBe('ok');
    if (outcome.status === 'ok') {
      expect(outcome.version).toBe('0.1.0');
      expect(outcome.engineVersion).toBe('0.1.0');
      expect(outcome.objects.map(({ id, type }) => `${id}:${type}`)).toEqual([
        'x_axis:axis',
        'y_axis:axis',
        'origin_label:label',
        'shift:parameter',
        'sine:graph',
        'shifted_sine:graph',
        'title:label',
      ]);
    }
  });

  it('読み直したシーンは，既定値を補ったJSONで，そのまま読み直せる', () => {
    const outcome = sample('first');
    expect(outcome.status).toBe('ok');
    if (outcome.status === 'ok') {
      const again = parseScene(outcome.canonical);
      expect(again).toEqual(outcome);
    }
  });

  it('日本語の説明と，数式の文字列を，そのまま返す', () => {
    const outcome = sample('first');
    expect(outcome.status === 'ok' && outcome.description).toBe(
      'y=sin x のグラフと，x軸の方向に平行移動した点線のグラフ',
    );
    // JSONの文字列の中では，TeXの`\`が，`\\`と書かれる．
    expect(outcome.status === 'ok' && outcome.canonical).toContain(
      String.raw`Graph of $y=\\sin x$`,
    );
  });

  it('見本ごとに，決まった種類の誤りを，例外ではなく値で返す', () => {
    const codes = Object.fromEntries(
      SCENE_SAMPLES.map(({ id }) => {
        const outcome = sample(id);
        return [id, outcome.status === 'ok' ? 'ok' : outcome.code];
      }),
    );
    expect(codes).toEqual({
      first: 'ok',
      newer: 'incompatible_version',
      unknownField: 'invalid',
      duplicateId: 'duplicate_id',
      syntax: 'json',
      range: 'invalid_range',
      expression: 'expression',
      space: 'ok',
      missingRange: 'invalid',
    });
  });

  it('オブジェクトの誤りは，そのidを持つ', () => {
    const unknown = sample('unknownField');
    expect(unknown.status === 'error' && unknown.object).toBe('x_axis');
    expect(unknown.status === 'error' && unknown.message).toContain('colour');
    const duplicate = sample('duplicateId');
    expect(duplicate.status === 'error' && duplicate.object).toBe('sine');
  });

  it('JSONの構文の誤りは，行と列を持ち，オブジェクトを持たない', () => {
    const outcome = sample('syntax');
    expect(outcome.status).toBe('error');
    if (outcome.status === 'error') {
      expect(outcome.line).toBeGreaterThanOrEqual(1);
      expect(outcome.column).toBeGreaterThanOrEqual(1);
      expect(outcome.object).toBeNull();
    }
  });

  it('新しい版の誤りは，両方の版を説明に含む', () => {
    const outcome = sample('newer');
    expect(outcome.status === 'error' && outcome.message).toContain('99.0.0');
    expect(outcome.status === 'error' && outcome.message).toContain('0.1.0');
  });

  it('式の誤りは，オブジェクトのidと，項目と，式の中の位置を持つ', () => {
    for (const outcome of [sample('expression'), render('expression')]) {
      expect(outcome.status).toBe('error');
      if (outcome.status === 'error') {
        expect(outcome.code).toBe('expression');
        expect(outcome.object).toBe('shifted_sine');
        expect(outcome.field).toBe('expr');
        expect(outcome.index).toBe(0);
        expect(outcome.start).toBeGreaterThanOrEqual(0);
        expect(outcome.end).toBeGreaterThanOrEqual(outcome.start ?? 0);
        expect(outcome.line).toBeNull();
      }
    }
  });
});

describe('Wasmの空間の図', () => {
  it('球と軸の図を読み，オブジェクトの一覧を返す', () => {
    const outcome = sample('space');
    expect(outcome.status).toBe('ok');
    if (outcome.status === 'ok') {
      expect(outcome.objects.map(({ id, type }) => `${id}:${type}`)).toEqual([
        'x_axis:axis',
        'y_axis:axis',
        'z_axis:axis',
        'origin_label:label',
        'ball:sphere',
      ]);
    }
  });

  it('隠れた部分を点線にして描画し，球の輪郭は実線である', () => {
    const outcome = render('space');
    if (outcome.status !== 'ok') {
      throw new Error('描画できる');
    }
    const paths = outcome.figure.items.filter((item) => item.type === 'path');
    const dotted = paths.filter((item) => item.stroke.line === 'dotted');
    // 3本の軸が，それぞれ球に隠れる．
    expect(dotted).toHaveLength(3);
    expect(paths.at(-1)?.stroke.line).toBe('solid');
    expect(outcome.figure.items.filter((item) => item.type === 'label')).toHaveLength(4);
  });

  it('TikZは，ネイティブのRustで作った期待する出力と，一字も違わない', async () => {
    const outcome = render('space');
    const expected = await readFile(GOLDEN_SPACE_TIKZ, 'utf8');
    expect(outcome.status === 'ok' && outcome.tikz).toBe(expected);
  });

  it('空間の軸に範囲がないと，オブジェクトのidつきの誤りになる', () => {
    const outcome = sample('missingRange');
    expect(outcome.status === 'error' && outcome.code).toBe('invalid');
    expect(outcome.status === 'error' && outcome.object).toBe('x_axis');
    expect(outcome.status === 'error' && outcome.message).toContain('range');
  });
});

describe('Wasmの描画', () => {
  it('最初の図を描画し，描く順に並んだ中間表現を返す', () => {
    const outcome = render('first');
    expect(outcome.status).toBe('ok');
    if (outcome.status === 'ok') {
      expect(outcome.figure.items.map(({ type }) => type)).toEqual([
        'path',
        'label',
        'path',
        'label',
        'label',
        'path',
        'path',
        'label',
      ]);
      expect(outcome.figure.bounds.min[0]).toBeCloseTo(-7.6);
    }
  });

  it('軸の矢じりと，ラベルの向きを，型どおりの値で返す', () => {
    const outcome = render('first');
    if (outcome.status !== 'ok') {
      throw new Error('描画できる');
    }
    const { items } = outcome.figure;
    const axis = items.at(0);
    const name = items.at(1);
    const origin = items.at(4);
    const graph = items.at(5);
    const shifted = items.at(6);
    expect(axis?.type === 'path' && axis.arrow?.kind).toBe('stealth');
    expect(axis?.type === 'path' && axis.arrow?.polygon).toHaveLength(4);
    expect(name?.type === 'label' && name.anchor).toBe('west');
    expect(origin?.type === 'label' && origin.anchor).toBe('north west');
    expect(shifted?.type === 'path' && shifted.stroke.line).toBe('dotted');
    // 矢じりのない線は，nullである(undefinedではない)．
    expect(graph?.type === 'path' && graph.arrow).toBeNull();
  });

  it('TikZは，ネイティブのRustで作った期待する出力と，一字も違わない', async () => {
    const outcome = render('first');
    const expected = await readFile(GOLDEN_TIKZ, 'utf8');
    expect(outcome.status === 'ok' && outcome.tikz).toBe(expected);
  });
});
