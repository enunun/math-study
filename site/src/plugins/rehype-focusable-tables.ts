import type { Root } from 'hast';
import type { Plugin } from 'unified';
import { visit } from 'unist-util-visit';

/**
 * 表を，フォーカスできるようにする．
 * Starlightの表は，幅を超えると，表の中で横にスクロールする．
 * スクロールする領域がフォーカスできないと，キーボードだけでは，はみ出した列を読めない．
 */
const rehypeFocusableTables: Plugin<[], Root> = () => (tree) => {
  visit(tree, 'element', (node) => {
    if (node.tagName === 'table') {
      node.properties.tabIndex = 0;
    }
  });
};

export { rehypeFocusableTables };
