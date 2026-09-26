import { expect, test } from '@playwright/test';

const PAGE = 'dev/notation/';
const NARROW = { width: 390, height: 800 };

test.describe('数式', () => {
  test('Starlightの本文の余白が，数式の内側に及ばない', async ({ page }) => {
    await page.goto(PAGE);
    // Starlightは，本文の中で隣り合う要素に，1rem(16px)の上の余白を付ける．
    // 数式の内側の分数や上付き文字が，離れて描画されないことを確かめる．
    const spaced = await page.$$eval('mjx-container *', (elements) =>
      elements
        .filter((element) => getComputedStyle(element).marginTop === '16px')
        .map((element) => element.tagName),
    );
    expect(spaced).toEqual([]);
  });

  test('フォントとCSSが，エラーなく読み込まれる', async ({ page }) => {
    const failed: string[] = [];
    const errors: string[] = [];
    page.on('response', (response) => {
      if (response.status() >= 400) {
        failed.push(response.url());
      }
    });
    page.on('pageerror', (error) => errors.push(String(error)));
    await page.goto(PAGE);
    await page.evaluate(() => document.fonts.ready);
    const loaded = await page.evaluate(
      () =>
        [...document.fonts].filter(
          (font) => font.family.startsWith('MJX') && font.status === 'loaded',
        ).length,
    );
    expect(loaded).toBeGreaterThan(0);
    expect(failed).toEqual([]);
    expect(errors).toEqual([]);
  });

  test('mathjax.cssのURLには，ビルドごとに変わる版の印が付き，古いCSSが使われ続けない', async ({
    page,
  }) => {
    await page.goto(PAGE);
    const href = await page
      .locator('link[rel="stylesheet"][href*="mathjax.css"]')
      .getAttribute('href');
    expect(href).toMatch(/\/mathjax\.css\?v=\d+$/u);
  });

  test('幅の狭い画面で，長い式は，ページを横にあふれさせず，式の中でスクロールする', async ({
    page,
  }) => {
    await page.setViewportSize(NARROW);
    await page.goto(PAGE);
    const pageOverflow = await page.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
    );
    expect(pageOverflow).toBeLessThanOrEqual(0);
    const scrollable = await page.$$eval('mjx-container[display]', (containers) =>
      containers.some((container) => container.scrollWidth > container.clientWidth),
    );
    expect(scrollable).toBe(true);
  });
});
