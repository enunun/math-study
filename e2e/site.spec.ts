import { expect, test } from '@playwright/test';
import type { BrowserContext, APIRequestContext } from '@playwright/test';

import { listRoutes, routeName } from './routes';

const STATUS_OK = 200;

test.describe('すべてのページ', () => {
  for (const route of listRoutes()) {
    test(`${routeName(route)}を開いても，エラーが出ない`, async ({ page }) => {
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
      await page.goto(route);
      await page.evaluate(() => document.fonts.ready);
      expect(problems).toEqual([]);
    });
  }
});

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
