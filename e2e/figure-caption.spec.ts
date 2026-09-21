import { expect, test } from '@playwright/test';

const PAGE = 'dev/figure-algorithms/';

test.describe('図の番号と参照', () => {
  test('番号を付けた図は，図の下に，番号と説明文を持つ', async ({ page }) => {
    await page.goto(PAGE);
    const caption = page.locator('figure#fig-lines figcaption');
    await expect(caption).toContainText('図figalg-1');
    await expect(caption).toContainText('球と円柱と座標軸');
  });

  test('説明文の数式が，描画される', async ({ page }) => {
    await page.goto('dev/figure-fill/');
    await expect(page.locator('figure#fig-region figcaption mjx-container')).toHaveCount(2);
  });

  test('参照は，図の番号のリンクになり，押すと図へ移る', async ({ page }) => {
    await page.goto(PAGE);
    const link = page.getByRole('link', { name: '図figalg-1' }).first();
    await expect(link).toHaveAttribute('href', /#fig-lines$/u);
    await link.click();
    await expect(page).toHaveURL(/#fig-lines$/u);
    await expect(page.locator('figure#fig-lines')).toBeInViewport();
  });

  test('番号は，ページの中の文書の順に増える', async ({ page }) => {
    await page.goto(PAGE);
    const labels = await page.locator('figure figcaption strong').allTextContents();
    expect(labels.length).toBeGreaterThan(5);
    expect(labels).toEqual(labels.map((_, index) => `図figalg-${index + 1}`));
  });

  test('番号のない図には，見出しがない', async ({ page }) => {
    await page.goto('dev/figure-space/');
    await expect(page.locator('figure.figure figcaption')).toHaveCount(0);
  });
});
