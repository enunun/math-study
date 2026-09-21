import remarkFrontmatter from 'remark-frontmatter';
import remarkMath from 'remark-math';
import remarkMdx from 'remark-mdx';
import remarkParse from 'remark-parse';
import { unified } from 'unified';
import { parse as parseYaml } from 'yaml';

import { findStatementNodes, resolvePageId, toInfo } from './collect';
import type { StatementInfo } from './collect';
import { findLabelledEquations, numberPageItems } from './equations';
import { collectMdastMathSites } from './math-sites';

/** 定義や定理，式を持つ，1つのページ． */
interface PageEntry {
  pageId: string;
  /** ファイルの絶対パス． */
  file: string;
  /** 定義や定理，式． */
  statements: StatementInfo[];
}

// 式の`$$…$$`は，MDXの式(`{…}`)と読まれないよう，remark-mathで先に読む．
const processor = unified()
  .use(remarkParse)
  .use(remarkFrontmatter, ['yaml'])
  .use(remarkMath)
  .use(remarkMdx);

/** YAMLのfrontmatterから，`pageId`を読む． */
function readDeclaredPageId(yaml: string): unknown {
  const data: unknown = parseYaml(yaml);
  return typeof data === 'object' && data !== null && 'pageId' in data ? data.pageId : undefined;
}

/** 先頭の節がfrontmatterのとき，その`pageId`を優先して，ページの識別子を決める． */
function readPageId(first: { type: string; value?: string } | undefined, file: string): string {
  const yaml = first?.type === 'yaml' ? first.value : undefined;
  return resolvePageId(yaml === undefined ? undefined : readDeclaredPageId(yaml), file);
}

/**
 * MDXのソースから，定義や定理と，`\label`のある式を集める．
 * どちらもないページ，読み込めないページは，undefinedにする．
 * 読み込めないページの誤りは，そのページのビルドで報告される．
 */
function scanSource(source: string, file: string): PageEntry | undefined {
  try {
    const tree = processor.parse(source);
    const nodes = findStatementNodes(tree);
    const labelled = findLabelledEquations(collectMdastMathSites(tree));
    if (nodes.length + labelled.length === 0) {
      return undefined;
    }
    const pageId = readPageId(tree.children[0], file);
    const { statements, equations } = numberPageItems(nodes, labelled, pageId);
    return {
      pageId,
      file,
      statements: [
        ...statements.map((statement) => toInfo(statement)),
        ...equations.map(({ info }) => info),
      ],
    };
  } catch {
    return undefined;
  }
}

export { scanSource };
export type { PageEntry };
