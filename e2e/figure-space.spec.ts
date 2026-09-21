import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';

const PAGE = 'dev/figure-space/';
const TOLERANCE = 3;
const PX_PER_CM = 37.795;

interface Box {
  x: number;
  y: number;
  width: number;
  height: number;
}

async function box(locator: Locator): Promise<Box> {
  const found = await locator.boundingBox();
  if (found === null) {
    throw new Error('表示されていない要素は，測れない');
  }
  return found;
}

/** 線幅を含まない，図形の寸法．Playwrightのboundingboxは，線幅と，とがりの分を足す． */
function geometry(locator: Locator): Promise<Box> {
  return locator.evaluate((element) => {
    const { x, y, width, height } = element.getBoundingClientRect();
    return { x, y, width, height };
  });
}

/** ページの，最初の図と，2つ目の図． */
function first(page: Page): Locator {
  return page.locator('.figure').first();
}

function second(page: Page): Locator {
  return page.locator('.figure').nth(1);
}

function third(page: Page): Locator {
  return page.locator('.figure').nth(2);
}

function fourth(page: Page): Locator {
  return page.locator('.figure').nth(3);
}

function fifth(page: Page): Locator {
  return page.locator('.figure').nth(4);
}

function center({ x, y, width, height }: Box): [number, number] {
  return [x + width / 2, y + height / 2];
}

test.describe('空間の図', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(PAGE);
    // ラベルの数式が描画されるまで待つ．
    await expect(page.locator('.figure-label mjx-container')).toHaveCount(24);
  });

  test('SVGは，画像として説明を持ち，隠れた部分で分かれた線と，輪郭がある', async ({ page }) => {
    const svg = first(page).locator('svg');
    await expect(svg).toHaveAttribute('role', 'img');
    await expect(svg).toHaveAttribute('aria-label', /半径2の球/u);
    // 軸3本は，それぞれ3つに分かれ，球の輪郭が1本．
    await expect(svg.locator('path')).toHaveCount(10);
    // 矢じりは，各軸の，見える先端に付く．
    await expect(svg.locator('polygon')).toHaveCount(3);
  });

  test('球に隠れた部分だけが点線で，輪郭と，見える部分は実線である', async ({ page }) => {
    const paths = first(page).locator('svg path');
    const dashed = await paths.evaluateAll((elements) =>
      elements.map((element) => element.hasAttribute('stroke-dasharray')),
    );
    expect(dashed).toEqual([false, true, false, false, true, false, false, true, false, false]);
  });

  test('球の輪郭は，半径1.6cmの円で，z軸の上にある', async ({ page }) => {
    const paths = first(page).locator('svg path');
    const outline = await geometry(paths.nth(9));
    expect(Math.abs(outline.width - 3.2 * PX_PER_CM)).toBeLessThan(TOLERANCE);
    expect(Math.abs(outline.height - 3.2 * PX_PER_CM)).toBeLessThan(TOLERANCE);
    // 原点の投影は，z軸の上にある．
    const zAxis = await geometry(paths.nth(6));
    expect(Math.abs(center(outline)[0] - center(zAxis)[0])).toBeLessThan(TOLERANCE);
  });

  test('軸の点線の部分は，球の輪郭の内側にある', async ({ page }) => {
    const paths = first(page).locator('svg path');
    const outline = await geometry(paths.nth(9));
    const [cx, cy] = center(outline);
    const radius = outline.width / 2;
    const hiddenPieces = await Promise.all([1, 4, 7].map((index) => geometry(paths.nth(index))));
    for (const hidden of hiddenPieces) {
      const corners = [
        [hidden.x, hidden.y],
        [hidden.x + hidden.width, hidden.y + hidden.height],
        [hidden.x + hidden.width, hidden.y],
        [hidden.x, hidden.y + hidden.height],
      ];
      for (const [x, y] of corners) {
        expect(Math.hypot(x - cx, y - cy)).toBeLessThanOrEqual(radius + TOLERANCE);
      }
    }
  });

  test('xの名前は，x軸の左下の端の，左にある', async ({ page }) => {
    const paths = first(page).locator('svg path');
    const axis = await box(paths.nth(2));
    const label = await box(first(page).locator('.figure-label').nth(0));
    // x軸の先は，画面の左下へ向かう．名前の箱の右端が，軸の左の端に合う．
    expect(Math.abs(label.x + label.width - axis.x)).toBeLessThan(TOLERANCE * 2);
    expect(label.y).toBeGreaterThan(axis.y - label.height);
  });

  test('zの名前は，z軸の上の端の，上にある', async ({ page }) => {
    const axis = await box(first(page).locator('svg path').nth(8));
    const label = await box(first(page).locator('.figure-label').nth(2));
    expect(label.y + label.height).toBeLessThanOrEqual(axis.y + TOLERANCE);
    expect(Math.abs(center(label)[0] - center(axis)[0])).toBeLessThan(TOLERANCE);
  });

  test('Oの名前は，原点の左下に置く', async ({ page }) => {
    const paths = first(page).locator('svg path');
    const outline = await geometry(paths.nth(9));
    const [cx, cy] = center(outline);
    const label = await box(first(page).locator('.figure-label').nth(3));
    // 箱の右上の角が，原点(球の中心)に合う．
    expect(Math.abs(label.x + label.width - cx)).toBeLessThan(TOLERANCE * 2);
    expect(Math.abs(label.y - cy)).toBeLessThan(TOLERANCE * 2);
  });

  test('幅の狭い画面では，画面に収まるよう縮み，横にはみ出さない', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 800 });
    const svg = await box(first(page).locator('svg'));
    expect(svg.width).toBeLessThanOrEqual(390);
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
    );
    expect(overflow).toBeLessThanOrEqual(0);
  });

  test('球の面の上の円は，遠い側が点線で，輪郭と同じ球に沿う', async ({ page }) => {
    const paths = second(page).locator('svg path');
    // 軸9つ，円2つが3つずつ，球の輪郭1本．
    await expect(paths).toHaveCount(16);
    const dashed = await paths.evaluateAll((elements) =>
      elements.map((element) => element.hasAttribute('stroke-dasharray')),
    );
    expect(dashed.filter(Boolean)).toHaveLength(5);
    // 円の点線は，どれも，球の輪郭の外接する正方形の内側にある．
    const outline = await geometry(paths.nth(15));
    const circles = await Promise.all([10, 13].map((index) => geometry(paths.nth(index))));
    for (const hidden of circles) {
      expect(hidden.x).toBeGreaterThanOrEqual(outline.x - TOLERANCE);
      expect(hidden.y).toBeGreaterThanOrEqual(outline.y - TOLERANCE);
      expect(hidden.x + hidden.width).toBeLessThanOrEqual(outline.x + outline.width + TOLERANCE);
      expect(hidden.y + hidden.height).toBeLessThanOrEqual(outline.y + outline.height + TOLERANCE);
    }
  });

  test('式で書いた曲面は，輪郭と縁が実線で，曲面に隠れた軸が点線で描かれる', async ({ page }) => {
    const svg = third(page).locator('svg');
    // 矢じりは，見える先端に付く3つである．
    await expect(svg.locator('polygon')).toHaveCount(3);
    const dashed = await svg
      .locator('path')
      .evaluateAll((elements) =>
        elements.map((element) => element.hasAttribute('stroke-dasharray')),
      );
    expect(dashed.some(Boolean)).toBe(true);
    expect(dashed.some((value) => !value)).toBe(true);
    // 線の太さ0.8ptの実線(輪郭と縁)が，あわせて2本以上ある．
    const widths = await svg
      .locator('path')
      .evaluateAll((elements) =>
        elements.map((element) =>
          Number(/[\d.]+/u.exec(getComputedStyle(element).strokeWidth)?.[0]),
        ),
      );
    const thick = widths.filter((width) => Math.abs(width - (0.8 * 2.54) / 72.27) < 1e-4);
    expect(thick.length).toBeGreaterThanOrEqual(2);
  });

  test('切り口は，色を付けた線で描かれ，曲面に隠れる部分は点線になる', async ({ page }) => {
    const paths = fourth(page).locator('svg path');
    const strokes = await paths.evaluateAll((elements) =>
      elements.map((element) => ({
        stroke: element.getAttribute('stroke'),
        dashed: element.hasAttribute('stroke-dasharray'),
      })),
    );
    const blue = strokes.filter(({ stroke }) => stroke === 'var(--figure-blue)');
    const red = strokes.filter(({ stroke }) => stroke === 'var(--figure-red)');
    // 各切り口は，見える部分と隠れる部分に分かれる．
    expect(blue.length).toBeGreaterThanOrEqual(2);
    expect(red.length).toBeGreaterThanOrEqual(2);
    expect(blue.some(({ dashed }) => dashed)).toBe(true);
    expect(blue.some(({ dashed }) => !dashed)).toBe(true);
  });

  test('空間のベクトルの和の図は，対角線のベクトルが，Dの印に届く', async ({ page }) => {
    const svg = fifth(page).locator('svg');
    // 軸3本と，破線の辺2本と，ベクトル3本．矢じりは，軸とベクトルの6つ．印は1つ．
    await expect(svg.locator('path')).toHaveCount(8);
    await expect(svg.locator('polygon')).toHaveCount(6);
    await expect(svg.locator('circle')).toHaveCount(1);
    // 対角線の矢じりの先端は，印の中心にほぼ重なる(単位はcm)．
    const tip = await svg.locator('polygon').nth(5).getAttribute('points');
    const [tipX, tipY] = (tip ?? '').split(' ')[0]?.split(',').map(Number) ?? [];
    const dot = svg.locator('circle');
    const cx = Number(await dot.getAttribute('cx'));
    const cy = Number(await dot.getAttribute('cy'));
    expect(Math.hypot((tipX ?? Number.NaN) - cx, (tipY ?? Number.NaN) - cy)).toBeLessThan(0.05);
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
    await expect(page.locator('.figure-label mjx-container')).toHaveCount(24);
    expect(problems).toEqual([]);
  });
});
