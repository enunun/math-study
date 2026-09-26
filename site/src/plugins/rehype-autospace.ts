import type { Element, ElementContent, Root, RootContent, Text } from 'hast';
import type { Plugin } from 'unified';

import { autospaceClasses, isJapanese, isLatin, segmentAutospace } from '../typesetting/autospace';

/** 端の文字を調べるとき，要素の外へたどってよい，行内の要素． */
const TRANSPARENT_INLINE = new Set(['a', 'del', 'em', 'mark', 'span', 'strong', 'sub', 'sup']);

/** 中の文字を変えない要素．コードブロック，スクリプト，図． */
const OPAQUE = new Set(['pre', 'script', 'style', 'svg', 'math', 'template', 'textarea']);

const MATH_CONTAINER = 'mjx-container';

/** 行内の数式の端の文字．数式は，欧文として扱う． */
const MATH_EDGE = 'x';

type Direction = 'before' | 'after';

type Parent = Root | Element;

/** 木の中の位置．`node`の`index`番目の子が，たどってきた道の上にある． */
interface Frame {
  node: Parent;
  index: number;
}

function isInlineMath(element: Element): boolean {
  return element.tagName === MATH_CONTAINER && element.properties.display === undefined;
}

/** 文字列の端の文字．サロゲートペアの文字(𠮷など)を，1文字として扱うため，コードポイントで取る． */
function textEdge(value: string, direction: Direction): string | undefined {
  const pattern = direction === 'before' ? /.$/su : /^./su;
  return pattern.exec(value)?.[0];
}

/** 節の端にある文字．文字のない要素は，undefinedにして，その先の文字を探させる． */
function edgeChar(node: RootContent | ElementContent, direction: Direction): string | undefined {
  if (node.type === 'text') {
    return textEdge(node.value, direction);
  }
  if (node.type !== 'element') {
    return undefined;
  }
  if (isInlineMath(node)) {
    return MATH_EDGE;
  }
  if (node.tagName === 'br' || node.tagName === MATH_CONTAINER) {
    // 改行と別行立ての式は，隙間の対象にならない．和文でも欧文でもない文字として返す．
    return '\n';
  }
  const children = direction === 'before' ? node.children.toReversed() : node.children;
  return children.map((child) => edgeChar(child, direction)).find((char) => char !== undefined);
}

/**
 * 節の隣にある文字を返す．
 * 隣の節に文字がなければ，行内の要素(リンクや強調)の外へたどる．段落などの端に着いたら，undefinedにする．
 */
function neighborChar(stack: readonly Frame[], direction: Direction): string | undefined {
  const frame = stack.at(-1);
  if (frame === undefined) {
    return undefined;
  }
  const { node, index } = frame;
  const siblings: (RootContent | ElementContent)[] =
    direction === 'before'
      ? node.children.slice(0, index).toReversed()
      : node.children.slice(index + 1);
  const found = siblings
    .map((sibling) => edgeChar(sibling, direction))
    .find((char) => char !== undefined);
  if (found !== undefined || node.type !== 'element' || !TRANSPARENT_INLINE.has(node.tagName)) {
    return found;
  }
  return neighborChar(stack.slice(0, -1), direction);
}

/** 行内の数式やコードで，和文の隣にある欧文の側に，クラスを付ける． */
function annotate(atom: Element, stack: readonly Frame[]): void {
  const side = (direction: Direction): boolean =>
    isLatin(edgeChar(atom, direction === 'before' ? 'after' : 'before')) &&
    isJapanese(neighborChar(stack, direction));
  const { className } = atom.properties;
  atom.properties.className = [
    ...(Array.isArray(className) ? className : []),
    ...autospaceClasses({ before: side('before'), after: side('after') }),
  ];
}

/** 本文の文字を，和文の隣の欧文ごとに分け，隙間を付ける欧文を，クラスを付けたspanで包む． */
function splitText(node: Text, stack: readonly Frame[]): ElementContent[] {
  const segments = segmentAutospace(
    node.value,
    neighborChar(stack, 'before'),
    neighborChar(stack, 'after'),
  );
  if (segments.every((segment) => !segment.before && !segment.after)) {
    return [node];
  }
  return segments.map((segment): ElementContent => {
    const content: Text = { type: 'text', value: segment.text };
    const classes = autospaceClasses(segment);
    return classes.length === 0
      ? content
      : {
          type: 'element',
          tagName: 'span',
          properties: { className: classes },
          children: [content],
        };
  });
}

function visitChildren(parent: Parent, stack: readonly Frame[]): void {
  const children: (RootContent | ElementContent)[] = parent.children;
  const replaced = children.flatMap((child, index): (RootContent | ElementContent)[] => {
    const next = [...stack, { node: parent, index }];
    if (child.type === 'text') {
      return splitText(child, next);
    }
    if (child.type === 'element') {
      if (isInlineMath(child) || child.tagName === 'code') {
        annotate(child, next);
      } else if (child.tagName !== MATH_CONTAINER && !OPAQUE.has(child.tagName)) {
        visitChildren(child, next);
      }
    } else if ('children' in child) {
      // MDXのコンポーネント(`<Theorem>`など)の中身も，段落と同じように扱う．
      visitChildren(child as unknown as Element, next);
    }
    return [child];
  });
  // 子を置き換えるのは，すべての子の隣の文字を調べた後にする．調べている途中で，兄弟の位置がずれないようにするためである．
  parent.children = replaced as typeof parent.children;
}

/**
 * 和文と欧文の間に，隙間(TeXの`\xkanjiskip`)を入れるためのクラスを付ける．
 * 本文の欧文と数字は，クラスを付けたspanで包む．行内の数式とコードは，独立した箱なので，その要素にクラスを付ける．
 * 前後の文字が，漢字，ひらがな，カタカナのときだけ付け，句読点や括弧，スペースの隣には付けない．
 * 隙間の幅は，CSS(`typesetting.css`)が決める．
 * 数式を描画するプラグインの後に，実行する．
 */
const rehypeAutospace: Plugin<[], Root> = () => (tree) => {
  visitChildren(tree, []);
};

export { rehypeAutospace };
