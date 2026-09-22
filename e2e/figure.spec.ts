import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';

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
    await expect(page.locator('.figure-label mjx-container')).toHaveCount(34);
  });

  test('SVGは，画像として説明を持ち，線と矢じりがある', async ({ page }) => {
    const svg = first(page).locator('svg');
    await expect(svg).toHaveAttribute('role', 'img');
    await expect(svg).toHaveAttribute('aria-label', /sin x のグラフ/u);
    // 軸2本と曲線2本，軸の矢じり2つ．
    await expect(svg.locator('path')).toHaveCount(4);
    await expect(svg.locator('polygon')).toHaveCount(2);
  });

  test('実線と点線は，線の種類が違う', async ({ page }) => {
    const paths = first(page).locator('svg path');
    await expect(paths.nth(2)).not.toHaveAttribute('stroke-dasharray', /.+/u);
    await expect(paths.nth(3)).toHaveAttribute('stroke-dasharray', /^[\d.]+ [\d.]+$/u);
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

  test('yの名前は，y軸の上の端の，上にあり，軸の上に揃う', async ({ page }) => {
    const axis = await box(first(page).locator('svg path').nth(1));
    const label = await box(first(page).locator('.figure-label').nth(1));
    expect(label.y + label.height).toBeLessThanOrEqual(axis.y + TOLERANCE);
    expect(Math.abs(label.x + label.width / 2 - (axis.x + axis.width / 2))).toBeLessThan(TOLERANCE);
  });

  test('Oの名前は，原点の右下に置く', async ({ page }) => {
    const xAxis = await box(first(page).locator('svg path').nth(0));
    const yAxis = await box(first(page).locator('svg path').nth(1));
    const origin = {
      x: yAxis.x + yAxis.width / 2,
      y: xAxis.y + xAxis.height / 2,
    };
    const label = await box(first(page).locator('.figure-label').nth(2));
    // 箱の左上の角が，原点に合う．
    expect(Math.abs(label.x - origin.x)).toBeLessThan(TOLERANCE);
    expect(Math.abs(label.y - origin.y)).toBeLessThan(TOLERANCE);
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
    test(`図の色は，文字の色に従う(${scheme})`, async ({ page }) => {
      await page.emulateMedia({ colorScheme: scheme });
      const [stroke, text] = await page.evaluate(() => {
        const path = document.querySelector('.figure svg path');
        const body = document.querySelector('.figure');
        return [
          path === null ? '' : getComputedStyle(path).stroke,
          body === null ? '' : getComputedStyle(body).color,
        ];
      });
      expect(stroke).not.toBe('');
      expect(stroke).toBe(text);
    });
  }

  test('読み込みでエラーが出ない', async ({ page }) => {
    const problems: string[] = [];
    page.on('console', (message) => {
      if (message.type() === 'error') {
        problems.push(message.text());
      }
    });
    page.on('pageerror', (error) => problems.push(String(error)));
    page.on('requestfailed', (request) => problems.push(request.url()));
    await page.reload();
    await expect(page.locator('.figure-label mjx-container')).toHaveCount(34);
    expect(problems).toEqual([]);
  });
});
