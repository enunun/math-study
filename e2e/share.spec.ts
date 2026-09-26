import { expect, test } from '@playwright/test';

const SITE = 'https://enunun.github.io/math-study/';

test.describe('共有ボタン', () => {
  test('ページ下部に，Xへの共有リンクがあり，そのページのURLと題名を渡す', async ({ page }) => {
    await page.goto('dev/notation/');
    const share = page.getByRole('navigation', { name: 'このページを共有' });
    const x = new URL((await share.getByRole('link', { name: 'X' }).getAttribute('href')) ?? '');
    expect(x.origin + x.pathname).toBe('https://x.com/intent/post');
    expect(x.searchParams.get('url')).toBe(`${SITE}dev/notation/`);
    expect(x.searchParams.get('text')).toMatch(/ \| えぬちゃんらんど$/u);
  });

  test('ホームでは，サイト名とトップのURLを渡す', async ({ page }) => {
    await page.goto('');
    const link = page
      .getByRole('navigation', { name: 'このページを共有' })
      .getByRole('link', { name: 'X' });
    const x = new URL((await link.getAttribute('href')) ?? '');
    expect(x.searchParams.get('url')).toBe(SITE);
    expect(x.searchParams.get('text')).toBe('えぬちゃんらんど');
  });

  test('X，Bluesky，Facebook，LINE，はてなブックマークへのリンクを，新しいタブで開く', async ({
    page,
  }) => {
    await page.goto('dev/notation/');
    const links = page.getByRole('navigation', { name: 'このページを共有' }).getByRole('link');
    await expect(links).toHaveText(['X', 'Bluesky', 'Facebook', 'LINE', 'はてなブックマーク']);
    for (const link of await links.all()) {
      await expect(link).toHaveAttribute('target', '_blank');
    }
  });
});
