/** 文書の中の位置． */
interface Point {
  line: number;
  column: number;
}

/** mdastとhastに共通する，木の節の最小限の形． */
interface TreeNode {
  type: string;
  children?: TreeNode[];
  position?: { start: Point };
}

interface JsxAttribute {
  type: 'mdxJsxAttribute';
  name: string;
  /** 文字列のとき`name="…"`，nullのとき値のない`name`，オブジェクトのとき`name={…}`． */
  value?: string | { type: string } | null;
}

interface JsxExpressionAttribute {
  type: 'mdxJsxExpressionAttribute';
}

/** MDXのJSX要素(`<Theorem>…</Theorem>`)． */
interface JsxElement extends TreeNode {
  type: 'mdxJsxFlowElement' | 'mdxJsxTextElement';
  name: string | null;
  attributes: (JsxAttribute | JsxExpressionAttribute)[];
}

/** 位置を持つ，文書の誤り．ビルドの失敗として，ファイルの位置とともに表示する． */
class DocumentError extends Error {
  public readonly place: Point | undefined;

  public constructor(message: string, place: Point | undefined) {
    super(message);
    this.name = 'DocumentError';
    this.place = place;
  }
}

function isJsxElement(node: TreeNode): node is JsxElement {
  return node.type === 'mdxJsxFlowElement' || node.type === 'mdxJsxTextElement';
}

/** 木を，文書の順(先に親，次に子)にたどる． */
function walk(node: TreeNode, visitor: (node: TreeNode) => void): void {
  visitor(node);
  for (const child of node.children ?? []) {
    walk(child, visitor);
  }
}

/** JSX要素を，文書の順に集める． */
function collectJsxElements(tree: TreeNode): JsxElement[] {
  const elements: JsxElement[] = [];
  walk(tree, (node) => {
    if (isJsxElement(node)) {
      elements.push(node);
    }
  });
  return elements;
}

function findAttribute(element: JsxElement, name: string): JsxAttribute | undefined {
  return element.attributes.find(
    (attribute): attribute is JsxAttribute =>
      attribute.type === 'mdxJsxAttribute' && attribute.name === name,
  );
}

/** 文字列の属性を読む．属性がなければundefined，文字列でなければ誤りにする． */
function readStringAttribute(element: JsxElement, name: string): string | undefined {
  const attribute = findAttribute(element, name);
  if (attribute === undefined) {
    return undefined;
  }
  if (typeof attribute.value !== 'string') {
    throw new DocumentError(
      `<${element.name ?? ''}>の${name}には，文字列(${name}="…")を書く．`,
      element.position?.start,
    );
  }
  return attribute.value;
}

function removeAttribute(element: JsxElement, name: string): void {
  element.attributes = element.attributes.filter(
    (attribute) => attribute.type !== 'mdxJsxAttribute' || attribute.name !== name,
  );
}

function setStringAttribute(element: JsxElement, name: string, value: string): void {
  removeAttribute(element, name);
  element.attributes.push({ type: 'mdxJsxAttribute', name, value });
}

export {
  collectJsxElements,
  DocumentError,
  findAttribute,
  isJsxElement,
  readStringAttribute,
  removeAttribute,
  setStringAttribute,
  walk,
};
export type { JsxElement, Point, TreeNode };
