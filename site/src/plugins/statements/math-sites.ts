import type { Element, Root } from 'hast';
import { visit } from 'unist-util-visit';

import type { EquationSource } from './equations';
import { walk } from './tree';
import type { TreeNode } from './tree';

/** hastの木にある数式．描画の前に，TeXと`id`を書き換えるため，要素を持つ． */
interface HastMathSite extends EquationSource {
  /** TeXを子に持つ`<code class="language-math">`． */
  code: Element;
  /** 別行立ての式を包む`<pre>`． */
  pre: Element | undefined;
}

/** 値(TeX)を持つmdastの節． */
interface ValueNode extends TreeNode {
  value?: string;
}

function hasClass(element: Element, name: string): boolean {
  const { className } = element.properties;
  return Array.isArray(className) && className.includes(name);
}

/** remark-mathが作る，`<code class="language-math math-inline|math-display">`を集める． */
function collectHastMathSites(tree: Root): HastMathSite[] {
  const sites: HastMathSite[] = [];
  visit(tree, 'element', (node, _index, parent) => {
    if (node.tagName !== 'code' || !hasClass(node, 'language-math')) {
      return;
    }
    const display = hasClass(node, 'math-display');
    const pre =
      display && parent?.type === 'element' && parent.tagName === 'pre' ? parent : undefined;
    sites.push({
      tex: node.children.map((child) => (child.type === 'text' ? child.value : '')).join(''),
      display,
      block: pre !== undefined,
      // 別行立ての式の位置は，包む<pre>にだけ付く．
      place: (pre ?? node).position?.start,
      code: node,
      pre,
    });
  });
  return sites;
}

/** remark-mathが作る，mdastの`math`(別行立て)と`inlineMath`(行内)を集める． */
function collectMdastMathSites(tree: TreeNode): EquationSource[] {
  const sites: EquationSource[] = [];
  walk(tree, (node) => {
    if (node.type === 'math' || node.type === 'inlineMath') {
      const block = node.type === 'math';
      sites.push({
        tex: (node as ValueNode).value ?? '',
        display: block,
        block,
        place: node.position?.start,
      });
    }
  });
  return sites;
}

export { collectHastMathSites, collectMdastMathSites };
export type { HastMathSite };
