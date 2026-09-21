import path from 'node:path';

import { chromium } from '@playwright/test';
import type { Page } from '@playwright/test';

import { KINDS, OUT_DIR, SCALE } from './settings.ts';

const BASE_URL = 'http://127.0.0.1:4322/math-study/';

/** 種類ごとに，SVGの側で隠すものを見えなくするCSS． */
const HIDE_CSS = `
html[data-hide='paths'] .figure-label { visibility: hidden; }
html[data-hide='labels'] .figure-frame svg { visibility: hidden; }
`;

/** ページの各図の枠を，種類ごとに撮る．種類を切り替えて，全部の図を撮る，を繰り返す． */
async function captureKinds(tab: Page, names: string[], files: Map<string, string>): Promise<void> {
  const frames = tab.locator('.figure-frame');
  for (const kind of KINDS) {
    // eslint-disable-next-line no-await-in-loop -- 表示を切り替えては，撮る．順に行う．
    await tab.evaluate((value) => {
      document.documentElement.dataset.hide = value;
    }, kind);
    // eslint-disable-next-line no-await-in-loop -- 表示を切り替えては，撮る．順に行う．
    await Promise.all(
      names.map(async (name, index) => {
        const file = path.join(OUT_DIR, `${name}-${kind}-svg.png`);
        await frames.nth(index).screenshot({ path: file });
        files.set(`${name}-${kind}`, file);
      }),
    );
  }
}

/** サイトのページの各図の枠を，種類ごとに画像にする．`図の名前-種類`から，ファイル名への対応を返す． */
async function captureSvgs(pages: Map<string, string[]>): Promise<Map<string, string>> {
  const browser = await chromium.launch();
  const context = await browser.newContext({
    colorScheme: 'light',
    viewport: { width: 1400, height: 900 },
    deviceScaleFactor: SCALE,
  });
  const files = new Map<string, string>();
  for (const [page, names] of pages) {
    // eslint-disable-next-line no-await-in-loop -- ページごとに，順に開く．
    const tab = await context.newPage();
    // eslint-disable-next-line no-await-in-loop -- ページごとに，順に開く．
    await tab.goto(`${BASE_URL}${page}`);
    // eslint-disable-next-line no-await-in-loop -- ページごとに，順に開く．
    await tab.waitForSelector('.figure-label mjx-container');
    // eslint-disable-next-line no-await-in-loop -- ページごとに，順に開く．
    await tab.addStyleTag({ content: HIDE_CSS });
    // eslint-disable-next-line no-await-in-loop -- ページごとに，順に開く．
    await captureKinds(tab, names, files);
    // eslint-disable-next-line no-await-in-loop -- ページごとに，順に開く．
    await tab.close();
  }
  await browser.close();
  return files;
}

export { captureSvgs };
