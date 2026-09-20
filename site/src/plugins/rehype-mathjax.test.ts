import rehypeStringify from 'rehype-stringify';
import remarkMath from 'remark-math';
import remarkParse from 'remark-parse';
import remarkRehype from 'remark-rehype';
import { unified } from 'unified';
import { afterAll, describe, expect, it } from 'vitest';

import { macros } from '../math/macros';
import { getRenderer, shutdownRenderer } from '../math/renderer';
import { rehypeMathjax } from './rehype-mathjax';

const options = {
  macros,
  fontUrl: '/math-study/mathjax-fonts',
  cssUrl: '/math-study/mathjax.css',
};

const processor = unified()
  .use(remarkParse)
  .use(remarkMath)
  .use(remarkRehype)
  .use(rehypeMathjax, options)
  .use(rehypeStringify);

async function render(markdown: string): Promise<string> {
  return String(await processor.process(markdown));
}

afterAll(async () => {
  await shutdownRenderer();
});

describe('rehypeMathjax', () => {
  it('行内の式を，読み上げ用のaria-labelとroleを付けたCHTMLにする', async () => {
    const html = await render(String.raw`$x \in \R$`);
    expect(html).toContain('<mjx-container');
    expect(html).toContain('role="math"');
    expect(html).toContain('aria-label="x is a member of the real numbers"');
  });

  it('別行立ての式を，displayの属性を付けて描画する', async () => {
    const html = await render(String.raw`$$
\abs{x} \ge 0
$$`);
    expect(html).toContain('display="true"');
    expect(html).toContain('aria-label="the absolute value of x is greater than or equal to 0"');
  });

  it('自作マクロを展開する', async () => {
    const html = await render(String.raw`$\N \subset \Z \subset \Q \subset \R \subset \C$`);
    expect(html).toContain('the natural numbers');
    expect(html).toContain('the complex numbers');
  });

  it('引数を取る自作マクロを展開する', async () => {
    const html = await render(String.raw`$\norm{v} = \abs{a}$`);
    expect(html).toContain('the absolute value of a');
  });

  it('bussproofsの導出木を描画する', async () => {
    const html = await render(
      String.raw`$$
\begin{prooftree}
  \AxiomC{$A$}
  \AxiomC{$A \to B$}
  \RightLabel{$\to$-E}
  \BinaryInfC{$B$}
\end{prooftree}
$$`,
    );
    expect(html).toContain('inference rule');
    expect(html).toContain('conclusion B');
  });

  it('和文を含む式を描画する', async () => {
    const html = await render(String.raw`$\text{したがって} \ x \ge 0$`);
    expect(html).toContain('<mjx-container');
  });

  it('explorer用の意味づけの属性を削る', async () => {
    const html = await render('$x^2$');
    expect(html).not.toContain('data-semantic');
    expect(html).not.toContain('data-latex');
    expect(html).not.toContain('data-speech');
    expect(html).not.toContain('data-braille');
  });

  it('数式のある文書だけに，CSSを読み込むlinkを加える', async () => {
    expect(await render('$x$')).toContain('<link rel="stylesheet" href="/math-study/mathjax.css">');
    expect(await render('数式のない文書である．')).not.toContain('mathjax.css');
  });

  it('数式のない文書は，そのまま変換する', async () => {
    expect(await render('本文である．')).toBe('<p>本文である．</p>');
  });

  it('未定義のマクロを，ビルドの失敗にする', async () => {
    await expect(render(String.raw`$\undefinedmacro{x}$`)).rejects.toThrow(
      /Undefined control sequence \\undefinedmacro/u,
    );
  });

  it('構文の誤りを，ビルドの失敗にする', async () => {
    await expect(render(String.raw`$\frac{1}{$`)).rejects.toThrow(/Missing close brace/u);
  });

  it('失敗の位置と，元のTeXを，メッセージに含める', async () => {
    const markdown = String.raw`1行目である．

3行目に誤りがある$\undefinedmacro$．`;
    await expect(render(markdown)).rejects.toMatchObject({ line: 3, ruleId: 'rehype-mathjax' });
    await expect(render(markdown)).rejects.toThrow(/TeX: \\undefinedmacro/u);
  });

  it('複数の文書を同時に処理しても，順に処理した結果と同じになる', async () => {
    const documents = [
      '$a + b$',
      String.raw`$$\int_0^1 x\,dx$$`,
      String.raw`$\R$と$\N$`,
      String.raw`$\set{1, 2}$`,
    ];
    const sequential: string[] = [];
    for (const document of documents) {
      // 結果を比べるため，1つずつ処理する．
      // eslint-disable-next-line no-await-in-loop
      sequential.push(await render(document));
    }
    const concurrent = await Promise.all(documents.map((document) => render(document)));
    expect(concurrent).toEqual(sequential);
  });

  it('CSSに，フォントのURLと，使った文字の分が含まれる', async () => {
    await render(String.raw`$\sum_{k=1}^{n} k$`);
    const css = await getRenderer(options).stylesheet();
    expect(css).toContain('/math-study/mathjax-fonts/');
    expect(css).toContain('mjx-container');
  });
});
