import { readFile } from 'node:fs/promises';

import { beforeAll, describe, expect, it } from 'vitest';

import { SCENE_SAMPLES } from '@/figure/samples';
import { initSync, parseScene } from '@/wasm/figure';
import type { SceneOutcome } from '@/wasm/figure';

/**
 * Rustで書いたシーンの読み込みを，Wasmにしたものを，そのまま呼んで，JavaScriptとの約束を確かめる．
 * 検査の網羅は，Rust側のテストが担う．ここでは，値の形と，確認のページに置く見本の結果を確かめる．
 */
beforeAll(async () => {
  const module = await readFile(new URL('../wasm/figure_bg.wasm', import.meta.url));
  initSync({ module });
});

function sample(id: string): SceneOutcome {
  const found = SCENE_SAMPLES.find((candidate) => candidate.id === id);
  if (found === undefined) {
    throw new Error(`見本「${id}」がない`);
  }
  return parseScene(found.json);
}

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
});
