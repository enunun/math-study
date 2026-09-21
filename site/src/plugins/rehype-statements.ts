import type { Element, Root } from 'hast';
import type { Plugin } from 'unified';
import type { VFile } from 'vfile';

import { createCatalog } from './statements/catalog';
import type { Catalog, CatalogOptions } from './statements/catalog';
import {
  FIGURE_COMPONENT,
  findFigureNodes,
  findStatementNodes,
  numberFigures,
  resolvePageId,
  toInfo,
} from './statements/collect';
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
  /** 定義や定理，式，図のどれを指すか． */
  kind: 'statement' | 'equation' | 'figure';
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
  figureNodes: JsxElement[];
  equations: LabelledEquation<HastMathSite>[];
  refs: JsxElement[];
  proofs: JsxElement[];
}

/** frontmatterに書かれたpageIdを，Astroが渡すデータから読む． */
function readDeclaredPageId(file: VFile): unknown {
  const { astro } = file.data as { astro?: { frontmatter?: Record<string, unknown> } };
  return astro?.frontmatter?.pageId;
}

/** 定義や定理，図の要素に，コンポーネントが表示に使うラベルとアンカーを渡す． */
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

function kindOf(component: string): Target['kind'] {
  if (component === EQUATION_COMPONENT) {
    return 'equation';
  }
  return component === FIGURE_COMPONENT ? 'figure' : 'statement';
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
    kind: kindOf(target.component),
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
  const { label, kind } = await resolve(proof, 'of');
  if (kind !== 'statement') {
    throw new DocumentError(
      `<Proof>のofには，定義や定理の識別子を書く．${label}は${kind === 'equation' ? '式' : '図'}である．`,
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
    figureNodes: findFigureNodes(tree),
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

/** 定義や定理，式に加えて，図に番号を付ける．図の識別子は，定義や定理，式の識別子と重ならない． */
function numberWithFigures(
  { statementNodes, figureNodes, equations: labelled }: Targets,
  pageId: string,
): { infos: StatementInfo[]; equations: Equation<HastMathSite>[] } {
  const { statements, equations } = numberPageItems(statementNodes, labelled, pageId);
  const equationInfos = equations.map(({ info }) => info);
  const taken = new Set(
    [...statements, ...equationInfos].flatMap(({ id }) => (id === undefined ? [] : [id])),
  );
  const figures = numberFigures(figureNodes, pageId, taken);
  decorate(statements);
  decorate(figures);
  return {
    infos: [
      ...statements.map((statement) => toInfo(statement)),
      ...equationInfos,
      ...figures.map((figure) => toInfo(figure)),
    ],
    equations,
  };
}

/** ページの識別子を決め，定義や定理，式，図に番号を付ける． */
async function numberPage(targets: Targets, file: VFile, catalog: Catalog): Promise<OwnPage> {
  if (file.path === '') {
    throw new DocumentError('文書のファイルの場所が分からない．', undefined);
  }
  const pageId = resolvePageId(readDeclaredPageId(file), file.path);
  await catalog.assertUnique(pageId, file.path);
  const { infos, equations } = numberWithFigures(targets, pageId);
  decorateEquations(equations);
  return { pageId, statements: infos, url: '', where: 'このページ' };
}

/** 文書の1ページを変換する．番号を付け，参照を解決する． */
async function transform(tree: Root, file: VFile, catalog: Catalog): Promise<void> {
  const targets = findTargets(tree);
  const { statementNodes, figureNodes, equations, refs, proofs } = targets;
  if (
    statementNodes.length + figureNodes.length + equations.length + refs.length + proofs.length ===
    0
  ) {
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
 * - `id`を持つ`<Figure>`にも，「図+ページの識別子+連番」のラベルを付ける．連番は，図だけで数える．
 *   図への参照は，同じページの図だけを指せる．
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
