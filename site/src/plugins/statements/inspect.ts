import type { Element, Root } from 'hast';

import { collectJsxElements, readStringAttribute, walk } from './tree';

/** 変換後の木にある，定義や定理の(ラベル，アンカー)の組． */
function labelsOf(tree: Root): [string | undefined, string | undefined][] {
  return collectJsxElements(tree)
    .filter((element) => element.name !== 'Proof')
    .map((element) => [
      readStringAttribute(element, 'label'),
      readStringAttribute(element, 'anchor'),
    ]);
}

/** 変換後の木にあるリンク(`<a>`)の，(文字，href)の組． */
function linksOf(tree: Root): [string, string][] {
  const links: [string, string][] = [];
  walk(tree, (node) => {
    if (node.type === 'element' && (node as Element).tagName === 'a') {
      const { properties, children } = node as Element;
      const text = children.map((child) => (child.type === 'text' ? child.value : '')).join('');
      links.push([text, String(properties.href)]);
    }
  });
  return links;
}

/** 変換後の木にある，最初の指定した名前のJSX要素の，文字列の属性． */
function attributeOf(tree: Root, name: string, attribute: string): string | undefined {
  const element = collectJsxElements(tree).find((candidate) => candidate.name === name);
  return element === undefined ? undefined : readStringAttribute(element, attribute);
}

/** 変換後の木にある，最初の指定した名前のJSX要素の，属性の名前の一覧． */
function attributeNamesOf(tree: Root, name: string): string[] {
  const element = collectJsxElements(tree).find((candidate) => candidate.name === name);
  return (element?.attributes ?? []).flatMap((attribute) =>
    attribute.type === 'mdxJsxAttribute' ? [attribute.name] : [],
  );
}

/** 変換後の木にある，別行立ての式の(TeX，id)の組． */
function mathOf(tree: Root): [string, string | undefined][] {
  const math: [string, string | undefined][] = [];
  walk(tree, (node) => {
    if (node.type === 'element' && (node as Element).tagName === 'pre') {
      const [code] = (node as Element).children;
      if (code?.type === 'element' && code.tagName === 'code') {
        const text = code.children.map((child) => (child.type === 'text' ? child.value : ''));
        const { id } = (node as Element).properties;
        math.push([text.join(''), typeof id === 'string' ? id : undefined]);
      }
    }
  });
  return math;
}

export { attributeNamesOf, attributeOf, labelsOf, linksOf, mathOf };
