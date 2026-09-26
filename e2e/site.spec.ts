import { AxeBuilder } from '@axe-core/playwright';
import { expect, test } from '@playwright/test';
import type { APIRequestContext, BrowserContext, Page } from '@playwright/test';

import { listRoutes, routeName } from './routes';

const STATUS_OK = 200;

// 数式と図の多いページは，axeの検査に20秒以上かかる．既定の30秒では，並列で走らせると足りない．
const AXE_TIMEOUT_MS = 120_000;

const SCHEMES = ['light', 'dark'] as const;

/** ページを開く間に出た，コンソールのエラー，例外，失敗した要求，400番台以上の応答を集める． */
function watchProblems(page: Page): string[] {
  const problems: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') {
      problems.push(`console: ${message.text()}`);
    }
  });
  page.on('pageerror', (error) => problems.push(`pageerror: ${String(error)}`));
  page.on('requestfailed', (request) => problems.push(`requestfailed: ${request.url()}`));
  page.on('response', (response) => {
    if (response.status() >= 400) {
      problems.push(`${response.status()}: ${response.url()}`);
    }
  });
  return problems;
}

// すべてのページを，ライトとダークで開き，エラーが出ないことと，axeの違反がないことを確かめる．
// Wasmや，あとから組む数式のフォントの読み込みも待つため，通信が止むまで待つ．
// 閉じた折り畳みの中も検査するため，開いてから検査する．
// 狭い画面は，axeの結果がほぼ変わらないので検査しない．はみ出しは，各機能のテストで確かめる．
for (const scheme of SCHEMES) {
  test.describe(`すべてのページ(${scheme})`, () => {
    test.use({ colorScheme: scheme });
    test.setTimeout(AXE_TIMEOUT_MS);

    for (const route of listRoutes()) {
      test(`${routeName(route)}に，エラーとアクセシビリティの違反がない`, async ({ page }) => {
        const problems = watchProblems(page);
        await page.goto(route, { waitUntil: 'networkidle' });
        await page.evaluate(async () => {
          await document.fonts.ready;
          for (const details of document.querySelectorAll('details')) {
            details.open = true;
          }
          for (const detail of document.querySelectorAll('.detail')) {
            detail.classList.add('is-open');
          }
        });
        expect(problems).toEqual([]);
        const { violations } = await new AxeBuilder({ page }).analyze();
        const summary = violations.map(
          ({ id, impact, nodes }) =>
            `${id}(${impact}): ${nodes.map((node) => node.target.join(' ')).join(' | ')}`,
        );
        expect(summary).toEqual([]);
      });
    }
  });
}

/** すべてのページにある，このサイトの中のリンクの，絶対URLを集める． */
async function collectInternalLinks(context: BrowserContext, origin: string): Promise<string[]> {
  const hrefLists = await Promise.all(
    listRoutes().map(async (route) => {
      const page = await context.newPage();
      await page.goto(route);
      const hrefs = await page.$$eval('a[href]', (anchors) =>
        anchors.map((anchor) => new URL(anchor.getAttribute('href') ?? '', location.href).href),
      );
      await page.close();
      return hrefs;
    }),
  );
  return [...new Set(hrefLists.flat())].filter((href) => new URL(href).origin === origin);
}

/** ページのHTMLに，指定したidの要素が，あるかどうか． */
function hasId(html: string, id: string): boolean {
  const escaped = id.replaceAll(/[.*+?^${}()|[\]\\]/gu, String.raw`\$&`);
  return new RegExp(`\\bid=["']?${escaped}["'\\s>]`, 'u').test(html);
}

/** リンクの飛び先が，存在しないときの理由．存在するときはundefined． */
async function findProblem(request: APIRequestContext, href: string): Promise<string | undefined> {
  const url = new URL(href);
  const response = await request.get(`${url.origin}${url.pathname}${url.search}`);
  if (response.status() !== STATUS_OK) {
    return `${response.status()}: ${href}`;
  }
  const id = decodeURIComponent(url.hash.slice(1));
  if (id !== '' && !hasId(await response.text(), id)) {
    return `飛び先の要素がない: ${href}`;
  }
  return undefined;
}

test('内部リンクの飛び先が，すべて存在する', async ({ context, request, baseURL }) => {
  const links = await collectInternalLinks(context, new URL(baseURL ?? '').origin);
  expect(links.length).toBeGreaterThan(0);
  const problems = await Promise.all(links.map((href) => findProblem(request, href)));
  expect(problems.filter((problem) => problem !== undefined)).toEqual([]);
});
