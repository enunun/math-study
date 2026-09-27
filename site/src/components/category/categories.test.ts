import { describe, expect, it } from 'vitest';

import {
  articleGroups,
  articlesIn,
  CATEGORIES,
  formatDate,
  overviewCards,
  pageHref,
  placeOf,
  recentArticles,
  sidebarGroup,
} from './categories';
import type { ArticleEntry } from './categories';

function entry(id: string, date?: string): ArticleEntry {
  return {
    id,
    data: { title: id, description: '', ...(date === undefined ? {} : { date: new Date(date) }) },
  };
}

const ENTRIES = [
  entry('topics'),
  entry('topics/exam/a', '2026-09-25'),
  entry('topics/exam/b', '2026-09-26'),
  entry('graphics/c', '2026-09-21'),
  entry('tools/latex/d', '2026-09-25'),
  entry('dev/notation'),
];

describe('articlesIn', () => {
  it('ディレクトリの下の記事を新しい順に返し，カテゴリのページは含めない', () => {
    expect(articlesIn(ENTRIES, 'topics').map((article) => article.id)).toEqual([
      'topics/exam/b',
      'topics/exam/a',
    ]);
  });

  it('dateのない記事があれば，ビルドを止める', () => {
    expect(() => articlesIn([entry('graphics/x')], 'graphics')).toThrow(/graphics\/x/u);
  });
});

describe('recentArticles', () => {
  it('カテゴリをまたいで新しい順に並べ，同じ日付はidの順にする', () => {
    expect(recentArticles(ENTRIES, 3).map((article) => article.id)).toEqual([
      'topics/exam/b',
      'tools/latex/d',
      'topics/exam/a',
    ]);
  });

  it('カテゴリの外のページ(dev/)は含めない', () => {
    expect(recentArticles(ENTRIES, 10)).toHaveLength(4);
  });
});

describe('placeOf', () => {
  it('小分類があれば，カテゴリと小分類の名前を返す', () => {
    expect(placeOf('topics/exam/a')).toBe('単発ネタ／大学受験');
  });

  it('小分類がなければ，カテゴリの名前だけを返す', () => {
    expect(placeOf('graphics/c')).toBe('グラフィックス');
  });
});

describe('sidebarGroup', () => {
  it('小分類ごとに，入れ子のグループを作る', () => {
    const tools = CATEGORIES.find((category) => category.directory === 'tools');
    expect(tools && sidebarGroup(tools)).toEqual({
      label: 'ツール',
      items: [
        { label: 'グラフィックス', items: [{ autogenerate: { directory: 'tools/graphics' } }] },
        { label: 'LaTeX', items: [{ autogenerate: { directory: 'tools/latex' } }] },
      ],
    });
  });

  it('小分類がなければ，ディレクトリをそのまま並べる', () => {
    const graphics = CATEGORIES.find((category) => category.directory === 'graphics');
    expect(graphics && sidebarGroup(graphics)).toEqual({
      label: 'グラフィックス',
      items: [{ autogenerate: { directory: 'graphics' } }],
    });
  });
});

describe('pageHref', () => {
  it('baseの末尾のスラッシュの有無によらず，同じURLを作る', () => {
    expect(pageHref('/math-study', 'graphics/c')).toBe('/math-study/graphics/c/');
    expect(pageHref('/math-study/', 'graphics/c')).toBe('/math-study/graphics/c/');
  });
});

describe('formatDate', () => {
  it('frontmatterの日付を，和文の年月日にする', () => {
    expect(formatDate(new Date('2026-09-05'))).toBe('2026年9月5日');
  });
});

describe('articleGroups', () => {
  it('小分類ごとに，記事をまとめる', () => {
    const topics = CATEGORIES.find((category) => category.directory === 'topics');
    const groups = topics ? articleGroups(ENTRIES, topics) : [];
    expect(groups.map(({ section, articles }) => [section?.label, articles.length])).toEqual([
      ['大学受験', 2],
    ]);
  });
});

describe('overviewCards', () => {
  it('小分類ごとに，カテゴリのページの見出しへのカードを作り，記事の数を添える', () => {
    const tools = CATEGORIES.find((category) => category.directory === 'tools');
    const cards = tools ? overviewCards(ENTRIES, tools, '/math-study') : [];
    expect(cards.map((card) => card.href)).toEqual([
      '/math-study/tools/#graphics',
      '/math-study/tools/#latex',
    ]);
    expect(cards[1]?.description).toMatch(/記事は1件．$/u);
  });

  it('小分類がなければ，カテゴリのページへのカードを1枚作る', () => {
    const graphics = CATEGORIES.find((category) => category.directory === 'graphics');
    expect(graphics && overviewCards(ENTRIES, graphics, '/math-study')).toEqual([
      { title: 'グラフィックスの記事', description: '記事は1件．', href: '/math-study/graphics/' },
    ]);
  });
});
