import { afterAll, describe, expect, it } from 'vitest';

import { labelsOf, linksOf, mathOf } from './statements/inspect';
import { BASE, createSite, removeSites } from './statements/test-support';

afterAll(removeSites);

/** 複数行のMDXを，行から作る． */
function lines(...rows: string[]): string {
  return rows.join('\n');
}

const LABELLED = lines('$$', String.raw`a^2 + b^2 = c^2 \label{pythagoras}`, '$$');
const TAGGED = lines('a^2 + b^2 = c^2', String.raw`\tag{abs-1}`);

describe('rehypeStatements: 式番号', () => {
  it(
    String.raw`\labelのある別行立ての式に，ページの識別子と連番の番号を付け，idを付ける`,
    async () => {
      const site = await createSite();
      const tree = await site.transform('abs.mdx', LABELLED);
      expect(mathOf(tree)).toEqual([[TAGGED, 'pythagoras']]);
    },
  );

  it(String.raw`\labelのない式は，そのままにする．番号は，\labelのある式だけを数える`, async () => {
    const site = await createSite();
    const tree = await site.transform(
      'abs.mdx',
      lines(
        '$$',
        'x = 1',
        '$$',
        '',
        '$$',
        String.raw`y = 2 \label{second}`,
        '$$',
        '',
        '$$',
        String.raw`z = 3 \label{third}`,
        '$$',
      ),
    );
    expect(mathOf(tree)).toEqual([
      ['x = 1', undefined],
      [lines('y = 2', String.raw`\tag{abs-1}`), 'second'],
      [lines('z = 3', String.raw`\tag{abs-2}`), 'third'],
    ]);
  });

  it(String.raw`番号の文字にある_は，テキストモードのために\_と書く`, async () => {
    const site = await createSite();
    const tree = await site.transform('my_page.mdx', LABELLED);
    expect(mathOf(tree)).toEqual([
      [lines('a^2 + b^2 = c^2', String.raw`\tag{my\_page-1}`), 'pythagoras'],
    ]);
  });

  it('定義や定理の番号とは，別に数える', async () => {
    const site = await createSite();
    const tree = await site.transform(
      'abs.mdx',
      lines('<Theorem id="t">a</Theorem>', '', LABELLED, '', '<Lemma>b</Lemma>'),
    );
    expect(labelsOf(tree).map(([label]) => label)).toEqual(['定理abs-1', '補題abs-2']);
    expect(mathOf(tree)).toEqual([[TAGGED, 'pythagoras']]);
  });

  it('同じページの参照を，「式(ページの識別子-連番)」のリンクにする', async () => {
    const site = await createSite();
    const tree = await site.transform(
      'abs.mdx',
      lines(LABELLED, '', '<Ref to="pythagoras" />による．'),
    );
    expect(linksOf(tree)).toEqual([['式(abs-1)', '#pythagoras']]);
  });

  it('ほかのページの式への参照を，そのページのURLへのリンクにする．式だけのページも探せる', async () => {
    const site = await createSite({ 'analysis/abs.mdx': LABELLED });
    const tree = await site.transform('intro.mdx', '<Ref page="abs" to="pythagoras" />');
    expect(linksOf(tree)).toEqual([['式(abs-1)', `${BASE}/analysis/abs/#pythagoras`]]);
  });

  it('導出木のように波括弧を含む式があるページも，ほかのページから探せる', async () => {
    const tree = lines(
      '$$',
      String.raw`\begin{prooftree}\AxiomC{$A$}\UnaryInfC{$B$}\end{prooftree}`,
      '$$',
    );
    const site = await createSite({
      'analysis/abs.mdx': lines('<Theorem id="t">a</Theorem>', '', tree),
    });
    const result = await site.transform('intro.mdx', '<Ref page="abs" to="t" />');
    expect(linksOf(result)).toEqual([['定理abs-1', `${BASE}/analysis/abs/#t`]]);
  });

  it('識別子が，定義や定理の識別子と重なるときは，ビルドの失敗にする', async () => {
    const site = await createSite();
    await expect(
      site.transform('abs.mdx', lines('<Theorem id="pythagoras">a</Theorem>', '', LABELLED)),
    ).rejects.toThrow(/識別子「pythagoras」が，ページの中で重なっている/u);
  });

  it('式の識別子が重なるときは，位置つきで，ビルドの失敗にする', async () => {
    const site = await createSite();
    const source = lines(
      '$$',
      String.raw`a \label{x}`,
      '$$',
      '',
      '$$',
      String.raw`b \label{x}`,
      '$$',
    );
    await expect(site.transform('abs.mdx', source)).rejects.toThrow(/重なっている/u);
    await expect(site.transform('abs.mdx', source)).rejects.toMatchObject({ line: 5 });
  });

  it(String.raw`行内の式の\labelを，ビルドの失敗にする`, async () => {
    const site = await createSite();
    await expect(
      site.transform('abs.mdx', String.raw`文中の$x \label{a}$である．`),
    ).rejects.toThrow(/別行立ての式\(\$\$…\$\$\)だけに書ける/u);
  });

  it(String.raw`1つの式の複数の\labelと，使えない識別子を，ビルドの失敗にする`, async () => {
    const site = await createSite();
    const twice = lines('$$', String.raw`x \label{a} \label{b}`, '$$');
    const invalid = lines('$$', String.raw`x \label{式}`, '$$');
    await expect(site.transform('abs.mdx', twice)).rejects.toThrow(/1つだけ/u);
    await expect(site.transform('abs.mdx', invalid)).rejects.toThrow(/式の識別子「式」は使えない/u);
  });

  it(String.raw`式の中の\refと\eqrefを，ビルドの失敗にする`, async () => {
    const site = await createSite();
    const display = lines('$$', String.raw`x = \eqref{a}`, '$$');
    await expect(site.transform('abs.mdx', display)).rejects.toThrow(
      /本文で<Ref to="識別子" \/>を使う/u,
    );
    await expect(site.transform('abs.mdx', String.raw`文中の$\ref{a}$である．`)).rejects.toThrow(
      /\\refと\\eqrefは使えない/u,
    );
  });

  it('証明の題名に，式を指定することを，ビルドの失敗にする', async () => {
    const site = await createSite();
    await expect(
      site.transform('abs.mdx', lines(LABELLED, '', '<Proof of="pythagoras">b</Proof>')),
    ).rejects.toThrow(/式\(abs-1\)は式である/u);
  });
});
