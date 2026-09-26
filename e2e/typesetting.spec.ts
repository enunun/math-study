import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';

const PAGE = 'dev/continuity/';
// 本文の文字は16pxで，和欧文間の隙間は，その1/8である．
const AUTOSPACE = 2;

interface Gaps {
  before: number;
  after: number;
}

/**
 * 行内の数式の，前後の文字との隙間(px)を測る．
 * `beforeEnds`で終わる文字の直後にある数式を探し，前の文字と数式，数式と次の文字の間を測る．
 */
function gapsAround(page: Page, beforeEnds: string): Promise<Gaps> {
  return page.evaluate((ending) => {
    const containers = [...document.querySelectorAll('p mjx-container:not([display])')];
    const math = containers.find(
      (element) => element.previousSibling?.textContent?.endsWith(ending) === true,
    );
    const previous = math?.previousSibling;
    const next = math?.nextSibling;
    if (!math || !previous || !next) {
      throw new Error(`「${ending}」の直後の数式が見つからない．`);
    }
    const lastChar = document.createRange();
    lastChar.setStart(previous, (previous.textContent ?? '').length - 1);
    lastChar.setEnd(previous, (previous.textContent ?? '').length);
    const firstChar = document.createRange();
    firstChar.setStart(next, 0);
    firstChar.setEnd(next, 1);
    const box = math.getBoundingClientRect();
    return {
      before: box.left - lastChar.getBoundingClientRect().right,
      after: firstChar.getBoundingClientRect().left - box.right,
    };
  }, beforeEnds);
}

test.describe('和欧文間の隙間', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(PAGE);
    await page.evaluate(() => document.fonts.ready);
  });

  test('和文の間にある行内の数式の前後に，1/8emの隙間が入る', async ({ page }) => {
    const gaps = await gapsAround(page, '点');
    expect(gaps.before).toBeCloseTo(AUTOSPACE, 0);
    expect(gaps.after).toBeCloseTo(AUTOSPACE, 0);
  });

  test('本文の和文と欧文，数字の間に，1/8emの隙間が入る', async ({ page }) => {
    // 新しい段落で，text-autospaceを効かせた幅と，切った幅の差を測る．境目は，「あ|A」と「1|あ」の2か所である．
    const widths = await page.evaluate(() => {
      const host = document.querySelector('.sl-markdown-content');
      const measure = (autospace: string): number => {
        const paragraph = document.createElement('p');
        paragraph.textContent = 'あA1あ';
        paragraph.style.cssText = `width:max-content;text-autospace:${autospace}`;
        host?.append(paragraph);
        const { width } = paragraph.getBoundingClientRect();
        paragraph.remove();
        return width;
      };
      const inherited = document.createElement('p');
      host?.append(inherited);
      const value = getComputedStyle(inherited).textAutospace;
      inherited.remove();
      return { inherited: value, normal: measure('normal'), none: measure('no-autospace') };
    });
    expect(widths.inherited).toBe('normal');
    expect(widths.normal - widths.none).toBeCloseTo(AUTOSPACE * 2, 0);
  });

  test('コードブロックと数式の内側には，適用しない', async ({ page }) => {
    // 記事にはコードブロックがないため，コードブロックのある，記法のページで測る．
    await page.goto('dev/notation/');
    const values = await page.evaluate(() => ({
      pre: getComputedStyle(document.querySelector('pre') ?? document.body).textAutospace,
      math: getComputedStyle(document.querySelector('mjx-container') ?? document.body)
        .textAutospace,
    }));
    expect(values).toEqual({ pre: 'no-autospace', math: 'no-autospace' });
  });
});
