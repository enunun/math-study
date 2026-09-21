import { AxeBuilder } from '@axe-core/playwright';
import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';

const PAGE = 'dev/figure-scene/';

const EMPTY_SCENE = JSON.stringify({
  version: '0.1.0',
  description: '空の図',
  view: { x: [0, 1], y: [0, 1], unit: { x: '1cm', y: '1cm' } },
  objects: [],
});

function input(page: Page): ReturnType<Page['getByLabel']> {
  return page.getByLabel('シーン(JSON)', { exact: true });
}

test.describe('図のシーンの確認', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(PAGE);
    // Wasmの読み込みと，最初の読み込みを待つ．
    await expect(page.getByText('シーンを読み込めた．')).toBeVisible();
  });

  test('最初の図を読み込み，版と説明とオブジェクトを表示する', async ({ page }) => {
    await expect(
      page.locator('dd', { hasText: 'y=sin x のグラフと，x軸の方向に平行移動した点線のグラフ' }),
    ).toBeVisible();
    const versions = page.getByRole('term');
    await expect(versions).toHaveText(['版', 'エンジンの版', '説明']);
    const rows = page.getByRole('table', { name: 'オブジェクト' }).getByRole('row');
    // 見出しの行と，7個のオブジェクト．
    await expect(rows).toHaveCount(8);
    await expect(rows.filter({ hasText: 'shifted_sine' })).toContainText('graph');
    await expect(rows.filter({ hasText: 'shift' }).first()).toBeVisible();
  });

  test('版と説明の，見出しと値は，同じ高さに並ぶ', async ({ page }) => {
    const terms = page.getByRole('term');
    const definitions = page.getByRole('definition');
    await expect(terms).toHaveCount(3);
    const gaps = await Promise.all(
      [0, 1, 2].map(async (index) => {
        const [term, definition] = await Promise.all([
          terms.nth(index).boundingBox(),
          definitions.nth(index).boundingBox(),
        ]);
        return Math.abs((term?.y ?? Number.NaN) - (definition?.y ?? Number.NaN));
      }),
    );
    // 上の余白が付くと，見出しと値が，16pxずれる．測れないときは，NaNになり，失敗する．
    expect(gaps.every((gap) => gap < 2)).toBe(true);
  });

  test('読み直したJSONを，既定値を補って表示する', async ({ page }) => {
    await page.locator('summary', { hasText: '読み直したJSON' }).click();
    const canonical = page.getByRole('region', { name: '読み直したJSON' });
    await expect(canonical).toContainText('"arrow": "stealth"');
    // スタイルは，省いた項目を書き出さず，指定した項目だけを書き出す．
    await expect(canonical).toContainText('"line": "dotted"');
    await expect(canonical).not.toContainText('"line": "solid"');
  });

  test('新しい版は，版の誤りとして，両方の版を示す', async ({ page }) => {
    await page.getByRole('button', { name: '新しい版' }).click();
    const alert = page.getByRole('alert');
    await expect(alert).toContainText('99.0.0');
    await expect(alert).toContainText('incompatible_version');
    await expect(input(page)).toHaveAttribute('aria-invalid', 'true');
    await expect(page.getByText('シーンを読み込めた．')).toHaveCount(0);
  });

  test('オブジェクトの誤りは，そのidと，元の説明を示す', async ({ page }) => {
    await page.getByRole('button', { name: '未知の項目' }).click();
    const alert = page.getByRole('alert');
    await expect(alert).toContainText('x_axis');
    await expect(alert).toContainText('colour');
  });

  test('idの重なりは，重なったidを示す', async ({ page }) => {
    await page.getByRole('button', { name: 'idの重なり' }).click();
    await expect(page.getByRole('alert')).toContainText('duplicate_id');
    await expect(page.getByRole('alert')).toContainText('sine');
  });

  test('範囲の誤りを示す', async ({ page }) => {
    await page.getByRole('button', { name: '範囲の誤り' }).click();
    await expect(page.getByRole('alert')).toContainText('invalid_range');
  });

  test('式の誤りは，項目と式の中の位置を示す', async ({ page }) => {
    await page.getByRole('button', { name: '式の誤り' }).click();
    const alert = page.getByRole('alert');
    await expect(alert).toContainText('expression');
    await expect(alert).toContainText('shifted_sine');
    await expect(alert).toContainText(/expr.*1番目の式/su);
    await expect(alert).toContainText(/\d+文字目/u);
  });

  test('読み込めたシーンの，TikZの出力を表示する', async ({ page }) => {
    await page.locator('summary', { hasText: 'TikZ' }).click();
    const tikz = page.getByRole('region', { name: 'TikZ' });
    await expect(tikz).toContainText(String.raw`\begin{tikzpicture}`);
    await expect(tikz).toContainText('-{Stealth}');
    // 元のシーンが，コメントとして埋め込まれている．
    await expect(tikz).toContainText('"id": "shifted_sine"');
  });

  test('JSONの構文の誤りは，行と列を示す', async ({ page }) => {
    await page.getByRole('button', { name: 'JSONの構文の誤り' }).click();
    await expect(page.getByRole('alert')).toContainText(/\d+行\d+列/u);
  });

  test('入力を書き換えると，結果が更新される', async ({ page }) => {
    await input(page).fill(EMPTY_SCENE);
    await expect(page.getByText('空の図', { exact: true })).toBeVisible();
    await expect(page.getByText('オブジェクトはない．')).toBeVisible();
    await expect(page.getByRole('table', { name: 'オブジェクト' })).toHaveCount(0);
  });

  test('見本のボタンで，入力欄が見本に置き換わる', async ({ page }) => {
    await page.getByRole('button', { name: '新しい版' }).click();
    await expect(input(page)).toHaveValue(/"version": "99\.0\.0"/u);
    await page.getByRole('button', { name: '最初の図' }).click();
    await expect(page.getByText('シーンを読み込めた．')).toBeVisible();
  });

  test('入力を空にすると，結果と誤りを消し，案内を出す', async ({ page }) => {
    await input(page).fill('');
    await expect(page.getByText('シーンのJSONを入力すると，結果が表示される．')).toBeVisible();
    await expect(page.getByRole('alert')).toHaveCount(0);
    await expect(page.getByText('シーンを読み込めた．')).toHaveCount(0);
  });

  for (const scheme of ['light', 'dark'] as const) {
    test(`結果と誤りの表示に，アクセシビリティの違反がない(${scheme})`, async ({ page }) => {
      await page.emulateMedia({ colorScheme: scheme });
      await page.locator('summary', { hasText: '読み直したJSON' }).click();
      const ok = await new AxeBuilder({ page }).analyze();
      expect(ok.violations.map(({ id }) => id)).toEqual([]);
      await page.getByRole('button', { name: '未知の項目' }).click();
      await expect(page.getByRole('alert')).toBeVisible();
      const failed = await new AxeBuilder({ page }).analyze();
      expect(failed.violations.map(({ id }) => id)).toEqual([]);
    });
  }

  test('読み込みでエラーが出ない', async ({ page }) => {
    const problems: string[] = [];
    page.on('console', (message) => {
      if (message.type() === 'error') {
        problems.push(message.text());
      }
    });
    page.on('pageerror', (error) => problems.push(String(error)));
    page.on('requestfailed', (request) => problems.push(request.url()));
    await page.reload();
    await expect(page.getByText('シーンを読み込めた．')).toBeVisible();
    expect(problems).toEqual([]);
  });
});
