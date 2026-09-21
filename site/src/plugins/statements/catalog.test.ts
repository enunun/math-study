import { utimes, writeFile } from 'node:fs/promises';
import path from 'node:path';

import { afterAll, describe, expect, it } from 'vitest';

import { createCatalog } from './catalog';
import { BASE, createSite, removeSites } from './test-support';

/** 更新時刻を確実に変えるための，進める時間(ミリ秒)． */
const LATER_MS = 10_000;

afterAll(removeSites);

describe('createCatalog', () => {
  it('ファイルを編集すると，次の検索で読み直す', async () => {
    const { directory } = await createSite({ 'abs.mdx': '<Theorem id="a">1</Theorem>' });
    const catalog = createCatalog({ contentDirectory: directory, base: BASE });
    const before = await catalog.lookup('abs');
    expect(before?.statements.map(({ id }) => id)).toEqual(['a']);
    const file = path.join(directory, 'abs.mdx');
    await writeFile(file, '<Lemma id="b">1</Lemma><Theorem id="a">2</Theorem>');
    const later = new Date(Date.now() + LATER_MS);
    await utimes(file, later, later);
    const after = await catalog.lookup('abs');
    expect(after?.statements.map(({ id }) => id)).toEqual(['b', 'a']);
  });

  it('定義や定理のないページと，読み込めないページは，一覧に載せない', async () => {
    const { directory } = await createSite({
      'plain.mdx': '本文だけ．',
      'broken.mdx': '<Theorem>',
    });
    const catalog = createCatalog({ contentDirectory: directory, base: BASE });
    expect(await catalog.lookup('plain')).toBeUndefined();
    expect(await catalog.lookup('broken')).toBeUndefined();
  });
});
