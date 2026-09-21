import type { Root } from 'hast';
import remarkMath from 'remark-math';
import remarkMdx from 'remark-mdx';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import { unified } from 'unified';
import type { VFile } from 'vfile';

import { rehypeStatements } from '../rehype-statements';
import type { Options } from '../rehype-statements';

/** 単体テストで使う，MDXを読んで番号を付けるまでの変換．ビルドと同じく，数式はremark-mathで読む． */
function createTransformer(options: Options): (file: VFile) => Promise<Root> {
  const processor = unified()
    .use(remarkParse)
    .use(remarkMath)
    .use(remarkMdx)
    .use(remarkRehype, { passThrough: ['mdxJsxFlowElement', 'mdxJsxTextElement'] })
    .use(rehypeStatements, options);
  return (file) => processor.run(processor.parse(file), file);
}

export { createTransformer };
