import { expect, test } from '@playwright/test';

const PAGE = 'dev/notation/';
const TARGET_PAGE = 'dev/abs/';
const REF = 'a.statement-ref';

test.describe('定義と定理', () => {
  test('ページの識別子と連番のラベルが，見出しに表示される', async ({ page }) => {
    await page.goto(PAGE);
    await expect(page.getByRole('group', { name: /^定義notation-1（偶数）$/u })).toBeVisible();
    await expect(page.getByRole('group', { name: /^定理notation-2$/u })).toBeVisible();
  });

  test('frontmatterのpageIdが，ページの識別子になる', async ({ page }) => {
    await page.goto(TARGET_PAGE);
    await expect(
      page.getByRole('group', { name: /^定義absolute-value-1（絶対値）$/u }),
    ).toBeVisible();
    await expect(page.getByRole('group', { name: /^補題absolute-value-2$/u })).toBeVisible();
    await expect(
      page.getByRole('group', { name: /^定理absolute-value-3（三角不等式）$/u }),
    ).toBeVisible();
  });

  test('参照が，定義や定理のラベルのリンクになる', async ({ page }) => {
    await page.goto(PAGE);
    const links = page.locator(REF);
    await expect(links).toHaveText(['定義notation-1', '定理notation-2', '定理absolute-value-3']);
    await expect(links.nth(0)).toHaveAttribute('href', '#even-number');
    await expect(links.nth(1)).toHaveAttribute('href', '#even-square');
    await expect(links.nth(2)).toHaveAttribute('href', /\/dev\/abs\/#triangle-inequality$/u);
  });

  test('ページをまたぐ参照を押すと，参照先の定理へ移る', async ({ page }) => {
    await page.goto(PAGE);
    await page.locator(REF).nth(2).click();
    await expect(page).toHaveURL(/\/dev\/abs\/#triangle-inequality$/u);
    await expect(page.locator('#triangle-inequality')).toBeInViewport();
  });

  test('証明の題名が，証明する定理のラベルになる', async ({ page }) => {
    await page.goto(PAGE);
    await expect(
      page.locator('details.fold-proof summary', { hasText: '定理notation-2' }),
    ).toBeVisible();
  });

  test('折り畳みの中の参照を押すと，同じページの参照先へ移る', async ({ page }) => {
    await page.goto(TARGET_PAGE);
    const proof = page.locator('details.fold-proof', { hasText: '定理absolute-value-3' });
    await proof.locator('summary').click();
    await proof.locator(REF).click();
    await expect(page).toHaveURL(/#abs-bounds$/u);
    await expect(page.locator('#abs-bounds')).toBeInViewport();
  });
});
