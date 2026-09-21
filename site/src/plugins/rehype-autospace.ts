import type { Element, ElementContent, Root, RootContent } from 'hast';
import type { Plugin } from 'unified';

/** 和文の文字．漢字，ひらがな，カタカナ，長音符．句読点や括弧は，含まない． */
const JAPANESE_LETTER = /[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}ー]/u;

/** 端の文字を調べるとき，要素の外へたどってよい，行内の要素． */
const TRANSPARENT_INLINE = new Set(['a', 'del', 'em', 'mark', 'span', 'strong', 'sub', 'sup']);

const MATH_CONTAINER = 'mjx-container';
const BEFORE_CLASS = 'autospace-before';
const AFTER_CLASS = 'autospace-after';

type Direction = 'before' | 'after';

/** 木の中の位置．`node`の`index`番目の子が，たどってきた道の上にある． */
interface Frame {
  node: Root | Element;
  index: number;
}

/** 節の端にある文字．文字のない要素は，undefinedにして，その先の文字を探させる． */
function edgeChar(node: RootContent | ElementContent, direction: Direction): string | undefined {
  if (node.type === 'text') {
    // サロゲートペアの文字(𠮷など)を，1文字として扱うため，コードポイントで取る．
    const pattern = direction === 'before' ? /.$/su : /^./su;
    return pattern.exec(node.value)?.[0];
  }
  if (node.type !== 'element') {
    return undefined;
  }
  if (node.tagName === 'br') {
    // 改行は，隙間の対象にならない．和文でも欧文でもない文字として返す．
    return '\n';
  }
  const children = direction === 'before' ? node.children.toReversed() : node.children;
  return children.map((child) => edgeChar(child, direction)).find((char) => char !== undefined);
}

/**
 * 数式の隣にある文字を返す．
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

function isJapanese(char: string | undefined): boolean {
  return char !== undefined && JAPANESE_LETTER.test(char);
}

/** 行内の数式で，和文の隣にある側に，クラスを付ける． */
function annotate(math: Element, stack: readonly Frame[]): void {
  const added = [
    ...(isJapanese(neighborChar(stack, 'before')) ? [BEFORE_CLASS] : []),
    ...(isJapanese(neighborChar(stack, 'after')) ? [AFTER_CLASS] : []),
  ];
  const { className } = math.properties;
  math.properties.className = [...(Array.isArray(className) ? className : []), ...added];
}

function isInlineMath(element: Element): boolean {
  return element.tagName === MATH_CONTAINER && element.properties.display === undefined;
}

function visitChildren(parent: Root | Element, stack: readonly Frame[]): void {
  for (const [index, child] of parent.children.entries()) {
    if (child.type === 'element') {
      const next = [...stack, { node: parent, index }];
      if (isInlineMath(child)) {
        annotate(child, next);
      } else {
        visitChildren(child, next);
      }
    }
  }
}

/**
 * 和文の隣にある行内の数式に，隙間を入れるためのクラスを付ける．
 * 和文と欧文の間の隙間は，CSSのtext-autospaceが入れるが，数式は独立した箱で，その対象にならない．
 * 前後の文字が，漢字，ひらがな，カタカナのときだけ，`autospace-before`と`autospace-after`を付ける．
 * 句読点や括弧，スペース，欧文の隣には付けない．隙間の幅は，CSS(`typesetting.css`)が決める．
 * 数式を描画するプラグインの後に，実行する．
 */
const rehypeAutospace: Plugin<[], Root> = () => (tree) => {
  visitChildren(tree, []);
};

export { rehypeAutospace };
