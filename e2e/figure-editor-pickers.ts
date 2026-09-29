import type { Locator, Page } from '@playwright/test';

/** 選択の欄の名前と，決めるボタンの文字． */
interface Picker {
  name: string;
  action: string;
}

const SAMPLES: Picker = { name: '見本', action: '読み込む' };
const TEMPLATES: Picker = { name: '部品', action: '挿入' };

/** 分類と項目を選び，ボタンを押す(見本と部品のテンプレートの選択は，同じ形である)． */
async function pick(
  picker: Locator,
  { name, action }: Picker,
  [category, label]: readonly [string, string],
): Promise<void> {
  await picker.getByLabel(`${name}の分類`, { exact: true }).selectOption({ label: category });
  await picker.getByLabel(name, { exact: true }).selectOption({ label });
  await picker.getByRole('button', { name: action, exact: true }).click();
}

/** 図の作成で，見本を，平面・空間，種別，見本の順に選んで読み込む． */
async function loadSample(
  page: Page,
  [tier, category, label]: readonly [string, string, string],
): Promise<void> {
  const picker = page.locator('.figure-editor').getByRole('group', { name: '見本', exact: true });
  await picker.getByLabel('見本の種類', { exact: true }).selectOption({ label: tier });
  await pick(picker, SAMPLES, [category, label]);
}

/** 図の作成で，部品のテンプレートを分類から選んで挿入する． */
async function insertTemplate(page: Page, category: string, label: string): Promise<void> {
  const picker = page
    .locator('.figure-editor')
    .getByRole('group', { name: 'テンプレート(部品)', exact: true });
  await pick(picker, TEMPLATES, [category, label]);
}

export { insertTemplate, loadSample };
