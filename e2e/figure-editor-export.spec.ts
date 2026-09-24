import { readFile } from 'node:fs/promises';

import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';
import { PNG } from 'pngjs';

function editor(page: Page): Locator {
  return page.locator('.figure-editor');
}

/** ボタンを押して，始まったダウンロードの内容を，バイト列のまま読む． */
async function downloadedBytes(page: Page, name: string): Promise<{ file: string; bytes: Buffer }> {
  const started = page.waitForEvent('download');
  await editor(page).getByRole('button', { name }).click();
  const download = await started;
  return { file: download.suggestedFilename(), bytes: await readFile(await download.path()) };
}

/** ボタンを押して，ダウンロードした画像のバイト列を返す． */
async function downloadedImage(page: Page, name: string): Promise<Buffer> {
  const { bytes } = await downloadedBytes(page, name);
  return bytes;
}

test.describe('図の作成：画像の書き出し', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('tools/figure-editor/');
    await expect(editor(page).locator('.fe-output-preview svg')).toBeVisible();
  });

  test('画像をSVG，PNG，JPEGで書き出せ，透過できる形式では背景を透過できる', async ({ page }) => {
    await editor(page).getByRole('button', { name: '関数のグラフ', exact: true }).click();
    const format = editor(page).getByLabel('画像の形式');
    const transparent = editor(page).getByLabel(/背景を透過する/u);

    await format.selectOption({ label: 'SVG' });
    const svg = await downloadedBytes(page, '画像を書き出す(.svg)');
    expect(svg.file).toBe('figure.svg');
    const text = svg.bytes.toString('utf8');
    // XMLとして読め，ラベルの数式を，文字の形のパスとして持つ．
    const parsed = await page.evaluate((source) => {
      const document = new DOMParser().parseFromString(source, 'image/svg+xml');
      return {
        error: document.querySelector('parsererror') !== null,
        math: document.querySelectorAll('[data-mml-node="math"]').length,
      };
    }, text);
    expect(parsed.error).toBe(false);
    expect(parsed.math).toBeGreaterThan(0);
    expect(text).toContain('fill="#ffffff"');

    await format.selectOption({ label: 'PNG' });
    const opaque = PNG.sync.read(await downloadedImage(page, '画像を書き出す(.png)'));
    // 左上の隅は，白く塗った背景である．
    expect([...opaque.data.subarray(0, 4)]).toEqual([255, 255, 255, 255]);
    await transparent.check();
    const clear = PNG.sync.read(await downloadedImage(page, '画像を書き出す(.png)'));
    expect(clear.data[3]).toBe(0);
    // 300dpiなので，1cmあたり約118画素である．
    expect(clear.width).toBeGreaterThan(300);

    await format.selectOption({ label: 'JPEG' });
    await expect(transparent).toBeDisabled();
    await expect(transparent).not.toBeChecked();
    const jpeg = await downloadedBytes(page, '画像を書き出す(.jpg)');
    expect(jpeg.file).toBe('figure.jpg');
    // JPEGのファイルは，0xFF 0xD8(SOI)で始まる．
    expect([...jpeg.bytes.subarray(0, 2)]).toEqual([255, 216]);
    await expect(editor(page).getByRole('status')).toHaveText('JPEGを書き出した．');
  });
});
