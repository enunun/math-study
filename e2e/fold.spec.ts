import { expect, test } from '@playwright/test';

const PAGE = 'dev/notation/';
const FOLD = 'details.fold';
const HEADING_ID = '折り畳みの中の見出し';
const LINK_NAME = '折り畳みの中の見出しへ移動';

test.describe('折り畳み(Proof，Remark)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(PAGE);
  });

  test('open属性のあるものだけが，最初から開いている', async ({ page }) => {
    await expect(page.locator(`${FOLD}[open]`)).toHaveCount(1);
  });

  test('クリックで開閉できる', async ({ page }) => {
    const fold = page.locator(FOLD).first();
    await fold.locator('summary').click();
    await expect(fold).toHaveJSProperty('open', true);
    await fold.locator('summary').click();
    await expect(fold).toHaveJSProperty('open', false);
  });

  test('キーボードで開閉できる', async ({ page }) => {
    const fold = page.locator(FOLD).first();
    await fold.locator('summary').focus();
    await page.keyboard.press('Enter');
    await expect(fold).toHaveJSProperty('open', true);
    await page.keyboard.press('Space');
    await expect(fold).toHaveJSProperty('open', false);
  });

  test('折り畳みの中を指すリンクを押すと，その折り畳みが開く', async ({ page }) => {
    const heading = page.locator(`#${HEADING_ID}`);
    await expect(heading).toBeHidden();
    await page.getByRole('link', { name: LINK_NAME }).click();
    await expect(heading).toBeVisible();
  });

  test('印刷の間だけ，すべて開く', async ({ page }) => {
    await page.evaluate(() => {
      dispatchEvent(new Event('beforeprint'));
    });
    await expect(page.locator(`${FOLD}:not([open])`)).toHaveCount(0);
    await page.evaluate(() => {
      dispatchEvent(new Event('afterprint'));
    });
    await expect(page.locator(`${FOLD}[open]`)).toHaveCount(1);
  });

  test('入れ子の内側を指すハッシュで，外側も開く', async ({ page }) => {
    const outer = page.locator(FOLD).nth(3);
    const inner = outer.locator(FOLD);
    await inner.evaluate((element) => {
      element.id = 'inner-target';
      location.hash = '#inner-target';
    });
    await expect(outer).toHaveJSProperty('open', true);
    await expect(inner).toHaveJSProperty('open', true);
  });
});

test('ハッシュ付きのURLで開くと，その折り畳みが開いている', async ({ page }) => {
  await page.goto(`${PAGE}#${encodeURIComponent(HEADING_ID)}`);
  await expect(page.locator(`#${HEADING_ID}`)).toBeVisible();
});

test('不正なハッシュでも，例外が出ず，ハンドラが動く', async ({ page }) => {
  const errors: string[] = [];
  page.on('pageerror', (error) => errors.push(String(error)));
  await page.goto(`${PAGE}#%E0%A4%A`);
  await page.getByRole('link', { name: LINK_NAME }).click();
  await expect(page.locator(`#${HEADING_ID}`)).toBeVisible();
  expect(errors).toEqual([]);
});

test('検索でハイライトされた語が折り畳みの中に入ると，その折り畳みが開く', async ({ page }) => {
  await page.goto(`${PAGE}?pagefind-highlight=x`);
  const fold = page.locator(FOLD).last();
  await expect(fold).toHaveJSProperty('open', false);
  await fold.evaluate((element) => {
    const mark = document.createElement('mark');
    mark.dataset.pagefindHighlight = 'true';
    mark.textContent = 'かたつむり';
    element.querySelector('.fold-body p')?.prepend(mark);
  });
  await expect(fold).toHaveJSProperty('open', true);
});
