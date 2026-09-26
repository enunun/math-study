import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';

const PAGE = 'dev/continuity/';
// 本文の文字は16pxで，和欧文間の隙間は，その1/4(pLaTeXのxkanjiskipの既定)である．
const AUTOSPACE = 4;

interface Gaps {
  before: number;
  after: number;
}

/**
 * `selector`に合う要素のうち，`beforeEnds`で終わる文字の直後にあるものを探し，前後の文字との隙間(px)を測る．
 */
function gapsAround(page: Page, selector: string, beforeEnds: string): Promise<Gaps> {
  return page.evaluate(
    ([query, ending]) => {
      const targets = [...document.querySelectorAll(query)];
      const target = targets.find(
        (element) => element.previousSibling?.textContent?.endsWith(ending) === true,
      );
      const previous = target?.previousSibling;
      const next = target?.nextSibling;
      if (!target || !previous || !next) {
        throw new Error(`「${ending}」の直後の${query}が見つからない．`);
      }
      const lastChar = document.createRange();
      lastChar.setStart(previous, (previous.textContent ?? '').length - 1);
      lastChar.setEnd(previous, (previous.textContent ?? '').length);
      const firstChar = document.createRange();
      firstChar.setStart(next, 0);
      firstChar.setEnd(next, 1);
      // 包んだ要素の外枠ではなく，中の文字の端から測る．余白は，外枠の外にあるためである．
      const inner = document.createRange();
      inner.selectNodeContents(target);
      const box =
        target.tagName === 'SPAN' ? inner.getBoundingClientRect() : target.getBoundingClientRect();
      return {
        before: box.left - lastChar.getBoundingClientRect().right,
        after: firstChar.getBoundingClientRect().left - box.right,
      };
    },
    [selector, beforeEnds] as const,
  );
}

test.describe('和欧文間の隙間', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(PAGE);
    await page.evaluate(() => document.fonts.ready);
  });

  test('和文の間にある行内の数式の前後に，1/4emの隙間が入る', async ({ page }) => {
    const gaps = await gapsAround(page, 'p mjx-container:not([display])', '点');
    expect(gaps.before).toBeCloseTo(AUTOSPACE, 0);
    expect(gaps.after).toBeCloseTo(AUTOSPACE, 0);
  });

  test('本文の和文と欧文，数字，インラインコードの間に，1/4emの隙間が入る', async ({ page }) => {
    // 記事の本文には，和文に挟まれた欧文とコードが少ないため，記法のページで測る．
    await page.goto('dev/notation/');
    await page.evaluate(() => document.fonts.ready);
    const word = await gapsAround(
      page,
      '.sl-markdown-content span.autospace-before.autospace-after',
      'して',
    );
    expect(word.before).toBeCloseTo(AUTOSPACE, 0);
    expect(word.after).toBeCloseTo(AUTOSPACE, 0);
    const code = await gapsAround(
      page,
      '.sl-markdown-content code.autospace-before.autospace-after',
      'と',
    );
    expect(code.before).toBeCloseTo(AUTOSPACE, 0);
    expect(code.after).toBeCloseTo(AUTOSPACE, 0);
  });

  test('本文，コードブロック，数式の内側では，text-autospaceを切る', async ({ page }) => {
    // 本文の隙間はビルド時のクラスが入れるため，text-autospaceの隙間が重ならないようにする．
    await page.goto('dev/notation/');
    const selectors = {
      content: '.sl-markdown-content p',
      pre: 'pre',
      math: 'mjx-container',
      title: 'h1',
    };
    const values = await page.evaluate((queries) => {
      const entries = Object.entries(queries).map(([name, selector]): [string, string] => [
        name,
        getComputedStyle(document.querySelector(selector) ?? document.body).getPropertyValue(
          'text-autospace',
        ),
      ]);
      return Object.fromEntries(entries);
    }, selectors);
    expect(values).toEqual({
      content: 'no-autospace',
      pre: 'no-autospace',
      math: 'no-autospace',
      title: 'normal',
    });
  });
});
