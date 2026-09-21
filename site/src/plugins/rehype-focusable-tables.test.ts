import type { Element, Root } from 'hast';
import { unified } from 'unified';
import { describe, expect, it } from 'vitest';

import { rehypeFocusableTables } from './rehype-focusable-tables';

function element(tagName: string, children: Element[] = []): Element {
  return { type: 'element', tagName, properties: {}, children };
}

describe('rehypeFocusableTables', () => {
  it('表にtabindexを付け，ほかの要素は変えない', async () => {
    const table = element('table');
    const paragraph = element('p');
    const tree: Root = { type: 'root', children: [element('div', [table, paragraph])] };
    await unified().use(rehypeFocusableTables).run(tree);
    expect(table.properties.tabIndex).toBe(0);
    expect(paragraph.properties.tabIndex).toBeUndefined();
  });
});
