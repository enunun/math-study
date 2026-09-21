import { expect, test } from '@playwright/test';

// サイドバーと検索に出さない開発用のページは，トップページから，リンクをたどれるようにする．
const DEV_PAGES = [
  ['仕組みの解説', 'dev/internals/'],
  ['記法とテスト', 'dev/notation/'],
  ['図のシーンの確認', 'dev/figure-scene/'],
  ['最初の図', 'dev/figure-first/'],
  ['空間の図', 'dev/figure-space/'],
  ['記事の見本', 'dev/continuity/'],
  ['参照先の見本', 'dev/abs/'],
] as const;

const ARTICLES = [['図を描く数学', 'dev/figure-algorithms/']] as const;

test.describe('トップページ', () => {
  for (const [title, path] of ARTICLES) {
    test(`解説「${title}」へのリンクから，${path}を開ける`, async ({ page }) => {
      await page.goto('');
      await expect(page.getByRole('heading', { name: '解説' })).toBeVisible();
      await page.getByRole('link', { name: new RegExp(title, 'u') }).click();
      await expect(page).toHaveURL(new RegExp(`/${path}$`, 'u'));
      await expect(page.locator('h1')).toBeVisible();
    });
  }

  for (const [title, path] of DEV_PAGES) {
    test(`「${title}」へのリンクから，${path}を開ける`, async ({ page }) => {
      await page.goto('');
      await page.getByRole('link', { name: new RegExp(title, 'u') }).click();
      await expect(page).toHaveURL(new RegExp(`/${path}$`, 'u'));
      await expect(page.locator('h1')).toBeVisible();
    });
  }

  test('開発用のページは，サイドバーに出ない', async ({ page }) => {
    await page.goto('dev/notation/');
    await expect(page.locator('nav[aria-label="メイン"] a[href$="dev/continuity/"]')).toHaveCount(
      0,
    );
  });
});
