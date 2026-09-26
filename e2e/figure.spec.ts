import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';

// 図の線の本数，線の種類，太さ，ラベルの位置とアンカーは，site/src/figure/article-figures-{plane,space}.test.tsで確かめる．
// ここでは，ブラウザでしか分からない，ラベルの箱の置き方，実寸，画面の幅，色を確かめる．
const PAGE = 'dev/figure-first/';
const TOLERANCE = 3;

/** ページの，最初の図． */
function first(page: Page): Locator {
  return page.locator('.figure').first();
}

async function box(
  locator: Locator,
): Promise<{ x: number; y: number; width: number; height: number }> {
  const found = await locator.boundingBox();
  if (found === null) {
    throw new Error('表示されていない要素は，測れない');
  }
  return found;
}

test.describe('シーンから描いた図', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(PAGE);
    // ラベルの数式が描画されるまで待つ．
    await expect(page.locator('.figure-label mjx-container')).toHaveCount(44);
  });

  test('xの名前は，x軸の右の端の，右にある', async ({ page }) => {
    const axis = await box(first(page).locator('svg path').nth(0));
    const label = await box(first(page).locator('.figure-label').nth(0));
    expect(label.x).toBeGreaterThanOrEqual(axis.x + axis.width - TOLERANCE);
    // 上下は，軸の高さに揃う．
    expect(Math.abs(label.y + label.height / 2 - (axis.y + axis.height / 2))).toBeLessThan(
      TOLERANCE,
    );
  });

  test('図の実寸は，幅15.2cmで，1cmは，画面の37.8pxである', async ({ page }) => {
    const svg = await box(first(page).locator('svg'));
    // 横は，-7cmから7cmの14cmに，左右の余白0.6cmずつを足した，15.2cmである．
    expect(svg.width / 37.795).toBeGreaterThan(15.1);
    expect(svg.width / 37.795).toBeLessThan(15.3);
  });

  test('幅の狭い画面では，画面に収まるよう縮み，横にはみ出さない', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 800 });
    const svg = await box(first(page).locator('svg'));
    expect(svg.width).toBeLessThanOrEqual(390);
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
    );
    expect(overflow).toBeLessThanOrEqual(0);
    // 縮んでも，ラベルは，図の中に収まる．
    const frame = await box(first(page).locator('.figure-frame'));
    const title = await box(first(page).locator('.figure-label').nth(3));
    expect(title.x + title.width).toBeLessThanOrEqual(frame.x + frame.width + TOLERANCE);
  });

  for (const scheme of ['light', 'dark'] as const) {
    test(`色のない線は文字の色に従い，色の名前は背景に合わせた色になる(${scheme})`, async ({
      page,
    }) => {
      await page.emulateMedia({ colorScheme: scheme });
      // 3つ目の図(parabola-on-grid)の，放物線は青，接線は赤，格子は灰色，軸は色の指定がない．
      const paths = page.locator('.figure').nth(2).locator('svg path');
      const stroke = (index: number): Promise<string> =>
        paths.nth(index).evaluate((element) => getComputedStyle(element).stroke);
      const expected =
        scheme === 'light'
          ? { blue: 'rgb(21, 101, 192)', red: 'rgb(198, 40, 40)', gray: 'rgb(138, 143, 152)' }
          : { blue: 'rgb(130, 177, 255)', red: 'rgb(255, 138, 128)', gray: 'rgb(154, 160, 166)' };
      await expect.poll(() => stroke(16)).toBe(expected.blue);
      expect(await stroke(17)).toBe(expected.red);
      expect(await stroke(0)).toBe(expected.gray);
      const text = await page
        .locator('.figure')
        .nth(2)
        .evaluate((element) => getComputedStyle(element).color);
      expect(await stroke(14)).toBe(text);
    });
  }
});
