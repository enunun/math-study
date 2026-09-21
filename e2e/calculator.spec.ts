import { AxeBuilder } from '@axe-core/playwright';
import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';

const PAGE = 'dev/calculator/';

function input(page: Page): ReturnType<Page['getByLabel']> {
  return page.getByLabel('式', { exact: true });
}

test.describe('多項式の計算機', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(PAGE);
    // Wasmの読み込みと，最初の計算を待つ．
    await expect(page.getByRole('heading', { name: '展開' })).toBeVisible();
  });

  test('最初の式を，展開して，数式で表示する', async ({ page }) => {
    await expect(page.getByText('x^2 + 2*x + 1', { exact: true })).toBeVisible();
    await expect(page.locator('.math-view mjx-container').first()).toBeVisible();
    await expect(page.getByRole('heading', { name: 'xによる偏微分' })).toBeVisible();
    await expect(page.getByText('2*x + 2', { exact: true })).toBeVisible();
  });

  test('式を入力し直すと，結果が更新される', async ({ page }) => {
    await input(page).fill('(x+1)(x-1)');
    await expect(page.getByText('x^2 - 1', { exact: true })).toBeVisible();
    await expect(page.getByText('x^2 + 2*x + 1', { exact: true })).toHaveCount(0);
  });

  test('複数の変数は，名前の順に，変数ごとに偏微分する', async ({ page }) => {
    await input(page).fill('(a+b)^2');
    await expect(page.getByRole('heading', { name: 'aによる偏微分' })).toBeVisible();
    await expect(page.getByRole('heading', { name: 'bによる偏微分' })).toBeVisible();
  });

  test('全角の記号で入力した式も，計算する', async ({ page }) => {
    await input(page).fill('（ｘ＋２）＾２');
    await expect(page.getByText('x^2 + 4*x + 4', { exact: true })).toBeVisible();
  });

  test('例のボタンで，入力欄に式が入り，結果が更新される', async ({ page }) => {
    await page.getByRole('button', { name: '(a+b)^3' }).click();
    await expect(input(page)).toHaveValue('(a+b)^3');
    await expect(page.getByText('a^3 + 3*a^2*b + 3*a*b^2 + b^3', { exact: true })).toBeVisible();
  });

  test('式に誤りがあると，説明と位置を表示し，入力欄を誤りの状態にする', async ({ page }) => {
    await input(page).fill('1 + * 2');
    const alert = page.getByRole('alert');
    await expect(alert).toContainText('ここに「*」は置けない');
    await expect(alert).toContainText('5文字目');
    await expect(alert.locator('mark')).toHaveText('*');
    await expect(input(page)).toHaveAttribute('aria-invalid', 'true');
    await expect(page.getByRole('heading', { name: '展開' })).toHaveCount(0);
  });

  test('式が途中で終わるときは，終わりの位置を示す', async ({ page }) => {
    await input(page).fill('(x+1');
    await expect(page.getByRole('alert')).toContainText('対応する閉じ括弧がない');
  });

  test('入力を空にすると，結果と誤りを消し，案内を出す', async ({ page }) => {
    await input(page).fill('');
    await expect(page.getByText('式を入力すると，結果が表示される．')).toBeVisible();
    await expect(page.getByRole('alert')).toHaveCount(0);
    await expect(page.getByRole('heading', { name: '展開' })).toHaveCount(0);
  });

  test('数式は，読み上げ用の平文のラベルを持ち，キーボードでフォーカスできる', async ({ page }) => {
    const math = page.locator('.math-view').first();
    await expect(math).toHaveAttribute('role', 'img');
    await expect(math).toHaveAttribute('aria-label', 'x^2 + 2*x + 1');
    await math.focus();
    await expect(math).toBeFocused();
  });

  test('大きすぎる結果は，誤りとして表示し，画面を止めない', async ({ page }) => {
    await input(page).fill('(x+y+z+1)^100');
    await expect(page.getByRole('alert')).toContainText('項が多すぎる');
    await input(page).fill('x+1');
    await expect(page.getByText('x + 1', { exact: true }).first()).toBeVisible();
  });

  for (const scheme of ['light', 'dark'] as const) {
    test(`結果と誤りの表示に，アクセシビリティの違反がない(${scheme})`, async ({ page }) => {
      await page.emulateMedia({ colorScheme: scheme });
      await expect(page.locator('.math-view mjx-container').first()).toBeVisible();
      const ok = await new AxeBuilder({ page }).analyze();
      expect(ok.violations.map(({ id }) => id)).toEqual([]);
      await input(page).fill('1 + * 2');
      await expect(page.getByRole('alert')).toBeVisible();
      const failed = await new AxeBuilder({ page }).analyze();
      expect(failed.violations.map(({ id }) => id)).toEqual([]);
    });
  }

  test('Starlightの本文の余白が，ブラウザで描いた数式の内側に及ばない', async ({ page }) => {
    // 隣り合う要素に付く1rem(16px)の上の余白が，上付き文字などを離すと，文字が切り取られて崩れる．
    await page.locator('.math-view mjx-container').first().waitFor();
    const spaced = await page.$$eval('.math-view mjx-container *', (elements) =>
      elements
        .filter((element) => getComputedStyle(element).marginTop === '16px')
        .map((element) => element.tagName),
    );
    expect(spaced).toEqual([]);
  });

  test('読み込みでエラーが出ない', async ({ page }) => {
    const problems: string[] = [];
    page.on('console', (message) => {
      if (message.type() === 'error') {
        problems.push(message.text());
      }
    });
    page.on('pageerror', (error) => problems.push(String(error)));
    page.on('requestfailed', (request) => problems.push(request.url()));
    page.on('response', (response) => {
      if (response.status() >= 400) {
        problems.push(`${response.status()} ${response.url()}`);
      }
    });
    await page.reload();
    await expect(page.locator('.math-view mjx-container').first()).toBeVisible();
    // フォントは，数式を組んだあとで要求されるため，読み込みが終わるまで待つ．
    await page.evaluate(() => document.fonts.ready);
    const loaded = await page.evaluate(
      () =>
        [...document.fonts].filter(
          (font) => font.family.startsWith('MJX') && font.status === 'loaded',
        ).length,
    );
    expect(loaded).toBeGreaterThan(0);
    expect(problems).toEqual([]);
  });
});
