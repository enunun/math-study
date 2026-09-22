import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';

const PAGE = 'dev/figure-first/';
const TOLERANCE = 3;

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

/** 線幅を含まない，図形の寸法．Playwrightのboundingboxは，線幅と，とがりの分を足す． */
function geometry(locator: Locator): Promise<Box> {
  return locator.evaluate((element) => {
    const { x, y, width, height } = element.getBoundingClientRect();
    return { x, y, width, height };
  });
}

interface Box {
  x: number;
  y: number;
  width: number;
  height: number;
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

test.describe('シーンから描いた図の部品', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(PAGE);
    // ラベルの数式が描画されるまで待つ．
    await expect(page.locator('.figure-label mjx-container')).toHaveCount(34);
  });

  test('目盛の線は，軸に直角な，3pt(約4px)ずつ両側に出る短い線である', async ({ page }) => {
    const paths = second(page).locator('svg path');
    // x軸1本，目盛4本，y軸1本，目盛2本，sin x．
    await expect(paths).toHaveCount(9);
    const xTick = await geometry(paths.nth(1));
    expect(xTick.width).toBeLessThan(1);
    expect(Math.abs(xTick.height - 2 * 3 * 1.333)).toBeLessThan(1);
    const yTick = await geometry(paths.nth(6));
    expect(yTick.height).toBeLessThan(1);
    expect(Math.abs(yTick.width - 2 * 3 * 1.333)).toBeLessThan(1);
  });

  test('目盛の名前は，anchorに従い，曲線を避けて置かれ，y軸の名前は，目盛の左にある', async ({
    page,
  }) => {
    const paths = second(page).locator('svg path');
    const labels = second(page).locator('.figure-label');
    const xAxis = await geometry(paths.nth(0));
    const yAxis = await geometry(paths.nth(5));
    const piTick = await geometry(paths.nth(3));
    const twoPiTick = await geometry(paths.nth(4));
    // `north east`の名前は，目盛の左下に延び，`north west`の名前は，右下に延びる．
    const pi = await box(labels.nth(2));
    const twoPi = await box(labels.nth(3));
    expect(Math.abs(pi.x + pi.width - piTick.x)).toBeLessThan(TOLERANCE);
    expect(Math.abs(twoPi.x - twoPiTick.x)).toBeLessThan(TOLERANCE);
    expect(pi.y).toBeGreaterThanOrEqual(xAxis.y + TOLERANCE);
    const oneTick = await geometry(paths.nth(6));
    const one = await box(labels.nth(5));
    expect(one.x + one.width).toBeLessThanOrEqual(yAxis.x + TOLERANCE);
    expect(Math.abs(one.y + one.height / 2 - (oneTick.y + oneTick.height / 2))).toBeLessThan(
      TOLERANCE,
    );
  });

  test('格子は，細い点線で，見える範囲を区切り，曲線は，格子の外へ出ない', async ({ page }) => {
    const paths = third(page).locator('svg path');
    // 縦7本，横7本，軸2本，曲線2本．
    await expect(paths).toHaveCount(18);
    const dashed = await paths.evaluateAll((elements) =>
      elements.map((element) => element.hasAttribute('stroke-dasharray')),
    );
    // 格子の点線14本と，破線の接線．
    expect(dashed.filter(Boolean)).toHaveLength(15);
    const firstLine = await geometry(paths.nth(0));
    const top = await geometry(paths.nth(13));
    const curve = await geometry(paths.nth(16));
    // 縦の線は，1cm(約37.8px)おきで，見える範囲の上の辺から下の辺まで届く．
    const secondLine = await geometry(paths.nth(1));
    expect(Math.abs(secondLine.x - firstLine.x - 37.795)).toBeLessThan(1);
    expect(firstLine.height).toBeGreaterThan(6 * 37.795 - 1);
    // 曲線の頂は，格子の上の辺(y=5)に届き，それより上へ出ない．
    expect(curve.y).toBeGreaterThanOrEqual(top.y - 1);
    expect(curve.y).toBeLessThan(top.y + 2);
  });

  test('線の太さは，指定した長さで描かれる', async ({ page }) => {
    const paths = third(page).locator('svg path');
    // SVGの座標の単位はcmで，線の太さも，cmで書かれる．放物線は1.2pt，格子は0.3ptである．
    const widths = await paths.evaluateAll((elements) =>
      elements.map((element) => Number(/[\d.]+/u.exec(getComputedStyle(element).strokeWidth)?.[0])),
    );
    const cmPerPt = 2.54 / 72.27;
    expect(Math.abs((widths[16] ?? 0) - 1.2 * cmPerPt)).toBeLessThan(1e-4);
    expect(Math.abs((widths[0] ?? 0) - 0.3 * cmPerPt)).toBeLessThan(1e-4);
  });

  test('ベクトルの和の図は，対角線のベクトルが，Dの印に届き，名前は，印の右上にある', async ({
    page,
  }) => {
    const svg = fourth(page).locator('svg');
    // 破線の辺2本と，ベクトル3本．矢じりは3つ，点の印は1つ．
    await expect(svg.locator('path')).toHaveCount(5);
    await expect(svg.locator('polygon')).toHaveCount(3);
    await expect(svg.locator('circle')).toHaveCount(1);
    const dashed = await svg
      .locator('path')
      .evaluateAll((elements) =>
        elements.map((element) => element.hasAttribute('stroke-dasharray')),
      );
    expect(dashed).toEqual([true, true, false, false, false]);
    // 対角線の矢じりの先端(多角形の最初の点)は，印の中心にほぼ重なる(単位はcm)．
    const tip = await svg.locator('polygon').nth(2).getAttribute('points');
    const [tipX, tipY] = (tip ?? '').split(' ')[0]?.split(',').map(Number) ?? [];
    const dot = svg.locator('circle');
    const cx = Number(await dot.getAttribute('cx'));
    const cy = Number(await dot.getAttribute('cy'));
    expect(Math.hypot((tipX ?? Number.NaN) - cx, (tipY ?? Number.NaN) - cy)).toBeLessThan(0.05);
    // 名前Dの箱の左下の角が，印の中心に合う(anchorはsouth west)．
    const center = await geometry(dot);
    const name = await box(fourth(page).locator('.figure-label').nth(0));
    expect(Math.abs(name.x - (center.x + center.width / 2))).toBeLessThan(TOLERANCE * 2);
    expect(Math.abs(name.y + name.height - (center.y + center.height / 2))).toBeLessThan(
      TOLERANCE * 2,
    );
  });

  test('領域は，青い薄い色で塗られ，線は引かれない', async ({ page }) => {
    const filled = fifth(page).locator('svg path[fill-opacity]');
    await expect(filled).toHaveCount(1);
    await expect(filled).toHaveAttribute('fill', 'var(--figure-blue)');
    await expect(filled).toHaveAttribute('stroke', 'none');
    await expect(filled).toHaveAttribute('fill-opacity', '0.25');
  });

  test('領域の斜線は，2つのグラフの交点の間に収まり，斜めに引かれる', async ({ page }) => {
    const paths = fifth(page).locator('svg path');
    const widths = await paths.evaluateAll((elements) =>
      elements.map((element) => Number(/[\d.]+/u.exec(getComputedStyle(element).strokeWidth)?.[0])),
    );
    // 軸2本とグラフ2本は，0.6ptと0.8pt．斜線は，0.4ptである．
    const cmPerPt = 2.54 / 72.27;
    const hatchIndexes = widths.flatMap((width, index) =>
      Math.abs(width - 0.4 * cmPerPt) < 1e-4 ? [index] : [],
    );
    expect(hatchIndexes.length).toBeGreaterThanOrEqual(6);
    const yAxis = await geometry(paths.nth(1));
    const originX = yAxis.x;
    const cmPx = 37.795;
    const low = originX - 0.75 * Math.PI * cmPx - 2;
    const high = originX + 0.25 * Math.PI * cmPx + 2;
    const boxes = await Promise.all(hatchIndexes.map((index) => geometry(paths.nth(index))));
    for (const line of boxes) {
      expect(line.x).toBeGreaterThanOrEqual(low);
      expect(line.x + line.width).toBeLessThanOrEqual(high);
      // 45度の斜線は，幅と高さが等しい．
      expect(Math.abs(line.width - line.height)).toBeLessThan(2);
    }
  });

  for (const scheme of ['light', 'dark'] as const) {
    test(`色の名前は，背景に合わせた色になる(${scheme})`, async ({ page }) => {
      await page.emulateMedia({ colorScheme: scheme });
      const paths = third(page).locator('svg path');
      const stroke = (index: number): Promise<string> =>
        paths.nth(index).evaluate((element) => getComputedStyle(element).stroke);
      const expected =
        scheme === 'light'
          ? { blue: 'rgb(21, 101, 192)', red: 'rgb(198, 40, 40)', gray: 'rgb(138, 143, 152)' }
          : { blue: 'rgb(130, 177, 255)', red: 'rgb(255, 138, 128)', gray: 'rgb(154, 160, 166)' };
      // 放物線は青，接線は赤，格子は灰色である．
      await expect.poll(() => stroke(16)).toBe(expected.blue);
      expect(await stroke(17)).toBe(expected.red);
      expect(await stroke(0)).toBe(expected.gray);
      // 矢じりのない軸は，文字の色のままである．
      const axis = await stroke(14);
      const text = await third(page).evaluate((element) => getComputedStyle(element).color);
      expect(axis).toBe(text);
    });
  }
});
