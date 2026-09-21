import type { Element, Root } from 'hast';
import type { Plugin } from 'unified';
import { visit } from 'unist-util-visit';
import type { VFile } from 'vfile';

import { getRenderer } from '../math/renderer';
import type { Renderer, RendererOptions } from '../math/renderer';

/** 文書の中の位置． */
type Point = NonNullable<Element['position']>['start'];

interface Options extends RendererOptions {
  /** 数式のCSSを配信するURL(baseパスを含む)．数式のある文書に，このCSSを読み込むlinkを加える． */
  cssUrl: string;
}

/** 文書の中の1つの数式． */
interface MathSite {
  /** 置き換える要素．行内の式は`code`，別行立ての式は，それを包む`pre`． */
  target: Element;
  tex: string;
  display: boolean;
  /** 文書の中の位置．エラーの表示に使う． */
  place: Point | undefined;
}

function hasClass(element: Element, name: string): boolean {
  const { className } = element.properties;
  return Array.isArray(className) && className.includes(name);
}

/** remark-mathが作る，`<code class="language-math math-inline|math-display">`を集める． */
function collectSites(tree: Root): MathSite[] {
  const sites: MathSite[] = [];
  visit(tree, 'element', (node, _index, parent) => {
    if (node.tagName !== 'code' || !hasClass(node, 'language-math')) {
      return;
    }
    const display = hasClass(node, 'math-display');
    const target =
      display && parent?.type === 'element' && parent.tagName === 'pre' ? parent : node;
    const tex = node.children.map((child) => (child.type === 'text' ? child.value : '')).join('');
    sites.push({
      target,
      tex,
      display,
      // 別行立ての式の位置は，包む<pre>にだけ付く．
      place: (target.position ?? node.position)?.start,
    });
  });
  return sites;
}

/**
 * Starlightが，本文の中で隣り合う要素に付ける上の余白(`margin-top`)を，数式の内側で無効にするクラス．
 * MathJaxのCHTMLは`mjx-num`と`mjx-dbox`などの独自要素を隣り合わせに並べるため，
 * このクラスがないと，分数や上付き文字，導出木の規則名の位置がずれる．
 */
const STARLIGHT_EXCLUDE_CLASS = 'not-content';

/** 描画した要素で，文書の中の要素を置き換える． */
function replace(target: Element, container: Element): void {
  const { className } = container.properties;
  // 式番号を付けた式のidは，参照の飛び先になるため，引き継ぐ．
  const { id } = target.properties;
  target.tagName = container.tagName;
  target.properties = {
    ...container.properties,
    ...(id === undefined ? {} : { id }),
    // 別行立ての式は，長いと横にスクロールする．キーボードでもスクロールできるよう，フォーカスできるようにする．
    ...(container.properties.display === undefined ? {} : { tabIndex: 0 }),
    className: [...(Array.isArray(className) ? className : []), STARLIGHT_EXCLUDE_CLASS],
  };
  target.children = container.children;
}

/** 描画に失敗した式を，文書の中の位置を付けて，ビルドの失敗にする． */
function fail(file: VFile, site: MathSite, error: unknown): never {
  const reason = error instanceof Error ? error.message : String(error);
  return file.fail(`MathJax: ${reason}\n  TeX: ${site.tex}`, {
    place: site.place,
    ruleId: 'rehype-mathjax',
  });
}

/** 式を順に描画して，文書の中の要素を置き換える． */
async function renderSites(
  sites: readonly MathSite[],
  renderer: Renderer,
  file: VFile,
): Promise<void> {
  const [site, ...rest] = sites;
  if (site === undefined) {
    return;
  }
  try {
    replace(site.target, await renderer.render(site.tex, site.display));
  } catch (error) {
    fail(file, site, error);
  }
  await renderSites(rest, renderer, file);
}

/**
 * 数式を，ビルド時にMathJaxのCHTMLへ置き換える．
 * 未定義のマクロや構文の誤りは，ビルドの失敗にする．
 * 数式のある文書の末尾には，共通のCSSを読み込むlinkの要素を加える．
 */
const rehypeMathjax: Plugin<[Options], Root> = (options) => {
  const renderer = getRenderer(options);
  return async (tree, file) => {
    const sites = collectSites(tree);
    if (sites.length === 0) {
      return;
    }
    await renderSites(sites, renderer, file);
    tree.children.push({
      type: 'element',
      tagName: 'link',
      properties: { rel: ['stylesheet'], href: options.cssUrl },
      children: [],
    });
  };
};

export { rehypeMathjax };
export type { Options };
