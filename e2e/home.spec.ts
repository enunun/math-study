import { expect, test } from '@playwright/test';

import { listRoutes } from './routes';

const ROUTES = listRoutes();
const RECENT_COUNT = 5;

// 公開するページは，カテゴリと小分類のディレクトリに置く．トップページにはカテゴリの見出しと小分類のカードを，
// カテゴリのページには記事の一覧を，サイドバーには入れ子のグループを並べる．
const CATEGORIES = [
  { name: '単発ネタ', directory: 'topics', sections: [['大学受験', 'exam']] },
  {
    name: '分野別',
    directory: 'fields',
    sections: [
      ['結晶学', 'crystallography'],
      ['フーリエ解析', 'fourier'],
      ['楕円関数と楕円曲線', 'elliptic'],
    ],
  },
  { name: 'グラフィックス', directory: 'graphics', sections: [] },
  {
    name: 'ツール',
    directory: 'tools',
    sections: [
      ['グラフィックス', 'graphics'],
      ['LaTeX', 'latex'],
    ],
  },
] as const;

/** ディレクトリの下の記事．カテゴリのページ自身は含めない． */
function articlesIn(directory: string): string[] {
  return ROUTES.filter((route) => route.startsWith(`${directory}/`) && route !== `${directory}/`);
}

// 著書は，カテゴリに属さない1ページで，新しい本を上に並べる．
const BOOKS = [
  '統計学は最強の学問ではない',
  '0から始める数理論理学入門',
  '構造化マークアップ志向のLaTeX入門',
  '記号論理から始める集合論超入門',
  '電磁気学を学ぶための物理数学',
] as const;

test.describe('トップページ', () => {
  test(`新着記事に，公開日の新しい順に${RECENT_COUNT}件を並べる`, async ({ page }) => {
    await page.goto('');
    const items = page.locator('.recent-articles li');
    await expect(items).toHaveCount(RECENT_COUNT);
    const dates = await items
      .locator('time')
      .evaluateAll((times) => times.map((time) => time.getAttribute('datetime') ?? ''));
    expect(dates).toEqual(dates.toSorted().toReversed());
    await items.first().locator('a').click();
    await expect(page.locator('h1')).toBeVisible();
  });

  for (const { name, directory, sections } of CATEGORIES) {
    test(`「${name}」の見出しの下のカードから，カテゴリのページを開ける`, async ({ page }) => {
      await page.goto('');
      await expect(page.locator('main h2', { hasText: name }).first()).toBeVisible();
      const card = sections.length === 0 ? `${name}の記事` : sections[0][0];
      await page
        .locator(`main h2#${directory} ~ .card-grid`)
        .first()
        .getByRole('link', { name: new RegExp(card, 'u') })
        .click();
      await expect(page).toHaveURL(new RegExp(`/${directory}/(#.*)?$`, 'u'));
      await expect(page.locator('h1')).toHaveText(name);
    });

    test(`「${name}」のページに，カテゴリのすべての記事を並べる`, async ({ page }) => {
      await page.goto(`${directory}/`);
      const articles = articlesIn(directory);
      expect(articles.length).toBeGreaterThan(0);
      for (const route of articles) {
        await expect(page.locator(`main .card-grid a[href$="/${route}"]`)).toHaveCount(1);
      }
      for (const [label, section] of sections) {
        await expect(page.locator(`main h2#${section}`)).toHaveText(label);
      }
    });

    test(`サイドバーの「${name}」に，小分類ごとに記事が並ぶ`, async ({ page }) => {
      const articles = articlesIn(directory);
      await page.goto(articles[0] ?? '');
      const group = page.locator('nav[aria-label="メイン"] details').filter({ hasText: name });
      for (const [label, section] of sections) {
        const nested = group.locator('details').filter({ hasText: label });
        for (const route of articlesIn(`${directory}/${section}`)) {
          await expect(nested.locator(`a[href$="/${route}"]`)).toHaveCount(1);
        }
      }
      for (const route of articles) {
        await expect(group.locator(`a[href$="/${route}"]`)).toHaveCount(1);
      }
      // カテゴリのページは，トップページから開き，サイドバーには出さない．
      await expect(group.locator(`a[href$="/${directory}/"]`)).toHaveCount(0);
    });
  }

  test('開発用のページの一覧から，すべての開発用のページを開ける', async ({ page }) => {
    await page.goto('');
    await page
      .locator('main')
      .getByRole('link', { name: /開発用のページの一覧/u })
      .click();
    await expect(page).toHaveURL(/\/dev\/$/u);
    // カードのリンクは相対パスなので，解決したURLで比べる．
    const links = await page
      .locator('main a')
      .evaluateAll((anchors) =>
        anchors.map((anchor) => new URL(anchor.getAttribute('href') ?? '', document.baseURI).href),
      );
    const pages = articlesIn('dev');
    expect(pages.length).toBeGreaterThan(0);
    for (const route of pages) {
      expect(links.filter((href) => href.endsWith(`/${route}`))).toHaveLength(1);
    }
  });

  test('開発用のページは，サイドバーに出ない', async ({ page }) => {
    await page.goto('dev/notation/');
    await expect(page.locator('nav[aria-label="メイン"] a[href*="/dev/"]')).toHaveCount(0);
  });

  test('「著書」の見出しの下とサイドバーから，著書のページを開ける', async ({ page }) => {
    await page.goto('');
    await expect(page.locator('main').getByRole('heading', { name: '著書' })).toBeVisible();
    await expect(page.locator('nav[aria-label="メイン"] a[href$="/books/"]')).toHaveText('著書');
    await page
      .locator('main')
      .getByRole('link', { name: /著書の紹介/u })
      .click();
    await expect(page).toHaveURL(/\/books\/$/u);
    await expect(page.locator('main h2')).toHaveText([...BOOKS]);
  });

  test('ページ下部に，「前へ」「次へ」のリンクを出さない', async ({ page }) => {
    await page.goto('graphics/figure-fill/');
    await expect(page.locator('.pagination-links a')).toHaveCount(0);
  });
});
