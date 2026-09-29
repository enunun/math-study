import { readFile } from 'node:fs/promises';

import { expect, test } from '@playwright/test';

import { loadSample } from './figure-editor-pickers';

const PAGE = 'tools/graphics/figure-reference/';

/** JSONの値の，項目`key`の値．オブジェクトでなければ`undefined`． */
function field(value: unknown, key: string): unknown {
  return typeof value === 'object' && value !== null
    ? Object.getOwnPropertyDescriptor(value, key)?.value
    : undefined;
}

/** スキーマの`object`に並ぶ，オブジェクトの種類． */
async function objectTypes(): Promise<string[]> {
  const text = await readFile('site/public/schema/scene.schema.json', 'utf8');
  const schema: unknown = JSON.parse(text);
  const choices = field(field(field(schema, '$defs'), 'object'), 'oneOf');
  return (Array.isArray(choices) ? choices : []).flatMap((choice: unknown) => {
    const ref = field(choice, '$ref');
    return typeof ref === 'string' ? [ref.slice(ref.lastIndexOf('/') + 1)] : [];
  });
}

test.describe('図のシーンのリファレンス', () => {
  test('スキーマのオブジェクトの種類ごとに，見出しと項目の表がある', async ({ page }) => {
    await page.goto(PAGE);
    const types = await objectTypes();
    expect(types).toContain('implicit_curve');
    for (const type of types) {
      const heading = page.locator(`h3#object-${type}`);
      await expect(heading, type).toHaveText(type);
      await expect(page.locator(`section:has(> h3#object-${type}) table`)).toHaveCount(1);
    }
  });

  test('狭い画面でも，ページは横にはみ出さない(表だけが横に送れる)', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 800 });
    await page.goto(PAGE);
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
    );
    expect(overflow).toBeLessThanOrEqual(0);
  });

  test('図の作成のフォームから，選んだ種類の節を開ける', async ({ page }) => {
    await page.goto('tools/graphics/figure-editor/');
    await loadSample(page, ['空間', '曲面', '円錐と切り口']);
    await page
      .getByRole('button', { name: /切り口「/u })
      .first()
      .click();
    await page.getByRole('link', { name: 'この種類の説明' }).click();
    await expect(page).toHaveURL(/figure-reference\/#object-cut$/u);
    await expect(page.locator('h3#object-cut')).toBeInViewport();
  });
});
