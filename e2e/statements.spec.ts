import { expect, test } from '@playwright/test';

// ラベル，連番，参照のリンク先は，rehype-statements.test.tsとrehype-equations.test.tsで確かめる．
// ページをまたぐリンクの飛び先は，site.spec.tsのリンクの検査で確かめる．
// ここでは，ブラウザでしか分からない，画面の幅と，折り畳みの中からの移動だけを確かめる．
const PAGE = 'dev/abs/';
const REF = 'a.statement-ref';

test.describe('式番号と参照', () => {
  test('幅の狭い画面で，番号のある式が，ページを横にあふれさせない', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 800 });
    await page.goto(PAGE);
    await expect(page.locator('#abs-cases')).toContainText('(abs-1)');
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
    );
    expect(overflow).toBeLessThanOrEqual(0);
  });

  test('折り畳みの中の参照を押すと，同じページの式へ移る', async ({ page }) => {
    await page.goto(PAGE);
    // 最初の証明は，補題の証明である．
    const proof = page.locator('details.fold-proof').first();
    await proof.locator('summary').click();
    await proof.locator(REF).first().click();
    await expect(page).toHaveURL(/#abs-cases$/u);
    await expect(page.locator('#abs-cases')).toBeInViewport();
  });
});
