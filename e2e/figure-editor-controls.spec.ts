import { AxeBuilder } from '@axe-core/playwright';
import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';

const PAGE = 'tools/graphics/figure-editor/';

function editor(page: Page): Locator {
  return page.locator('.figure-editor');
}

function preview(page: Page): Locator {
  return editor(page).locator('.fe-edit-preview svg');
}

function outputPreview(page: Page): Locator {
  return editor(page).locator('.fe-output-preview svg');
}

async function addObject(page: Page, type: string): Promise<void> {
  await editor(page).getByLabel('追加する種類').selectOption({ label: type });
  await editor(page).getByRole('button', { name: '追加', exact: true }).click();
}

test.describe('曲面・球・曲線の制御点', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(PAGE);
    await expect(preview(page)).toBeVisible();
  });

  test('見本の曲面はワイヤーフレームがオンで，外すと，編集中の図とプレビュー，どちらからも線が減る', async ({
    page,
  }) => {
    await editor(page).getByRole('button', { name: '放物面と座標軸', exact: true }).click();
    await editor(page)
      .getByRole('list', { name: 'オブジェクトの一覧' })
      .getByRole('button', { name: /曲面/u })
      .click();
    const toggle = editor(page).getByLabel('ワイヤーフレームを表示');
    await expect(toggle).toBeChecked();
    await expect(editor(page).getByLabel('縁を描く')).toBeChecked();
    const editBefore = await preview(page).locator('path').count();
    const outputBefore = await outputPreview(page).locator('path').count();
    await toggle.uncheck();
    await expect(async () => {
      expect(await preview(page).locator('path').count()).toBeLessThan(editBefore);
    }).toPass();
    await expect(async () => {
      expect(await outputPreview(page).locator('path').count()).toBeLessThan(outputBefore);
    }).toPass();
  });

  test('球のフォームにも，ワイヤーフレームの選択がある', async ({ page }) => {
    await editor(page).getByRole('button', { name: '球と座標軸', exact: true }).click();
    await editor(page)
      .getByRole('list', { name: 'オブジェクトの一覧' })
      .getByRole('button', { name: /球/u })
      .click();
    await expect(editor(page).getByLabel('ワイヤーフレームを表示')).toBeVisible();
  });

  test('Bézier曲面のフォームには，ワイヤーフレームと，別に，制御点の網の選択もある', async ({
    page,
  }) => {
    await editor(page).getByRole('button', { name: '座標軸(空間)', exact: true }).click();
    const picker = editor(page).locator('.fe-template-picker').filter({ hasText: 'Bézier曲面' });
    await picker.getByLabel('Bézier曲面').selectOption({ label: '双3次Bézier曲面(制御点4×4個)' });
    await picker.getByRole('button', { name: '挿入' }).click();
    await editor(page)
      .getByRole('list', { name: 'オブジェクトの一覧' })
      .getByRole('button', { name: /曲面/u })
      .click();
    await expect(editor(page).getByLabel('ワイヤーフレームを表示')).toBeVisible();
    await expect(editor(page).getByLabel('制御点の網を表示')).toBeVisible();
  });

  test('Bézier曲線を足すと，制御点で図を描ける', async ({ page }) => {
    const before = await preview(page).locator('path').count();
    await addObject(page, '曲線(Bézier)');
    await expect(async () => {
      expect(await preview(page).locator('path').count()).toBeGreaterThan(before);
    }).toPass();
    const withDefault = await preview(page).innerHTML();
    await editor(page).getByLabel('制御点(JSON)').fill('[[0,0],[2,3],[4,0]]');
    await expect(async () => {
      expect(await preview(page).innerHTML()).not.toBe(withDefault);
    }).toPass();
    await expect(editor(page).getByRole('alert')).toHaveCount(0);
  });

  test('スプライン曲線を足すと，通る点で図を描ける', async ({ page }) => {
    const before = await preview(page).locator('path').count();
    await addObject(page, '曲線(スプライン)');
    await expect(async () => {
      expect(await preview(page).locator('path').count()).toBeGreaterThan(before);
    }).toPass();
    const withDefault = await preview(page).innerHTML();
    await editor(page).getByLabel('通る点(JSON)').fill('[[0,0],[2,3],[4,0],[5,-1]]');
    await expect(async () => {
      expect(await preview(page).innerHTML()).not.toBe(withDefault);
    }).toPass();
    await expect(editor(page).getByRole('alert')).toHaveCount(0);
  });

  test('フラクタルは_深さを変えると図形の数が変わる', async ({ page }) => {
    await addObject(page, 'フラクタル');
    const withDefaultDepth = await preview(page).locator('path').count();
    expect(withDefaultDepth).toBeGreaterThan(0);
    await editor(page).getByLabel('再帰の深さ').fill('2');
    await expect(async () => {
      const count = await preview(page).locator('path').count();
      expect(count).toBeGreaterThan(0);
      expect(count).toBeLessThan(withDefaultDepth);
    }).toPass();
    await expect(editor(page).getByRole('alert')).toHaveCount(0);
  });

  test('変換の手順を選んで足すと，正多角形が回り，JSONに手順が入る', async ({ page }) => {
    await addObject(page, '正多角形');
    const before = await outputPreview(page).innerHTML();
    const transform = editor(page).getByRole('group', { name: '変換' });
    await transform
      .getByRole('combobox', { name: '手順', exact: true })
      .selectOption({ label: '回転(原点のまわり)' });
    await transform.getByRole('button', { name: '手順を足す' }).click();
    await expect(transform.getByLabel('手順(JSON，上から順に施す)')).toHaveValue(/"rotate":90/u);
    await expect(async () => {
      expect(await outputPreview(page).innerHTML()).not.toBe(before);
    }).toPass();
    await expect(editor(page).getByRole('alert')).toHaveCount(0);
  });

  test('接線は，グラフを選んで接する点を決めると描ける', async ({ page }) => {
    await editor(page).getByRole('button', { name: '関数のグラフ', exact: true }).click();
    const before = await preview(page).locator('path').count();
    await addObject(page, '接線');
    await expect(async () => {
      expect(await preview(page).locator('path').count()).toBeGreaterThan(before);
    }).toPass();
    await expect(editor(page).getByRole('alert')).toHaveCount(0);
  });

  test('接平面は，曲面を選んで接する点を決めると描ける', async ({ page }) => {
    await editor(page).getByRole('button', { name: '放物面と座標軸', exact: true }).click();
    const before = await preview(page).locator('path').count();
    await addObject(page, '接平面');
    // 既定の接する点(原点)は，このサンプルの極座標では特異点になるので，ずらす．
    const at = editor(page).getByRole('group', { name: '接する点(変数の値)' });
    await at.getByLabel('1番目').fill('1');
    await expect(async () => {
      expect(await preview(page).locator('path').count()).toBeGreaterThan(before);
    }).toPass();
    await expect(editor(page).getByRole('alert')).toHaveCount(0);
  });
});

test.describe('媒介変数のスライダー', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(PAGE);
    await expect(preview(page)).toBeVisible();
  });

  test('範囲のある媒介変数はスライダーになり，動かすと図と値が変わり，再生すると値が動き続ける', async ({
    page,
  }) => {
    await editor(page).getByRole('button', { name: 'JSON', exact: true }).click();
    await editor(page)
      .getByRole('textbox', { name: 'シーン(JSON)' })
      .fill(
        JSON.stringify({
          version: '0.1.0',
          description: '動く点',
          view: { x: [-1, 3], y: [-1, 1], unit: { x: '1cm', y: '1cm' } },
          objects: [
            { id: 't', type: 'parameter', value: 0, range: [0, 2] },
            { id: 'p', type: 'point', at: ['t', 0], dot: true },
          ],
        }),
      );
    const sliders = editor(page).getByRole('group', { name: '媒介変数のスライダー' });
    const slider = sliders.getByRole('slider', { name: 't' });
    await expect(slider).toHaveValue('0');
    const before = await outputPreview(page).innerHTML();
    await slider.fill('1.5');
    await expect(sliders.locator('output')).toHaveText('1.5');
    await expect(async () => {
      expect(await outputPreview(page).innerHTML()).not.toBe(before);
    }).toPass();

    await sliders.getByRole('button', { name: 'tを再生する' }).click();
    const playing = sliders.getByRole('button', { name: 'tを止める' });
    await expect(playing).toHaveAttribute('aria-pressed', 'true');
    const first = await sliders.locator('output').textContent();
    await expect(async () => {
      expect(await sliders.locator('output').textContent()).not.toBe(first);
    }).toPass();
    await playing.click();
    const { violations } = await new AxeBuilder({ page }).include('.fe-edit-preview').analyze();
    expect(violations).toEqual([]);
    const stopped = await sliders.locator('output').textContent();
    await page.waitForTimeout(300);
    await expect(sliders.locator('output')).toHaveText(stopped ?? '');
    await expect(editor(page).getByRole('alert')).toHaveCount(0);
  });
});

test.describe('ラベル', () => {
  test(
    String.raw`見本「Ewald球(断面)」のラベルの，拡張のマクロ(\boldsymbol)も描画される`,
    async ({ page }) => {
      // ブラウザのMathJaxは，拡張をその場で読み込む．読み込めないと，式の文字列がそのまま残る．
      await page.goto(PAGE);
      await editor(page).getByRole('button', { name: 'Ewald球(断面)', exact: true }).click();
      const labels = editor(page).locator('.fe-edit-preview .inline-math');
      await expect(labels.first()).toBeVisible();
      const count = await labels.count();
      await expect(labels.filter({ has: page.locator('mjx-container') })).toHaveCount(count);
    },
  );
});
