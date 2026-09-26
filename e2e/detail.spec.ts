import { readFileSync } from 'node:fs';
import path from 'node:path';

import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';

import { listRoutes, routeName } from './routes';

const PAGE = 'dev/notation/';
const SENTENCE = '実数の二乗は0以上である．';
const BODY = '実際，正の数の二乗は正であり，負の数の二乗も正である．0の二乗は0である．';
const TOTAL_PERIODS = 3;
const INNER_TEXT = { useInnerText: true };

const paragraph = (page: Page): Locator => page.locator('p:has(.detail)').first();
const countPeriods = (text: string): number => text.split('．').length - 1;

test.describe('補足(Detail)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(PAGE);
  });

  test('閉じているとき，文の後ろに「［…］」が付き，本文は見えない', async ({ page }) => {
    await expect(paragraph(page)).toHaveText(new RegExp(`^${SENTENCE}\\s*［…］$`, 'u'), INNER_TEXT);
    await expect(page.getByRole('button', { name: '補足', expanded: false })).toBeVisible();
    await expect(page.locator('.detail .detail-body').first()).toBeHidden();
  });

  test('句点は書き手が書いた1つだけで，コンポーネントは句点を足さない', async ({ page }) => {
    expect(countPeriods(await paragraph(page).innerText())).toBe(1);
    await page.getByRole('button', { name: '補足' }).first().click();
    expect(countPeriods(await paragraph(page).innerText())).toBe(TOTAL_PERIODS);
  });

  test('開くと，「である．実際，…」と続き，末尾が「［閉じる］」になる', async ({ page }) => {
    await page.getByRole('button', { name: '補足' }).first().click();
    await expect(paragraph(page)).toHaveText(
      new RegExp(`^${SENTENCE}${BODY}\\s*［閉じる］$`, 'u'),
      INNER_TEXT,
    );
    await expect(page.getByRole('button', { name: '補足', expanded: true })).toBeVisible();
  });

  test('クリックで開閉できる', async ({ page }) => {
    const detail = page.locator('.detail').first();
    const toggle = page.getByRole('button', { name: '補足' }).first();
    await toggle.click();
    await expect(detail).toHaveClass(/is-open/u);
    await toggle.click();
    await expect(detail).not.toHaveClass(/is-open/u);
  });

  test('キーボードで開閉できる', async ({ page }) => {
    const detail = page.locator('.detail').first();
    await page.getByRole('button', { name: '補足' }).first().focus();
    await page.keyboard.press('Enter');
    await expect(detail).toHaveClass(/is-open/u);
    await page.keyboard.press('Space');
    await expect(detail).not.toHaveClass(/is-open/u);
  });

  test('開いた本文は，地の文より文字が小さく，背景が付く', async ({ page }) => {
    await page.getByRole('button', { name: '補足' }).first().click();
    const styles = await page
      .locator('.detail .detail-body')
      .first()
      .evaluate((element) => {
        const body = getComputedStyle(element);
        const text = getComputedStyle(element.closest('p') ?? element);
        return {
          smaller:
            Number(body.fontSize.replace('px', '')) < Number(text.fontSize.replace('px', '')),
          background: body.backgroundColor !== text.backgroundColor,
        };
      });
    expect(styles).toEqual({ smaller: true, background: true });
  });

  test('切り替えボタンは，本文の後ろにある', async ({ page }) => {
    const classes = await page
      .locator('.detail')
      .first()
      .evaluate((element) => [...element.children].map((child) => child.classList.item(0)));
    expect(classes).toEqual(['detail-body', 'detail-toggle']);
  });

  test('印刷の間だけ開き，印刷のスタイルでは切り替えボタンが隠れる', async ({ page }) => {
    const detail = page.locator('.detail').first();
    await page.evaluate(() => {
      dispatchEvent(new Event('beforeprint'));
    });
    await expect(detail).toHaveClass(/is-open/u);
    await page.evaluate(() => {
      dispatchEvent(new Event('afterprint'));
    });
    await expect(detail).not.toHaveClass(/is-open/u);

    await page.emulateMedia({ media: 'print' });
    await expect(page.getByRole('button', { name: '補足' }).first()).toBeHidden();
  });

  test('本文の中を指すハッシュで開く', async ({ page }) => {
    const detail = page.locator('.detail').first();
    await detail.locator('.detail-body').evaluate((element) => {
      element.id = 'inside-detail';
      location.hash = '#inside-detail';
    });
    await expect(detail).toHaveClass(/is-open/u);
  });

  test('検索でハイライトされた語が本文に入ると開く', async ({ page }) => {
    await page.goto(`${PAGE}?pagefind-highlight=x`);
    const detail = page.locator('.detail').first();
    await expect(detail).not.toHaveClass(/is-open/u);
    await detail.locator('.detail-body').evaluate((element) => {
      const mark = document.createElement('mark');
      mark.dataset.pagefindHighlight = 'true';
      mark.textContent = '正';
      element.prepend(mark);
    });
    await expect(detail).toHaveClass(/is-open/u);
  });
});

test.describe('補足(Detail)，JavaScriptが無効なとき', () => {
  test.use({ javaScriptEnabled: false });

  test('本文が常に見え，切り替えボタンは見えない', async ({ page }) => {
    await page.goto(PAGE);
    await expect(paragraph(page)).toHaveText(new RegExp(`^${SENTENCE}${BODY}$`, 'u'), INNER_TEXT);
    await expect(page.getByRole('button', { name: '補足' })).toHaveCount(0);
  });
});

/** ビルドしたHTMLに補足を含むページ．ほかの折り畳みを使わないページも，ここに入る． */
const routesWithDetail = (): string[] =>
  listRoutes().filter((route) =>
    readFileSync(
      path.join('site/dist', route === '' || route.endsWith('/') ? `${route}index.html` : route),
      'utf8',
    ).includes('class="detail '),
  );

test.describe('補足(Detail)を含むすべてのページ', () => {
  for (const route of routesWithDetail()) {
    test(`${routeName(route)}の補足は，すべて開ける`, async ({ page }) => {
      await page.goto(route);
      // 証明などの折り畳みの中にある補足も押せるよう，先に折り畳みを開く．
      await page.evaluate(() => {
        for (const fold of document.querySelectorAll('details')) {
          fold.open = true;
        }
      });
      const details = page.locator('.detail');
      for (const detail of await details.all()) {
        await detail.locator(':scope > .detail-toggle').click();
        await expect(detail).toHaveClass(/is-open/u);
        await expect(detail.locator(':scope > .detail-body')).toBeVisible();
      }
    });
  }
});
