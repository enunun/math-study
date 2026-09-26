import { AxeBuilder } from '@axe-core/playwright';
import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';

const PAGE = 'topics/figure-algorithms/';

function explorer(page: Page): Locator {
  return page.locator('.projection-explorer');
}

function azimuth(page: Page): Locator {
  return explorer(page).getByRole('slider', { name: /方位角/u });
}

function elevation(page: Page): Locator {
  return explorer(page).getByRole('slider', { name: /仰角/u });
}

/** 行列の数値を読み上げる平文のラベル．方位角と仰角を動かすたびに，式の描画を待たずに，書き換わる． */
function matrixLabels(page: Page): Promise<string[]> {
  return explorer(page)
    .locator('.math-view')
    .evaluateAll((elements) => elements.map((element) => element.getAttribute('aria-label') ?? ''));
}

test.describe('投影の図(方位角と仰角を動かす)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(PAGE);
    await explorer(page).scrollIntoViewIfNeeded();
    // 図と，行列の式が描画されるまで待つ．
    await expect(explorer(page).locator('svg')).toBeVisible();
    await expect(explorer(page).locator('.math-view mjx-container')).toHaveCount(2);
  });

  test('最初は，解説の値(方位角60度，仰角20度)で，点Pの像と奥行きが合う', async ({ page }) => {
    await expect(azimuth(page)).toHaveValue('60');
    await expect(elevation(page)).toHaveValue('20');
    const [basis, product] = await matrixLabels(page);
    expect(basis).toContain('r=(-0.87, 0.50, 0)');
    expect(basis).toContain('u=(-0.17, -0.30, 0.94)');
    expect(basis).toContain('d=(0.47, 0.81, 0.34)');
    expect(product).toContain('画面の位置は(-2.10, 1.07)');
    expect(product).toContain('奥行きは2.91');
  });

  test('方位角と仰角を0度にすると，x軸が手前を向き，y軸が右，z軸が上になる', async ({ page }) => {
    await azimuth(page).fill('0');
    await elevation(page).fill('0');
    const [basis, product] = await matrixLabels(page);
    expect(basis).toContain('r=(0, 1.00, 0)');
    expect(basis).toContain('u=(0, 0, 1.00)');
    expect(basis).toContain('d=(1.00, 0, 0)');
    // 点(3, 1, 2)は，右へ1，上へ2に写り，カメラの側へ3の奥行きになる．
    expect(product).toContain('画面の位置は(1.00, 2.00)');
    expect(product).toContain('奥行きは3.00');
  });

  test('向きを変えると，図が描き直される', async ({ page }) => {
    const before = await explorer(page).locator('svg').innerHTML();
    await azimuth(page).fill('-40');
    await expect(async () => {
      expect(await explorer(page).locator('svg').innerHTML()).not.toBe(before);
    }).toPass();
  });

  test('自由な曲面を加えると，図の線が増える', async ({ page }) => {
    const paths = explorer(page).locator('svg path');
    const before = await paths.count();
    await explorer(page).getByRole('checkbox').check();
    await expect(async () => {
      expect(await paths.count()).toBeGreaterThan(before);
    }).toPass();
  });

  test('スライダーは，キーボードで動かせて，アクセシビリティの違反がない', async ({ page }) => {
    await azimuth(page).focus();
    await page.keyboard.press('ArrowRight');
    await expect(azimuth(page)).toHaveValue('61');
    const results = await new AxeBuilder({ page }).include('.projection-explorer').analyze();
    expect(results.violations).toEqual([]);
  });
});
