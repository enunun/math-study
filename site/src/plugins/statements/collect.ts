import path from 'node:path';

import { collectJsxElements, DocumentError, readStringAttribute } from './tree';
import type { JsxElement, TreeNode } from './tree';

/** 番号を付ける要素の名前と，ラベルの先頭に付く種類の名前．定義，補題，命題，定理，系は，共通の連番を使う． */
const STATEMENT_KINDS: Readonly<Record<string, string>> = {
  Definition: '定義',
  Lemma: '補題',
  Proposition: '命題',
  Theorem: '定理',
  Corollary: '系',
};

/** ページの識別子と定理の識別子に使える文字．英数字と，ハイフン，アンダースコア． */
const IDENTIFIER_PATTERN = /^[A-Za-z0-9][A-Za-z0-9_-]*$/u;

/** 番号を付けた，1つの定義や定理． */
interface StatementInfo {
  /** 要素の名前(`Theorem`など)． */
  component: string;
  /** ページの中の連番．1から数える． */
  number: number;
  /** 書き手が指定した識別子． */
  id: string | undefined;
  /** 見出しの語句(`name`属性)． */
  name: string | undefined;
  /** 参照や見出しに表示する名前(「定理abs-3」)． */
  label: string;
  /** 要素のidと，リンクの飛び先．識別子があればそれ，なければラベル． */
  anchor: string;
}

interface Statement extends StatementInfo {
  node: JsxElement;
}

function isValidIdentifier(value: string): boolean {
  return IDENTIFIER_PATTERN.test(value);
}

/** 識別子の書き方を，書き手に示すための説明． */
const IDENTIFIER_RULE = '英数字で始め，英数字，ハイフン，アンダースコアだけを使う';

function statementLabel(component: string, pageId: string, number: number): string {
  return `${STATEMENT_KINDS[component] ?? component}${pageId}-${number}`;
}

/** 番号を付ける要素を，文書の順に集める． */
function findStatementNodes(tree: TreeNode): JsxElement[] {
  return collectJsxElements(tree).filter(
    (element) => element.name !== null && Object.hasOwn(STATEMENT_KINDS, element.name),
  );
}

/** 書き手が指定した識別子を読み，使える文字だけでできているか検査する． */
function readIdentifier(node: JsxElement): string | undefined {
  const id = readStringAttribute(node, 'id');
  if (id === undefined) {
    return undefined;
  }
  if (!isValidIdentifier(id)) {
    throw new DocumentError(
      `識別子「${id}」は使えない．${IDENTIFIER_RULE}．`,
      node.position?.start,
    );
  }
  return id;
}

/**
 * 定義や定理に，ページの識別子と，文書の順の連番から，ラベルを付ける．
 * 書き手が指定した識別子が，同じページの中で重なるときは，誤りにする．
 * 連番のラベルは，日本語の種類の名前を含み，識別子に使える文字ではないので，識別子とは重ならない．
 */
function numberStatements(nodes: readonly JsxElement[], pageId: string): Statement[] {
  const taken = new Set<string>();
  return nodes.map((node, index) => {
    const component = node.name ?? '';
    const number = index + 1;
    const label = statementLabel(component, pageId, number);
    const id = readIdentifier(node);
    if (id !== undefined) {
      if (taken.has(id)) {
        throw new DocumentError(
          `識別子「${id}」が，ページの中で重なっている．識別子は，ページの中で一意にする．`,
          node.position?.start,
        );
      }
      taken.add(id);
    }
    return {
      component,
      number,
      id,
      name: readStringAttribute(node, 'name'),
      label,
      anchor: id ?? label,
      node,
    };
  });
}

/**
 * ページの識別子を決める．frontmatterの`pageId`があればそれを，なければファイル名を使う．
 * `index`という名前のファイルは，ディレクトリの名前を使う．
 */
function resolvePageId(declared: unknown, filePath: string): string {
  if (declared !== undefined) {
    if (typeof declared !== 'string' || !isValidIdentifier(declared)) {
      throw new DocumentError(`frontmatterのpageIdが使えない．${IDENTIFIER_RULE}．`, undefined);
    }
    return declared;
  }
  const { dir, name } = path.parse(filePath);
  const slug = name === 'index' ? path.basename(dir) : name;
  if (!isValidIdentifier(slug)) {
    throw new DocumentError(
      `ファイル名「${slug}」は，ページの識別子に使えない．frontmatterにpageIdを書く．${IDENTIFIER_RULE}．`,
      undefined,
    );
  }
  return slug;
}

function toInfo({ node: _node, ...info }: Statement): StatementInfo {
  return info;
}

export {
  findStatementNodes,
  IDENTIFIER_PATTERN,
  numberStatements,
  resolvePageId,
  STATEMENT_KINDS,
  toInfo,
};
export type { Statement, StatementInfo };
