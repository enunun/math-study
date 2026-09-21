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
    await expect(page.getByRole('group', { name: /^定義abs-1（絶対値）$/u })).toBeVisible();
    await expect(page.getByRole('group', { name: /^補題abs-2$/u })).toBeVisible();
    await expect(page.getByRole('group', { name: /^定理abs-3（三角不等式）$/u })).toBeVisible();
  });

  test('pageIdに書いた短縮名が，ラベルと式番号に使われる', async ({ page }) => {
    await page.goto('dev/continuity/');
    await expect(page.getByRole('group', { name: /^定義cont-1（開球）$/u })).toBeVisible();
    await expect(
      page.getByRole('group', { name: /^定理cont-7（連続写像の特徴づけ）$/u }),
    ).toBeVisible();
    await expect(page.locator('#euclidean-norm')).toContainText('(cont-1)');
  });

  test('参照が，定義や定理のラベルのリンクになる', async ({ page }) => {
    await page.goto(PAGE);
    const links = page.locator(REF);
    await expect(links).toHaveText([
      '定義notation-1',
      '定理notation-2',
      '定理abs-3',
      '式(notation-1)',
      '式(abs-1)',
      '図notation-1',
    ]);
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
    const proof = page.locator('details.fold-proof', { hasText: '定理abs-3' });
    await proof.locator('summary').click();
    await proof.locator(REF).click();
    await expect(page).toHaveURL(/#abs-bounds$/u);
    await expect(page.locator('#abs-bounds')).toBeInViewport();
  });
});

test.describe('式番号', () => {
  test(
    String.raw`\labelのある式の右に，ページの識別子と連番の番号が表示される`,
    async ({ page }) => {
      await page.goto(PAGE);
      const equation = page.locator('#integral-square');
      await expect(equation).toHaveJSProperty('tagName', 'MJX-CONTAINER');
      await expect(equation).toContainText('(notation-1)');
    },
  );

  test('番号のない式には，番号が付かない', async ({ page }) => {
    await page.goto(PAGE);
    const plain = page.locator('mjx-container[display]', { hasNotText: '(notation-1)' }).first();
    await expect(plain).toBeVisible();
    await expect(plain).not.toContainText(/\([a-z]+-\d+\)/u);
  });

  test('式への参照が，「式(識別子-連番)」のリンクになる', async ({ page }) => {
    await page.goto(PAGE);
    const links = page.locator(REF);
    await expect(links.nth(3)).toHaveText('式(notation-1)');
    await expect(links.nth(3)).toHaveAttribute('href', '#integral-square');
    await expect(links.nth(4)).toHaveAttribute('href', /\/dev\/abs\/#abs-cases$/u);
  });

  test('ほかのページの式への参照を押すと，その式へ移る', async ({ page }) => {
    await page.goto(PAGE);
    await page.locator(REF).nth(4).click();
    await expect(page).toHaveURL(/\/dev\/abs\/#abs-cases$/u);
    await expect(page.locator('#abs-cases')).toBeInViewport();
    await expect(page.locator('#abs-cases')).toContainText('(abs-1)');
  });

  test('幅の狭い画面で，番号のある式が，ページを横にあふれさせない', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 800 });
    await page.goto(TARGET_PAGE);
    await expect(page.locator('#abs-cases')).toContainText('(abs-1)');
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
    );
    expect(overflow).toBeLessThanOrEqual(0);
  });

  test('折り畳みの中の参照を押すと，同じページの式へ移る', async ({ page }) => {
    await page.goto(TARGET_PAGE);
    // 最初の証明は，補題の証明である．
    const proof = page.locator('details.fold-proof').first();
    await proof.locator('summary').click();
    await proof.locator(REF).first().click();
    await expect(page).toHaveURL(/#abs-cases$/u);
    await expect(page.locator('#abs-cases')).toBeInViewport();
  });
});
