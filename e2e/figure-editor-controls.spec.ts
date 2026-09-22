import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';

const PAGE = 'figure-editor/';

function editor(page: Page): Locator {
  return page.locator('.figure-editor');
}

function preview(page: Page): Locator {
  return editor(page).locator('.fe-edit-preview svg');
}

function outputPreview(page: Page): Locator {
  return editor(page).locator('.fe-output-preview svg');
}

async function addObject(page: Page, type: string): Promise<void> {
  await editor(page).getByLabel('追加する種類').selectOption({ label: type });
  await editor(page).getByRole('button', { name: '追加', exact: true }).click();
}

test.describe('曲面・球・曲線の制御点', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(PAGE);
    await expect(preview(page)).toBeVisible();
  });

  test('曲面のフォームで，ワイヤーフレームを選ぶと，編集中の図とプレビュー，どちらにも線が増える', async ({
    page,
  }) => {
    await editor(page).getByRole('button', { name: '放物面と座標軸', exact: true }).click();
    await editor(page)
      .getByRole('list', { name: 'オブジェクトの一覧' })
      .getByRole('button', { name: /曲面/u })
      .click();
    const editBefore = await preview(page).locator('path').count();
    const outputBefore = await outputPreview(page).locator('path').count();
    await editor(page).getByLabel('ワイヤーフレームを表示').check();
    await expect(async () => {
      expect(await preview(page).locator('path').count()).toBeGreaterThan(editBefore);
    }).toPass();
    await expect(async () => {
      expect(await outputPreview(page).locator('path').count()).toBeGreaterThan(outputBefore);
    }).toPass();
  });

  test('球のフォームにも，ワイヤーフレームの選択がある', async ({ page }) => {
    await editor(page).getByRole('button', { name: '球と座標軸', exact: true }).click();
    await editor(page)
      .getByRole('list', { name: 'オブジェクトの一覧' })
      .getByRole('button', { name: /球/u })
      .click();
    await expect(editor(page).getByLabel('ワイヤーフレームを表示')).toBeVisible();
  });

  test('ベジエ曲面のフォームには，ワイヤーフレームと，別に，制御点の網の選択もある', async ({
    page,
  }) => {
    await editor(page).getByRole('button', { name: 'ベジエ曲面', exact: true }).click();
    await editor(page)
      .getByRole('list', { name: 'オブジェクトの一覧' })
      .getByRole('button', { name: /曲面/u })
      .click();
    await expect(editor(page).getByLabel('ワイヤーフレームを表示')).toBeVisible();
    await expect(editor(page).getByLabel('制御点の網を表示')).toBeVisible();
  });

  test('ベジエ曲線を足すと，制御点で図を描ける', async ({ page }) => {
    const before = await preview(page).locator('path').count();
    await addObject(page, '曲線(ベジエ)');
    await expect(async () => {
      expect(await preview(page).locator('path').count()).toBeGreaterThan(before);
    }).toPass();
    const withDefault = await preview(page).innerHTML();
    await editor(page).getByLabel('制御点(JSON)').fill('[[0,0],[2,3],[4,0]]');
    await expect(async () => {
      expect(await preview(page).innerHTML()).not.toBe(withDefault);
    }).toPass();
    await expect(editor(page).getByRole('alert')).toHaveCount(0);
  });
});
