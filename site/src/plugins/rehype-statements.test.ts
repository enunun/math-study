import { afterAll, describe, expect, it } from 'vitest';

import { attributeNamesOf, attributeOf, labelsOf, linksOf } from './statements/inspect';
import { BASE, createSite, removeSites } from './statements/test-support';

afterAll(removeSites);

describe('rehypeStatements: 番号', () => {
  it('定義，補題，定理に，ページの識別子と共通の連番から，ラベルを付ける', async () => {
    const site = await createSite();
    const tree = await site.transform(
      'abs.mdx',
      ['<Definition>a</Definition>', '<Lemma>b</Lemma>', '<Theorem>c</Theorem>'].join('\n\n'),
    );
    expect(labelsOf(tree)).toEqual([
      ['定義abs-1', '定義abs-1'],
      ['補題abs-2', '補題abs-2'],
      ['定理abs-3', '定理abs-3'],
    ]);
  });

  it('識別子を指定すると，アンカーがその識別子になる．ラベルは連番のままである', async () => {
    const site = await createSite();
    const tree = await site.transform(
      'abs.mdx',
      '<Theorem id="triangle-inequality" name="三角不等式">a</Theorem>',
    );
    expect(labelsOf(tree)).toEqual([['定理abs-1', 'triangle-inequality']]);
  });

  it('入れ子の要素も，文書の順に数える．コードブロックの中は数えない', async () => {
    const site = await createSite();
    const tree = await site.transform(
      'abs.mdx',
      [
        '```mdx',
        '<Theorem>コードの例</Theorem>',
        '```',
        '<Theorem>',
        '',
        '<Lemma>入れ子</Lemma>',
        '',
        '</Theorem>',
        '',
        '<Corollary>d</Corollary>',
      ].join('\n'),
    );
    expect(labelsOf(tree).map(([label]) => label)).toEqual(['定理abs-1', '補題abs-2', '系abs-3']);
  });

  it('例，問題，解答も，定義や定理と共通の連番で数える', async () => {
    const site = await createSite();
    const tree = await site.transform(
      'abs.mdx',
      ['<Example>a</Example>', '<Problem>b</Problem>', '<Answer>c</Answer>'].join('\n\n'),
    );
    expect(labelsOf(tree).map(([label]) => label)).toEqual(['例abs-1', '問題abs-2', '解答abs-3']);
  });

  it('frontmatterのpageIdを，ファイル名より優先する', async () => {
    const site = await createSite();
    const tree = await site.transform('abs.mdx', '<Theorem>a</Theorem>', {
      pageId: 'absolute-value',
    });
    expect(labelsOf(tree)).toEqual([['定理absolute-value-1', '定理absolute-value-1']]);
  });

  it('index.mdxは，ディレクトリの名前を，ページの識別子にする', async () => {
    const site = await createSite();
    const tree = await site.transform('analysis/index.mdx', '<Theorem>a</Theorem>');
    expect(labelsOf(tree)).toEqual([['定理analysis-1', '定理analysis-1']]);
  });

  it('識別子に使えないファイル名は，ビルドの失敗にする', async () => {
    const site = await createSite();
    await expect(site.transform('絶対値.mdx', '<Theorem>a</Theorem>')).rejects.toThrow(
      /frontmatterにpageIdを書く/u,
    );
  });

  it('使えない文字のpageIdと識別子を，ビルドの失敗にする', async () => {
    const site = await createSite();
    await expect(
      site.transform('abs.mdx', '<Theorem>a</Theorem>', { pageId: '絶対値' }),
    ).rejects.toThrow(/frontmatterのpageIdが使えない/u);
    await expect(site.transform('abs.mdx', '<Theorem id="三角不等式">a</Theorem>')).rejects.toThrow(
      /識別子「三角不等式」は使えない/u,
    );
  });

  it('同じページで重なる識別子を，位置つきで，ビルドの失敗にする', async () => {
    const site = await createSite();
    const source = '<Theorem id="a">1</Theorem>\n\n<Lemma id="a">2</Lemma>';
    await expect(site.transform('abs.mdx', source)).rejects.toThrow(/重なっている/u);
    await expect(site.transform('abs.mdx', source)).rejects.toMatchObject({ line: 3 });
  });
});

describe('rehypeStatements: 図', () => {
  it('idを持つ図に，「図+ページの識別子+連番」のラベルを付ける．連番は，定理とは別に数える', async () => {
    const site = await createSite();
    const tree = await site.transform(
      'abs.mdx',
      [
        '<Figure src="a" id="fig-a" />',
        '<Theorem id="thm">t</Theorem>',
        '<Figure src="b" id="fig-b" />',
      ].join('\n\n'),
    );
    expect(labelsOf(tree)).toEqual([
      ['図abs-1', 'fig-a'],
      ['定理abs-1', 'thm'],
      ['図abs-2', 'fig-b'],
    ]);
  });

  it('idのない図には，番号を付けない', async () => {
    const site = await createSite();
    const tree = await site.transform(
      'abs.mdx',
      '<Figure src="a" />\n\n<Figure src="b" id="fig-b" />',
    );
    expect(labelsOf(tree)).toEqual([
      [undefined, undefined],
      ['図abs-1', 'fig-b'],
    ]);
  });

  it('図への参照を，ラベルのリンクにする', async () => {
    const site = await createSite();
    const tree = await site.transform(
      'abs.mdx',
      '<Figure src="a" id="fig-a" />\n\n<Ref to="fig-a" />を見る．',
    );
    expect(linksOf(tree)).toEqual([['図abs-1', '#fig-a']]);
  });

  it('図の識別子が，定理や式の識別子と重なるときは，ビルドの失敗にする', async () => {
    const site = await createSite();
    await expect(
      site.transform('abs.mdx', '<Theorem id="a">1</Theorem>\n\n<Figure src="x" id="a" />'),
    ).rejects.toThrow(/重なっている/u);
    await expect(
      site.transform('abs.mdx', '<Figure src="x" id="a" />\n\n<Figure src="y" id="a" />'),
    ).rejects.toThrow(/重なっている/u);
  });

  it('証明のofに，図の識別子を書くと，ビルドの失敗にする', async () => {
    const site = await createSite();
    await expect(
      site.transform('abs.mdx', '<Figure src="x" id="f" />\n\n<Proof of="f">b</Proof>'),
    ).rejects.toThrow(/図である/u);
  });
});

describe('rehypeStatements: 参照', () => {
  it('同じページの参照を，ラベルのリンクにする', async () => {
    const site = await createSite();
    const tree = await site.transform(
      'abs.mdx',
      '<Theorem id="triangle-inequality">a</Theorem>\n\n<Ref to="triangle-inequality" />による．',
    );
    expect(linksOf(tree)).toEqual([['定理abs-1', '#triangle-inequality']]);
  });

  it('参照は，連番のラベルに追従する．定理を挿入すると，番号だけが変わる', async () => {
    const site = await createSite();
    const before = await site.transform('abs.mdx', '<Theorem id="t">a</Theorem>\n\n<Ref to="t" />');
    const after = await site.transform(
      'abs.mdx',
      '<Lemma>b</Lemma>\n\n<Theorem id="t">a</Theorem>\n\n<Ref to="t" />',
    );
    expect(linksOf(before)).toEqual([['定理abs-1', '#t']]);
    expect(linksOf(after)).toEqual([['定理abs-2', '#t']]);
  });

  it('ほかのページの参照を，そのページのURLへのリンクにする', async () => {
    const site = await createSite({
      'analysis/abs.mdx': '<Theorem id="triangle-inequality">a</Theorem>',
    });
    const tree = await site.transform('intro.mdx', '<Ref page="abs" to="triangle-inequality" />');
    expect(linksOf(tree)).toEqual([['定理abs-1', `${BASE}/analysis/abs/#triangle-inequality`]]);
  });

  it('index.mdxのページのURLは，ディレクトリのURLになる', async () => {
    const site = await createSite({ 'analysis/index.mdx': '<Theorem id="t">a</Theorem>' });
    const tree = await site.transform('intro.mdx', '<Ref page="analysis" to="t" />');
    expect(linksOf(tree)).toEqual([['定理analysis-1', `${BASE}/analysis/#t`]]);
  });

  it('ページの識別子は，frontmatterのpageIdで探す', async () => {
    const site = await createSite({
      'analysis/abs.mdx': '---\npageId: absolute-value\n---\n\n<Theorem id="t">a</Theorem>',
    });
    const tree = await site.transform('intro.mdx', '<Ref page="absolute-value" to="t" />');
    expect(linksOf(tree)).toEqual([['定理absolute-value-1', `${BASE}/analysis/abs/#t`]]);
  });

  it('自分のページを，pageで指すことができる', async () => {
    const site = await createSite();
    const tree = await site.transform(
      'abs.mdx',
      '<Theorem id="t">a</Theorem>\n\n<Ref page="abs" to="t" />',
    );
    expect(linksOf(tree)).toEqual([['定理abs-1', '#t']]);
  });

  it('指す先のない参照を，位置つきで，ビルドの失敗にする', async () => {
    const site = await createSite();
    const source = '<Theorem>a</Theorem>\n\n<Ref to="missing" />';
    await expect(site.transform('abs.mdx', source)).rejects.toThrow(
      /識別子「missing」の定義や定理が/u,
    );
    await expect(site.transform('abs.mdx', source)).rejects.toMatchObject({ line: 3 });
  });

  it('識別子のない定理は，参照できない', async () => {
    const site = await createSite();
    await expect(
      site.transform('abs.mdx', '<Theorem>a</Theorem>\n\n<Ref to="定理abs-1" />'),
    ).rejects.toThrow(/識別子「定理abs-1」/u);
  });

  it('存在しないページへの参照を，ビルドの失敗にする', async () => {
    const site = await createSite();
    await expect(site.transform('abs.mdx', '<Ref page="nothing" to="t" />')).rejects.toThrow(
      /ページ「nothing」が見つからない/u,
    );
  });

  it('toのない参照と，文字列でない属性を，ビルドの失敗にする', async () => {
    const site = await createSite();
    await expect(site.transform('abs.mdx', '<Ref />')).rejects.toThrow(/to="識別子"が要る/u);
    await expect(site.transform('abs.mdx', '<Ref to={1} />')).rejects.toThrow(/文字列\(to="…"\)/u);
  });

  it('ページの識別子が，ほかのファイルと重なるときは，ビルドの失敗にする', async () => {
    const site = await createSite({ 'linear/abs.mdx': '<Theorem>a</Theorem>' });
    await expect(site.transform('analysis/abs.mdx', '<Theorem>b</Theorem>')).rejects.toThrow(
      /ページの識別子「abs」が/u,
    );
  });

  it('URLに使えないファイル名のページは，ページをまたぐ参照を，ビルドの失敗にする', async () => {
    const site = await createSite({
      'Linear.mdx': '---\npageId: linear\n---\n<Theorem id="t">a</Theorem>',
    });
    await expect(site.transform('abs.mdx', '<Ref page="linear" to="t" />')).rejects.toThrow(
      /小文字の英数字/u,
    );
  });
});

describe('rehypeStatements: 証明', () => {
  it('ofを，証明の題名にする', async () => {
    const site = await createSite();
    const tree = await site.transform(
      'abs.mdx',
      '<Theorem id="t">a</Theorem>\n\n<Proof of="t">b</Proof>',
    );
    expect(attributeOf(tree, 'Proof', 'title')).toBe('定理abs-1');
    expect(attributeNamesOf(tree, 'Proof')).toEqual(['title']);
  });

  it('ofとtitleの両方を，ビルドの失敗にする', async () => {
    const site = await createSite();
    await expect(
      site.transform('abs.mdx', '<Theorem id="t">a</Theorem>\n\n<Proof of="t" title="x">b</Proof>'),
    ).rejects.toThrow(/どちらか一方/u);
  });

  it('ofのない証明は，そのままにする', async () => {
    const site = await createSite();
    const tree = await site.transform('abs.mdx', '<Proof title="定理1">b</Proof>');
    expect(attributeOf(tree, 'Proof', 'title')).toBe('定理1');
  });
});
