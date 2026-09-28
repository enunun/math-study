import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';

import { loadSample } from './figure-editor-pickers';

const PAGE = 'tools/graphics/figure-editor/';

function editor(page: Page): Locator {
  return page.locator('.figure-editor');
}

function objects(page: Page): Locator {
  return editor(page).getByRole('list', { name: 'オブジェクトの一覧' }).getByRole('listitem');
}

function historyButton(page: Page, name: string): Locator {
  return editor(page)
    .getByRole('group', { name: '編集の履歴' })
    .getByRole('button', { name, exact: true });
}

async function addPoint(page: Page): Promise<void> {
  await editor(page).getByLabel('追加する種類').selectOption({ label: '点' });
  await editor(page).getByRole('button', { name: '追加', exact: true }).click();
}

test.describe('元に戻す・やり直す', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(PAGE);
    await expect(editor(page).locator('.fe-edit-preview svg')).toBeVisible();
  });

  test('ボタンで，オブジェクトの追加を戻し，やり直せる', async ({ page }) => {
    await editor(page).getByRole('button', { name: '座標軸(平面)', exact: true }).click();
    const count = await objects(page).count();
    await addPoint(page);
    await expect(objects(page)).toHaveCount(count + 1);
    await historyButton(page, '元に戻す').click();
    await expect(objects(page)).toHaveCount(count);
    await expect(historyButton(page, 'やり直す')).toBeEnabled();
    await historyButton(page, 'やり直す').click();
    await expect(objects(page)).toHaveCount(count + 1);
    await expect(historyButton(page, 'やり直す')).toBeDisabled();
  });

  test('Ctrl+Zで見本の読み込みを戻し，Ctrl+Shift+Zでやり直せる', async ({ page }) => {
    await editor(page).getByRole('button', { name: '座標軸(平面)', exact: true }).click();
    const count = await objects(page).count();
    await loadSample(page, '平面：ベクトル場', '2つの点電荷の電場');
    await expect(objects(page)).not.toHaveCount(count);
    const sampleCount = await objects(page).count();
    await page.locator('body').click({ position: { x: 1, y: 1 } });
    await page.keyboard.press('Control+z');
    await expect(objects(page)).toHaveCount(count);
    await page.keyboard.press('Control+Shift+z');
    await expect(objects(page)).toHaveCount(sampleCount);
  });
});
