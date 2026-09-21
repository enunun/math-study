import type { Element, ElementContent, Root } from 'hast';
import { unified } from 'unified';
import { describe, expect, it } from 'vitest';

import { rehypeAutospace } from './rehype-autospace';

const text = (value: string): ElementContent => ({ type: 'text', value });

function element(
  tagName: string,
  children: ElementContent[] = [],
  properties: Element['properties'] = {},
): Element {
  return { type: 'element', tagName, properties, children };
}

/** 描画済みの行内の数式に見立てた要素． */
function math(): Element {
  return element('mjx-container', [text('x')], { className: ['MathJax', 'not-content'] });
}

function displayMath(): Element {
  return element('mjx-container', [text('x')], { className: ['MathJax'], display: 'true' });
}

/** 段落の中身から，木を作り，変換して，数式の隙間のクラスを返す． */
async function classesOf(target: Element, ...paragraph: ElementContent[]): Promise<string[]> {
  const tree: Root = { type: 'root', children: [element('p', paragraph)] };
  await unified().use(rehypeAutospace).run(tree);
  const { className } = target.properties;
  return Array.isArray(className) ? className.map(String) : [];
}

const BOTH = ['MathJax', 'not-content', 'autospace-before', 'autospace-after'];
const NONE = ['MathJax', 'not-content'];

describe('rehypeAutospace', () => {
  it('和文の間にある行内の数式に，前後のクラスを付ける', async () => {
    const target = math();
    expect(await classesOf(target, text('点'), target, text('と実数'))).toEqual(BOTH);
  });

  it('漢字，ひらがな，カタカナ，長音符を，和文として扱う', async () => {
    for (const [before, after] of [
      ['点', '点'],
      ['あ', 'い'],
      ['ア', 'イ'],
      ['ー', 'ー'],
    ] as const) {
      const target = math();
      // eslint-disable-next-line no-await-in-loop
      expect(await classesOf(target, text(before), target, text(after))).toEqual(BOTH);
    }
  });

  it('句読点，括弧，スペースの隣には，付けない', async () => {
    const punctuation = math();
    expect(await classesOf(punctuation, text('このとき，'), punctuation, text('（と'))).toEqual(
      NONE,
    );
    const spaced = math();
    expect(await classesOf(spaced, text('点 '), spaced, text(' と'))).toEqual(NONE);
  });

  it('欧文，数字，ほかの数式の隣には，付けない', async () => {
    const latin = math();
    expect(await classesOf(latin, text('A'), latin, text('1'))).toEqual(NONE);
    const first = math();
    const second = math();
    expect(await classesOf(first, first, second)).toEqual(NONE);
  });

  it('和文の隣の側にだけ，付ける', async () => {
    const before = math();
    expect(await classesOf(before, text('点'), before, text('．'))).toEqual([
      ...NONE,
      'autospace-before',
    ]);
    const after = math();
    expect(await classesOf(after, text('，'), after, text('と'))).toEqual([
      ...NONE,
      'autospace-after',
    ]);
  });

  it('段落の端では，付けない', async () => {
    const target = math();
    expect(await classesOf(target, target)).toEqual(NONE);
  });

  it('リンクや強調の中の文字も，隣の文字として見る', async () => {
    const target = math();
    const link = element('a', [text('補題')]);
    const strong = element('strong', [element('code', [text('x')]), text('を')]);
    expect(await classesOf(target, link, target, strong)).toEqual([...NONE, 'autospace-before']);
  });

  it('コードの隣には，付けない', async () => {
    const target = math();
    const code = element('code', [text('mise')]);
    expect(await classesOf(target, code, target, text('を'))).toEqual([...NONE, 'autospace-after']);
  });

  it('リンクの先頭にある数式は，リンクの外の文字を，隣として見る', async () => {
    const target = math();
    const link = element('a', [target, text('を使う')]);
    expect(await classesOf(target, text('点'), link)).toEqual(BOTH);
  });

  it('段落などの外へは，たどらない', async () => {
    const target = math();
    const heading = element('h2', [text('見出し')]);
    const tree: Root = {
      type: 'root',
      children: [heading, element('p', [target, text('と')])],
    };
    await unified().use(rehypeAutospace).run(tree);
    expect(target.properties.className).toEqual([...NONE, 'autospace-after']);
  });

  it('改行の隣には，付けない', async () => {
    const target = math();
    expect(await classesOf(target, text('点'), element('br'), target, text('と'))).toEqual([
      ...NONE,
      'autospace-after',
    ]);
  });

  it('別行立ての式には，付けない', async () => {
    const target = displayMath();
    expect(await classesOf(target, text('点'), target, text('と'))).toEqual(['MathJax']);
  });
});
