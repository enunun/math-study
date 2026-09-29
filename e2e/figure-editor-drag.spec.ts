import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';

import { loadSample } from './figure-editor-pickers';

const PAGE = 'tools/graphics/figure-editor/';

function editor(page: Page): Locator {
  return page.locator('.figure-editor');
}

function preview(page: Page): Locator {
  return editor(page).locator('.fe-edit-preview svg');
}

function outputPreview(page: Page): Locator {
  return editor(page).locator('.fe-output-preview svg');
}

async function openTab(page: Page, name: string): Promise<void> {
  await editor(page)
    .getByRole('group', { name: '表示の切り替え' })
    .getByRole('button', { name })
    .click();
}

test.describe('点のドラッグ', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(PAGE);
    await expect(preview(page)).toBeVisible();
  });

  test('電荷の点をドラッグすると，位置が図に書き込まれ，ベクトル場も描き直される', async ({
    page,
  }) => {
    // 丸めや近い点の選び方はpoint-drag.test.tsで確かめるので，ここでは，実際のポインタの操作が図に届くことを確かめる．
    await loadSample(page, ['平面', 'ベクトル場', '2つの点電荷の電場']);
    await expect(preview(page)).toBeVisible();
    const before = await outputPreview(page).innerHTML();
    await preview(page).scrollIntoViewIfNeeded();
    // 点Q1は，数学の座標(-1.5, 0)，単位1cmなので，SVGの座標(cm)でも(-1.5, 0)にある．
    const start = await preview(page).evaluate((svg) => {
      if (!(svg instanceof SVGSVGElement)) {
        throw new TypeError('SVGではない');
      }
      const matrix = svg.getScreenCTM();
      if (matrix === null) {
        throw new Error('画面の位置がない');
      }
      const point = new DOMPoint(-1.5, 0).matrixTransform(matrix);
      return { x: point.x, y: point.y };
    });
    await page.mouse.move(start.x, start.y);
    await page.mouse.down();
    await page.mouse.move(start.x + 40, start.y - 30, { steps: 5 });
    await page.mouse.up();
    await expect.poll(() => outputPreview(page).innerHTML()).not.toBe(before);
    await openTab(page, 'JSON');
    const json = await editor(page).getByRole('textbox', { name: 'シーン(JSON)' }).inputValue();
    expect(json).not.toMatch(/"at": \[\s*-1\.5,\s*0\s*\]/u);
    expect(json).toMatch(/"id": "Q2",\s*"type": "point",\s*"at": \[\s*1\.5,\s*0\s*\]/u);
    await expect(editor(page).getByRole('alert')).toHaveCount(0);
  });
});
