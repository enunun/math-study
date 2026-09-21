import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';

import type { Root } from 'hast';
import remarkMdx from 'remark-mdx';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import { unified } from 'unified';
import { VFile } from 'vfile';

import { rehypeStatements } from '../rehype-statements';

const BASE = '/math-study';

const directories: string[] = [];

/** 単体テストのための，文書のディレクトリ． */
interface Site {
  directory: string;
  /** MDXを読み，番号を付ける変換までを行う．変換後のhastの木を返す． */
  transform: (name: string, source: string, frontmatter?: Record<string, unknown>) => Promise<Root>;
}

/** 一時ディレクトリに，ファイルの内容から，文書のディレクトリを作る． */
async function createSite(files: Record<string, string> = {}): Promise<Site> {
  const directory = await mkdtemp(path.join(tmpdir(), 'statements-'));
  directories.push(directory);
  await Promise.all(
    Object.entries(files).map(async ([name, content]) => {
      const file = path.join(directory, name);
      await mkdir(path.dirname(file), { recursive: true });
      await writeFile(file, content);
    }),
  );
  const processor = unified()
    .use(remarkParse)
    .use(remarkMdx)
    .use(remarkRehype, { passThrough: ['mdxJsxFlowElement', 'mdxJsxTextElement'] })
    .use(rehypeStatements, { contentDirectory: directory, base: BASE });
  return {
    directory,
    transform: (name, source, frontmatter) => {
      const file = new VFile({ path: path.join(directory, name), value: source });
      if (frontmatter !== undefined) {
        Object.assign(file.data, { astro: { frontmatter } });
      }
      return processor.run(processor.parse(file), file);
    },
  };
}

/** `createSite`が作ったディレクトリを，すべて削除する． */
async function removeSites(): Promise<void> {
  await Promise.all(directories.map((directory) => rm(directory, { recursive: true })));
}

export { BASE, createSite, removeSites };
export type { Site };
