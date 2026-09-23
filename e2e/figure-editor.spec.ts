import { readFile } from 'node:fs/promises';

import { AxeBuilder } from '@axe-core/playwright';
import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';

const PAGE = 'figure-editor/';

function editor(page: Page): Locator {
  return page.locator('.figure-editor');
}

function preview(page: Page): Locator {
  return editor(page).locator('.fe-edit-preview svg');
}

function outputPreview(page: Page): Locator {
  return editor(page).locator('.fe-output-preview svg');
}

/** ボタンを押して，始まったダウンロードの内容を読む． */
async function downloaded(page: Page, name: string): Promise<{ file: string; text: string }> {
  const started = page.waitForEvent('download');
  await editor(page).getByRole('button', { name }).click();
  const download = await started;
  const path = await download.path();
  return { file: download.suggestedFilename(), text: await readFile(path, 'utf8') };
}

async function openTab(page: Page, name: string): Promise<void> {
  await editor(page)
    .getByRole('group', { name: '表示の切り替え' })
    .getByRole('button', { name })
    .click();
}

async function addObject(page: Page, type: string): Promise<void> {
  await editor(page).getByLabel('追加する種類').selectOption({ label: type });
  await editor(page).getByRole('button', { name: '追加', exact: true }).click();
}

test.describe('図の作成', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(PAGE);
    await expect(preview(page)).toBeVisible();
  });

  test('空の図に，座標軸とグラフを足すと，図に線が描かれる', async ({ page }) => {
    const before = await preview(page).locator('path').count();
    await addObject(page, '座標軸');
    await addObject(page, '関数のグラフ');
    await expect(
      editor(page).getByRole('list', { name: 'オブジェクトの一覧' }).getByRole('listitem'),
    ).toHaveCount(2);
    await expect(preview(page).locator('path')).not.toHaveCount(before);
    await expect(editor(page).getByRole('alert')).toHaveCount(0);
  });

  test('式を書き換えると，図が描き直される', async ({ page }) => {
    await addObject(page, '関数のグラフ');
    const before = await preview(page).innerHTML();
    await editor(page).getByLabel('式', { exact: true }).fill('x^2/4');
    await expect(async () => {
      expect(await preview(page).innerHTML()).not.toBe(before);
    }).toPass();
  });

  test('式に誤りがあると，理由と，該当のオブジェクトを選ぶボタンが出る', async ({ page }) => {
    await addObject(page, '関数のグラフ');
    await editor(page).getByLabel('式', { exact: true }).fill('sin(');
    const alert = editor(page).locator('.fe-error');
    await expect(alert).toBeVisible();
    await expect(alert.getByRole('button', { name: /を選ぶ/u })).toBeVisible();
    await editor(page).getByLabel('式', { exact: true }).fill('sin(x)');
    await expect(alert).toHaveCount(0);
  });

  test('見本を読み込むと，図が描かれ，TikZが得られる', async ({ page }) => {
    await editor(page).getByRole('button', { name: '球と座標軸', exact: true }).click();
    await expect(preview(page)).toBeVisible();
    await openTab(page, 'TikZ');
    await expect(editor(page).getByRole('textbox', { name: 'TikZ' })).toHaveValue(
      /\\begin\{tikzpicture\}/u,
    );
  });

  test('図の種類を空間にすると，方位角と仰角を設定できる', async ({ page }) => {
    await editor(page).getByLabel('図の種類').selectOption('space');
    await expect(editor(page).getByLabel('方位角(度)')).toBeVisible();
    await expect(editor(page).getByLabel('仰角(度)')).toBeVisible();
    await addObject(page, '球');
    await expect(editor(page).getByRole('alert')).toHaveCount(0);
    await editor(page).getByLabel('方位角(度)').fill('10');
    await expect(preview(page)).toBeVisible();
  });

  test('空間の図は，プレビューをドラッグすると，見る向きが変わる', async ({ page }) => {
    await editor(page).getByRole('button', { name: '球と座標軸', exact: true }).click();
    const azimuth = editor(page).getByLabel('方位角(度)');
    const elevation = editor(page).getByLabel('仰角(度)');
    const before = { azimuth: await azimuth.inputValue(), elevation: await elevation.inputValue() };
    const draggable = editor(page).locator('.fe-draggable');
    await draggable.scrollIntoViewIfNeeded();
    const box = await draggable.boundingBox();
    if (box === null) {
      throw new Error('ドラッグする領域が見つからない');
    }
    const center = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
    await page.mouse.move(center.x, center.y);
    await page.mouse.down();
    await page.mouse.move(center.x + 60, center.y + 30, { steps: 5 });
    await page.mouse.up();
    await expect(azimuth).not.toHaveValue(before.azimuth);
    await expect(elevation).not.toHaveValue(before.elevation);
  });

  test('平面の図には，ドラッグの案内も，ドラッグできる領域もない', async ({ page }) => {
    await editor(page).getByRole('button', { name: '関数のグラフ', exact: true }).click();
    await expect(editor(page).locator('.fe-draggable')).toHaveCount(0);
    await expect(editor(page).getByText('ドラッグすると')).toHaveCount(0);
  });

  test('編集中の図とプレビューが，別々に表示される', async ({ page }) => {
    await addObject(page, '座標軸');
    await expect(editor(page).getByRole('heading', { name: '編集中の図' })).toBeVisible();
    await expect(
      editor(page).getByRole('heading', { name: 'プレビュー', exact: true }),
    ).toBeVisible();
    await expect(outputPreview(page)).toBeVisible();
  });

  test('編集中の図では，座標軸を補助のスタイル(灰色の点線)で描き，プレビューでは元のスタイルのままにする', async ({
    page,
  }) => {
    await addObject(page, '座標軸');
    const editStroke = await preview(page).locator('path').first().getAttribute('stroke');
    const outputStroke = await outputPreview(page).locator('path').first().getAttribute('stroke');
    expect(editStroke).toContain('gray');
    expect(outputStroke).not.toContain('gray');
  });

  test('JSONを書き出して，読み込み直すと，同じ図になる', async ({ page }) => {
    await editor(page).getByRole('button', { name: 'ベクトルの和', exact: true }).click();
    const exported = await downloaded(page, 'JSONを書き出す');
    expect(exported.file).toBe('figure.json');
    expect(JSON.parse(exported.text)).toMatchObject({ version: '0.1.0' });

    await editor(page).getByRole('button', { name: '空の図' }).click();
    await editor(page)
      .getByLabel('読み込むJSONのファイル')
      .setInputFiles({
        name: 'figure.json',
        mimeType: 'application/json',
        buffer: Buffer.from(exported.text),
      });
    await expect(editor(page).locator('.fe-message')).toContainText('読み込んだ');
    const again = await downloaded(page, 'JSONを書き出す');
    expect(again.text).toBe(exported.text);
  });

  test('読めないJSONのファイルは，理由を示して断る', async ({ page }) => {
    await editor(page)
      .getByLabel('読み込むJSONのファイル')
      .setInputFiles({
        name: 'broken.json',
        mimeType: 'application/json',
        buffer: Buffer.from('{'),
      });
    await expect(editor(page).locator('.fe-message')).toContainText('読めない');
  });

  test('TikZの断片と，単体の文書を書き出せる', async ({ page }) => {
    await editor(page).getByRole('button', { name: '関数のグラフ', exact: true }).click();
    await expect(editor(page).getByRole('button', { name: /TikZを書き出す/u })).toBeEnabled();
    const fragment = await downloaded(page, 'TikZを書き出す(.tikz)');
    expect(fragment.file).toBe('figure.tikz');
    expect(fragment.text).toContain(String.raw`\begin{tikzpicture}`);
    const document = await downloaded(page, '単体の文書を書き出す(.tex)');
    expect(document.file).toBe('figure.tex');
    expect(document.text).toContain(String.raw`\documentclass`);
    expect(document.text).toContain(fragment.text.trim());
  });

  test('JSONの欄を直接書き換えると，図に反映される．読めない間は，理由を示す', async ({ page }) => {
    await openTab(page, 'JSON');
    const box = editor(page).getByRole('textbox', { name: 'シーン(JSON)' });
    await box.fill('{');
    await expect(editor(page).getByText('JSONを読めない')).toBeVisible();
    await box.fill(
      JSON.stringify({
        version: '0.1.0',
        description: '点',
        view: { x: [-2, 2], y: [-2, 2], unit: { x: '1cm', y: '1cm' } },
        objects: [{ id: 'p', type: 'point', at: [1, 1], dot: true }],
      }),
    );
    await expect(editor(page).getByText('JSONを読めない')).toHaveCount(0);
    await openTab(page, 'フォーム');
    await expect(editor(page).getByRole('button', { name: /点「p」/u })).toBeVisible();
  });

  test('編集中の図は，開き直しても残る', async ({ page }) => {
    await addObject(page, '点');
    await page.reload();
    await expect(editor(page).getByRole('button', { name: /点「/u })).toBeVisible();
  });

  test('オブジェクトを並べ替え，削除できる', async ({ page }) => {
    await addObject(page, '点');
    await addObject(page, '点');
    const items = editor(page)
      .getByRole('list', { name: 'オブジェクトの一覧' })
      .getByRole('listitem');
    await expect(items).toHaveCount(2);
    const first = (await items.first().getByRole('button').first().textContent()) ?? '';
    await items
      .first()
      .getByRole('button', { name: /後ろへ/u })
      .click();
    await expect(items.last().getByRole('button').first()).toHaveText(first);
    await items
      .last()
      .getByRole('button', { name: /削除/u })
      .click();
    await expect(items).toHaveCount(1);
  });

  test('見本「フラクタル」は，誤りなく描ける', async ({ page }) => {
    // 見本のボタンは，選んだJSON文字列をparseDraftへ渡すだけで，中身に応じた分岐はない(toolbar.tsx)．
    // 各見本の図としての正しさは，サイトのビルドと，site/src/figure/scene-schema.test.tsで確かめているので，
    // ここでは，読み込みの仕組みが実際に動くことだけを，複雑な見本の1つで確かめる．
    await editor(page).getByRole('button', { name: 'フラクタル', exact: true }).click();
    await expect(preview(page)).toBeVisible();
    await expect(editor(page).getByRole('alert')).toHaveCount(0);
  });

  test('部品のテンプレート「正六角形」を挿入すると，頂点と辺のオブジェクトが増える', async ({
    page,
  }) => {
    // 生成する頂点・辺の正しさ(数，辺の長さなど)は，site/src/figure-editor/regular-shapes.test.tsと
    // templates.test.tsで確かめているので，ここでは，選んで挿入する操作が実際に図に反映されることだけを確かめる．
    const HEXAGON_OBJECT_COUNT = 12;
    const items = editor(page)
      .getByRole('list', { name: 'オブジェクトの一覧' })
      .getByRole('listitem');
    const before = await items.count();
    const picker = editor(page).locator('.fe-template-picker').filter({ hasText: '正多角形' });
    await picker.getByLabel('正多角形').selectOption({ label: '正六角形' });
    await picker.getByRole('button', { name: '挿入' }).click();
    await expect(items).toHaveCount(before + HEXAGON_OBJECT_COUNT);
    await expect(editor(page).getByRole('alert')).toHaveCount(0);
  });

  test('図全体のテンプレート「メビウスの帯」は，誤りなく描ける', async ({ page }) => {
    await editor(page).getByRole('button', { name: 'メビウスの帯', exact: true }).click();
    await expect(preview(page)).toBeVisible();
    await expect(editor(page).getByRole('alert')).toHaveCount(0);
  });

  for (const tab of ['フォーム', 'JSON', 'TikZ']) {
    test(`アクセシビリティ：${tab}の表示に，違反がない`, async ({ page }) => {
      await editor(page).getByRole('button', { name: '円錐と切り口', exact: true }).click();
      await expect(preview(page)).toBeVisible();
      await openTab(page, tab);
      const { violations } = await new AxeBuilder({ page }).analyze();
      expect(violations).toEqual([]);
    });
  }
});
