import { parseArgs } from 'node:util';

import { chromium } from '@playwright/test';
import type { Page } from '@playwright/test';

import { server } from './serve.ts';

const BASE_URL = 'http://127.0.0.1:4322/math-study/';
const USAGE = `使い方：mise run screenshot -- <ページのパス> <出力先.png> [オプション]
  --theme light|dark   表示のテーマ(既定はlight)
  --width <px>         ウィンドウの幅(既定は1100)
  --full               ページ全体を撮る
  --click <selector>   撮る前に押す要素(複数指定できる．順に押す)
  --clip <selector>    この要素だけを撮る
  --scroll <selector>  撮る前に，この要素までスクロールする
例：mise run screenshot -- dev/notation/ /tmp/notation.png --theme dark --click .detail-toggle
`;

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: {
    theme: { type: 'string', default: 'light' },
    width: { type: 'string', default: '1100' },
    full: { type: 'boolean', default: false },
    click: { type: 'string', multiple: true, default: [] },
    clip: { type: 'string' },
    scroll: { type: 'string' },
  },
});
const [pagePath, output] = positionals;

/** 指定された要素を，順に押す． */
async function clickInOrder(page: Page, selectors: readonly string[]): Promise<void> {
  const [first, ...rest] = selectors;
  if (first === undefined) {
    return;
  }
  await page.locator(first).first().click();
  await clickInOrder(page, rest);
}

/** ページ全体，または--clipで指定された要素を撮る． */
async function capture(page: Page, file: string): Promise<void> {
  if (values.clip === undefined) {
    await page.screenshot({ path: file, fullPage: values.full });
    return;
  }
  await page.locator(values.clip).first().screenshot({ path: file });
}

if (pagePath === undefined || output === undefined) {
  process.stderr.write(USAGE);
  process.exitCode = 1;
} else {
  const browser = await chromium.launch();
  const theme = values.theme === 'dark' ? 'dark' : 'light';
  const context = await browser.newContext({
    colorScheme: theme,
    viewport: { width: Number(values.width), height: 900 },
  });
  await context.addInitScript((value) => {
    localStorage.setItem('starlight-theme', value);
  }, theme);
  const page = await context.newPage();
  await page.goto(`${BASE_URL}${pagePath}`);
  await clickInOrder(page, values.click);
  if (values.scroll !== undefined) {
    await page.locator(values.scroll).first().scrollIntoViewIfNeeded();
  }
  await capture(page, output);
  await browser.close();
  process.stdout.write(`${output}\n`);
}
server.close();
