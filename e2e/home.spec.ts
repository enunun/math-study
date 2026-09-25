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

// 公開するページは，カテゴリごとに，トップページの見出しの下とサイドバーのグループに並べる．
const CATEGORIES = [
  {
    name: '単発ネタ',
    pages: [
      ['図を描く数学', 'topics/figure-algorithms/'],
      ['平面の図の塗りつぶし', 'topics/figure-fill/'],
    ],
  },
  {
    name: 'ツール',
    pages: [
      ['図の作成', 'tools/figure-editor/'],
      ['numbersetsパッケージ', 'tools/numbersets/'],
    ],
  },
] as const;

test.describe('トップページ', () => {
  for (const { name, pages } of CATEGORIES) {
    for (const [title, path] of pages) {
      test(`「${name}」の「${title}」へのリンクから，${path}を開ける`, async ({ page }) => {
        await page.goto('');
        await expect(page.locator('main').getByRole('heading', { name })).toBeVisible();
        // サイドバーにも同じ名前のリンクがあるので，本文のカードをたどる．
        await page
          .locator('main')
          .getByRole('link', { name: new RegExp(title, 'u') })
          .click();
        await expect(page).toHaveURL(new RegExp(`/${path}$`, 'u'));
        await expect(page.locator('h1')).toBeVisible();
      });
    }
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

  test('トップページにも，カテゴリのサイドバーがある', async ({ page }) => {
    await page.goto('');
    const sidebar = page.locator('nav[aria-label="メイン"]');
    for (const { name } of CATEGORIES) {
      await expect(sidebar.getByText(name, { exact: true })).toBeVisible();
    }
  });

  for (const { name, pages } of CATEGORIES) {
    test(`サイドバーの「${name}」に，カテゴリのページが並ぶ`, async ({ page }) => {
      await page.goto(pages[0][1]);
      const group = page.locator('nav[aria-label="メイン"] details').filter({ hasText: name });
      for (const [title, path] of pages) {
        await expect(group.locator(`a[href$="${path}"]`)).toHaveText(title);
      }
    });
  }

  test('ページ下部に，「前へ」「次へ」のリンクを出さない', async ({ page }) => {
    await page.goto('topics/figure-fill/');
    await expect(page.locator('.pagination-links a')).toHaveCount(0);
  });
});
