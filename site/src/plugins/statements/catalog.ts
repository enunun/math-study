import { readdir, readFile, stat } from 'node:fs/promises';
import path from 'node:path';

import remarkFrontmatter from 'remark-frontmatter';
import remarkMdx from 'remark-mdx';
import remarkParse from 'remark-parse';
import { unified } from 'unified';
import { parse as parseYaml } from 'yaml';

import { findStatementNodes, numberStatements, resolvePageId, toInfo } from './collect';
import type { StatementInfo } from './collect';
import { DocumentError } from './tree';

interface CatalogOptions {
  /** 文書のディレクトリ(`site/src/content/docs`)の絶対パス． */
  contentDirectory: string;
  /** 公開するサイトのbaseパス(`/math-study`)． */
  base: string;
}

/** 定義や定理を持つ，1つのページ． */
interface PageEntry {
  pageId: string;
  /** ファイルの絶対パス． */
  file: string;
  statements: StatementInfo[];
}

interface Catalog {
  /** 識別子のページを探す．なければundefined．同じ識別子のページが複数あるときは，誤りにする． */
  lookup: (pageId: string) => Promise<PageEntry | undefined>;
  /** ほかのファイルが，同じ識別子を持たないことを確かめる． */
  assertUnique: (pageId: string, file: string) => Promise<void>;
  /** ページのURL(baseパスを含み，`/`で終わる)． */
  urlOf: (entry: PageEntry) => string;
}

interface CacheEntry {
  mtimeMs: number;
  page: PageEntry | undefined;
}

const processor = unified().use(remarkParse).use(remarkFrontmatter, ['yaml']).use(remarkMdx);

/** YAMLのfrontmatterから，`pageId`を読む． */
function readDeclaredPageId(yaml: string): unknown {
  const data: unknown = parseYaml(yaml);
  return typeof data === 'object' && data !== null && 'pageId' in data ? data.pageId : undefined;
}

/**
 * MDXのソースから，定義や定理を集める．
 * 定義や定理のないページ，読み込めないページは，undefinedにする．
 * 読み込めないページの誤りは，そのページのビルドで報告される．
 */
function scanSource(source: string, file: string): PageEntry | undefined {
  try {
    const tree = processor.parse(source);
    const nodes = findStatementNodes(tree);
    if (nodes.length === 0) {
      return undefined;
    }
    const [first] = tree.children;
    const declared = first?.type === 'yaml' ? readDeclaredPageId(first.value) : undefined;
    const pageId = resolvePageId(declared, file);
    return {
      pageId,
      file,
      statements: numberStatements(nodes, pageId).map((statement) => toInfo(statement)),
    };
  } catch {
    return undefined;
  }
}

/** ファイル名の各部分は，URLになる．Starlightが作るURLと一致するよう，小文字の英数字に限る． */
const URL_SEGMENT_PATTERN = /^[a-z0-9][a-z0-9_-]*$/u;

/** 文書のファイルの場所から，ページのURLを作る． */
function pageUrl(options: CatalogOptions, file: string): string {
  const parts = path.relative(options.contentDirectory, file).split(path.sep);
  const { name } = path.parse(parts.pop() ?? '');
  const segments = name === 'index' ? parts : [...parts, name];
  for (const segment of segments) {
    if (!URL_SEGMENT_PATTERN.test(segment)) {
      throw new DocumentError(
        `ファイル「${path.relative(options.contentDirectory, file)}」のURLが，「${segment}」の部分で，リンクに使えない．ページをまたぐ参照は，ファイル名を小文字の英数字にする．`,
        undefined,
      );
    }
  }
  return `${options.base}/${segments.map((segment) => `${segment}/`).join('')}`;
}

/**
 * 文書のディレクトリの`.mdx`を読み，定義や定理を持つページの一覧を返す関数を作る．
 * ファイルの更新時刻が変わったものだけ，読み直す．
 */
function createPageCache(directory: string): () => Promise<PageEntry[]> {
  const cache = new Map<string, CacheEntry>();
  const state: { refreshing?: Promise<PageEntry[]> } = {};

  const load = async (file: string): Promise<void> => {
    const { mtimeMs } = await stat(file);
    if (cache.get(file)?.mtimeMs === mtimeMs) {
      return;
    }
    cache.set(file, { mtimeMs, page: scanSource(await readFile(file, 'utf8'), file) });
  };

  const refresh = async (): Promise<PageEntry[]> => {
    const entries = await readdir(directory, { recursive: true });
    const files = entries
      .filter((entry) => entry.endsWith('.mdx'))
      .map((entry) => path.join(directory, entry));
    for (const file of cache.keys()) {
      if (!files.includes(file)) {
        cache.delete(file);
      }
    }
    await Promise.all(files.map((file) => load(file)));
    return [...cache.values()].flatMap(({ page }) => (page === undefined ? [] : [page]));
  };

  // 複数のページが同時に描画されるため，読み直しは，同時には1つだけ走らせる．
  return async () => {
    state.refreshing ??= refresh();
    try {
      return await state.refreshing;
    } finally {
      delete state.refreshing;
    }
  };
}

function duplicateError(
  options: CatalogOptions,
  pageId: string,
  files: readonly string[],
): DocumentError {
  const names = files.map((file) => path.relative(options.contentDirectory, file));
  return new DocumentError(
    `ページの識別子「${pageId}」が，${names.join('と')}で重なっている．frontmatterのpageIdで，どちらかを変える．`,
    undefined,
  );
}

/** 定義や定理を持つページの一覧．ほかのページの定理を参照するために使う． */
function createCatalog(options: CatalogOptions): Catalog {
  const pages = createPageCache(options.contentDirectory);
  return {
    async lookup(pageId) {
      const all = await pages();
      const matches = all.filter((page) => page.pageId === pageId);
      if (matches.length > 1) {
        throw duplicateError(
          options,
          pageId,
          matches.map(({ file }) => file),
        );
      }
      return matches[0];
    },
    async assertUnique(pageId, file) {
      const all = await pages();
      const self = path.resolve(file);
      const others = all.filter((page) => page.pageId === pageId && page.file !== self);
      if (others.length > 0) {
        throw duplicateError(options, pageId, [self, ...others.map((page) => page.file)]);
      }
    },
    urlOf: (entry) => pageUrl(options, entry.file),
  };
}

export { createCatalog };
export type { Catalog, CatalogOptions, PageEntry };
