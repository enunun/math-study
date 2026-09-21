import type { Element, Root } from 'hast';
import type { Plugin } from 'unified';
import type { VFile } from 'vfile';

import { createCatalog } from './statements/catalog';
import type { Catalog, CatalogOptions } from './statements/catalog';
import { findStatementNodes, resolvePageId, toInfo } from './statements/collect';
import type { Statement, StatementInfo } from './statements/collect';
import { EQUATION_COMPONENT, findLabelledEquations, numberPageItems } from './statements/equations';
import type { Equation, LabelledEquation } from './statements/equations';
import { collectHastMathSites } from './statements/math-sites';
import type { HastMathSite } from './statements/math-sites';
import {
  collectJsxElements,
  DocumentError,
  findAttribute,
  readStringAttribute,
  removeAttribute,
  setStringAttribute,
} from './statements/tree';
import type { JsxElement, TreeNode } from './statements/tree';

type Options = CatalogOptions;

/** 参照の飛び先． */
interface Target {
  label: string;
  href: string;
  /** 定義や定理ではなく，式を指すときはtrue． */
  equation: boolean;
}

/** 参照の要素の属性(`to`や`of`)が指す，定義や定理を探す関数． */
type Resolve = (element: JsxElement, attribute: string) => Promise<Target>;

/** 定義や定理の集まりと，それを持つページのURL(同じページのときは空)，誤りの表示に使うページの呼び名． */
interface Scope {
  statements: StatementInfo[];
  url: string;
  where: string;
}

/** 変換しているページ自身の，定義や定理． */
type OwnPage = Scope & { pageId: string };

/** 変換の対象になる要素． */
interface Targets {
  statementNodes: JsxElement[];
  equations: LabelledEquation<HastMathSite>[];
  refs: JsxElement[];
  proofs: JsxElement[];
}

/** frontmatterに書かれたpageIdを，Astroが渡すデータから読む． */
function readDeclaredPageId(file: VFile): unknown {
  const { astro } = file.data as { astro?: { frontmatter?: Record<string, unknown> } };
  return astro?.frontmatter?.pageId;
}

/** 定義や定理の要素に，コンポーネントが表示に使うラベルとアンカーを渡す． */
function decorate(statements: readonly Statement[]): void {
  for (const { node, label, anchor } of statements) {
    setStringAttribute(node, 'label', label);
    setStringAttribute(node, 'anchor', anchor);
  }
}

/** 参照の要素が指す，定義や定理の識別子と，ページの識別子を読む． */
function readReference(
  element: JsxElement,
  attribute: string,
): { id: string; page: string | undefined } {
  const id = readStringAttribute(element, attribute);
  if (id === undefined) {
    throw new DocumentError(
      `<${element.name ?? ''}>には，${attribute}="識別子"が要る．`,
      element.position?.start,
    );
  }
  return { id, page: readStringAttribute(element, 'page') };
}

/** 定義や定理の集まりから，識別子の定義や定理を探す． */
function locate(scope: Scope, id: string, element: JsxElement): Target {
  const target = scope.statements.find((statement) => statement.id === id);
  if (target === undefined) {
    throw new DocumentError(
      `識別子「${id}」の定義や定理が，${scope.where}にない．`,
      element.position?.start,
    );
  }
  return {
    label: target.label,
    href: `${scope.url}#${target.anchor}`,
    equation: target.component === EQUATION_COMPONENT,
  };
}

/** 同じページの定義や定理と，ほかのページの定義や定理から，参照の飛び先を探す． */
function createResolver(own: OwnPage, catalog: Catalog): Resolve {
  return async (element, attribute) => {
    const { id, page } = readReference(element, attribute);
    if (page === undefined || page === own.pageId) {
      return locate(own, id, element);
    }
    const entry = await catalog.lookup(page);
    if (entry === undefined) {
      throw new DocumentError(
        `ページ「${page}」が見つからない．定義や定理を持つページの識別子を書く．`,
        element.position?.start,
      );
    }
    const scope = {
      statements: entry.statements,
      url: catalog.urlOf(entry),
      where: `ページ「${page}」`,
    };
    return locate(scope, id, element);
  };
}

/** 証明の`of`属性が指す定義や定理を，証明の題名にする． */
async function titleProof(proof: JsxElement, resolve: Resolve): Promise<void> {
  if (findAttribute(proof, 'title') !== undefined) {
    throw new DocumentError(
      '<Proof>に，ofとtitleは，どちらか一方だけを書く．',
      proof.position?.start,
    );
  }
  const { label, equation } = await resolve(proof, 'of');
  if (equation) {
    throw new DocumentError(
      `<Proof>のofには，定義や定理の識別子を書く．${label}は式である．`,
      proof.position?.start,
    );
  }
  setStringAttribute(proof, 'title', label);
  removeAttribute(proof, 'of');
  removeAttribute(proof, 'page');
}

/** 定義や定理へのリンク(`<a>`)を作る． */
function createLink({ label, href }: Target): Element {
  return {
    type: 'element',
    tagName: 'a',
    properties: { href, className: ['statement-ref'] },
    children: [{ type: 'text', value: label }],
  };
}

/** 木の中の節を，置き換える． */
function replaceNodes(node: TreeNode, replacements: ReadonlyMap<TreeNode, TreeNode>): void {
  if (node.children === undefined) {
    return;
  }
  node.children = node.children.map((child) => replacements.get(child) ?? child);
  for (const child of node.children) {
    replaceNodes(child, replacements);
  }
}

/** `<Ref>`を，定義や定理へのリンクに置き換える． */
async function linkRefs(
  tree: TreeNode,
  refs: readonly JsxElement[],
  resolve: Resolve,
): Promise<void> {
  const pairs = await Promise.all(
    refs.map(async (ref): Promise<[TreeNode, TreeNode]> => [
      ref,
      createLink(await resolve(ref, 'to')),
    ]),
  );
  replaceNodes(tree, new Map(pairs));
}

/** 変換の対象になる，定義や定理，参照，`of`を持つ証明を集める． */
function findTargets(tree: Root): Targets {
  const elements = collectJsxElements(tree);
  return {
    statementNodes: findStatementNodes(tree),
    equations: findLabelledEquations(collectHastMathSites(tree)),
    refs: elements.filter((element) => element.name === 'Ref'),
    proofs: elements.filter(
      (element) => element.name === 'Proof' && findAttribute(element, 'of') !== undefined,
    ),
  };
}

/** 式のTeXを，`\label`を取り除き`\tag`を加えたものにし，式を包む要素に`id`を付ける． */
function decorateEquations(equations: readonly Equation<HastMathSite>[]): void {
  for (const { site, tex, info } of equations) {
    site.code.children = [{ type: 'text', value: tex }];
    if (site.pre !== undefined) {
      site.pre.properties.id = info.anchor;
    }
  }
}

/** ページの識別子を決め，定義や定理と式に番号を付ける． */
async function numberPage(
  { statementNodes, equations: labelled }: Targets,
  file: VFile,
  catalog: Catalog,
): Promise<OwnPage> {
  if (file.path === '') {
    throw new DocumentError('文書のファイルの場所が分からない．', undefined);
  }
  const pageId = resolvePageId(readDeclaredPageId(file), file.path);
  const { statements, equations } = numberPageItems(statementNodes, labelled, pageId);
  await catalog.assertUnique(pageId, file.path);
  decorate(statements);
  decorateEquations(equations);
  return {
    pageId,
    statements: [
      ...statements.map((statement) => toInfo(statement)),
      ...equations.map(({ info }) => info),
    ],
    url: '',
    where: 'このページ',
  };
}

/** 文書の1ページを変換する．番号を付け，参照を解決する． */
async function transform(tree: Root, file: VFile, catalog: Catalog): Promise<void> {
  const targets = findTargets(tree);
  const { statementNodes, equations, refs, proofs } = targets;
  if (statementNodes.length + equations.length + refs.length + proofs.length === 0) {
    return;
  }
  const resolve = createResolver(await numberPage(targets, file, catalog), catalog);
  await Promise.all(proofs.map((proof) => titleProof(proof, resolve)));
  await linkRefs(tree, refs, resolve);
}

/**
 * 定義や定理に，「種類+ページの識別子+連番」のラベルを付け，`<Ref>`を，そのラベルのリンクにする．
 * - 番号は，ページの中の文書の順に，定義，補題，命題，定理，系で共通に数える．
 * - `<Ref to="識別子" />`は，同じページの，その識別子の定義や定理へのリンクになる．
 *   `page="ページの識別子"`を加えると，ほかのページの定義や定理を指す．
 * - `<Proof of="識別子">`は，証明の題名を，その定義や定理のラベルにする．
 * 識別子の誤りや，指す先のない参照は，ビルドの失敗にする．
 */
const rehypeStatements: Plugin<[Options], Root> = (options) => {
  const catalog = createCatalog(options);
  return async (tree, file) => {
    try {
      await transform(tree, file, catalog);
    } catch (error) {
      if (error instanceof DocumentError) {
        file.fail(error.message, { place: error.place, ruleId: 'rehype-statements' });
      }
      throw error;
    }
  };
};

export { rehypeStatements };
export type { Options };
