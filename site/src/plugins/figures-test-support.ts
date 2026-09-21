import type { Element, Root } from 'hast';
import remarkMdx from 'remark-mdx';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import { unified } from 'unified';
import { visit } from 'unist-util-visit';
import { VFile } from 'vfile';

import { rehypeFigures } from './rehype-figures';
import type { Options } from './rehype-figures';

/** 単体テストで使う，MDXを読んで，`<Figure>`を図に置き換えるまでの変換． */
function createFigureTransformer(options: Options): (source: string) => Promise<Root> {
  const processor = unified()
    .use(remarkParse)
    .use(remarkMdx)
    .use(remarkRehype, { passThrough: ['mdxJsxFlowElement', 'mdxJsxTextElement'] })
    .use(rehypeFigures, options);
  return (source) => {
    const file = new VFile({ value: source, path: 'page.mdx' });
    return processor.run(processor.parse(file), file);
  };
}

/** 木の中の，指定した名前の要素を，文書の順にすべて集める． */
function elementsNamed(node: Root | Element, tagName: string): Element[] {
  const found: Element[] = [];
  visit(node, 'element', (element) => {
    if (element.tagName === tagName) {
      found.push(element);
    }
  });
  return found;
}

export { createFigureTransformer, elementsNamed };
