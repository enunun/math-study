import { AxeBuilder } from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

import { listRoutes, routeName } from './routes';

// 数式と図の多いページは，axeの検査に20秒以上かかる．既定の30秒では，並列で走らせると足りない．
const AXE_TIMEOUT_MS = 120_000;

const SCHEMES = ['light', 'dark'] as const;
const VIEWPORTS = {
  wide: { width: 1280, height: 800 },
  narrow: { width: 390, height: 800 },
} as const;

// すべてのページを，ライトとダーク，広い画面と狭い画面で，axeで検査する．違反は，1件でも失敗にする．
// 閉じた折り畳みの中も検査するため，開いてから検査する．
for (const scheme of SCHEMES) {
  for (const [size, viewport] of Object.entries(VIEWPORTS)) {
    test.describe(`アクセシビリティ(${scheme}，${size})`, () => {
      test.use({ colorScheme: scheme, viewport });
      test.setTimeout(AXE_TIMEOUT_MS);

      for (const route of listRoutes()) {
        test(`${routeName(route)}に，違反がない`, async ({ page }) => {
          await page.goto(route);
          await page.evaluate(async () => {
            await document.fonts.ready;
            for (const details of document.querySelectorAll('details')) {
              details.open = true;
            }
            for (const detail of document.querySelectorAll('.detail')) {
              detail.classList.add('is-open');
            }
          });
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
}
