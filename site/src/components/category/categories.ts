/** 小分類．カテゴリの下のディレクトリ1つに対応し，サイドバーでは入れ子のグループになる． */
interface Section {
  readonly directory: string;
  readonly label: string;
  readonly description: string;
}

/** カテゴリ．content/docsの直下のディレクトリ1つに対応する．小分類がなければ，記事を直下に置く． */
interface Category {
  readonly directory: string;
  readonly label: string;
  readonly description: string;
  readonly sections: readonly Section[];
}

// サイドバー，トップページ，カテゴリのページは，すべてこの一覧から作る．カテゴリや小分類を足すときは，ここに足し，
// カテゴリのページ(<directory>/index.mdx)を置く．
const CATEGORIES: readonly Category[] = [
  {
    directory: 'topics',
    label: '単発ネタ',
    description: '1ページで完結する話題である．ほかのページを前提にせず，単独で読める．',
    sections: [
      {
        directory: 'exam',
        label: '大学受験',
        description: '高校数学の範囲の話題である．答案の論理や，教科書の議論の厳密化を扱う．',
      },
    ],
  },
  {
    directory: 'fields',
    label: '分野別',
    description: '数学と，数学を使う分野の解説である．分野ごとの小分類にまとめる．',
    sections: [
      {
        directory: 'crystallography',
        label: '結晶学',
        description: '結晶の格子と，X線や電子線の回折を扱う．逆格子，構造因子，回折の強さ．',
      },
      {
        directory: 'fourier',
        label: 'フーリエ解析',
        description: '関数を波の重ね合わせに分ける．フーリエ級数とフーリエ変換，光の回折．',
      },
      {
        directory: 'elliptic',
        label: '楕円関数と楕円曲線',
        description: '楕円積分から生まれた関数と，3次曲線の上の点の演算を扱う．',
      },
    ],
  },
  {
    directory: 'graphics',
    label: 'グラフィックス',
    description: '図を描くための数学の解説である．',
    sections: [],
  },
  {
    directory: 'tools',
    label: 'ツール',
    description: '数学の図や文書を作るための道具である．',
    sections: [
      {
        directory: 'graphics',
        label: 'グラフィックス',
        description: '平面や空間の図を作る道具である．',
      },
      { directory: 'latex', label: 'LaTeX', description: 'LaTeXで数学を書くための道具である．' },
    ],
  },
];

/** コンテンツコレクションの項目のうち，一覧に使う部分． */
interface ArticleEntry {
  readonly id: string;
  readonly data: { readonly title: string; readonly description?: string; readonly date?: Date };
}

/** 一覧に並べる記事． */
interface Article {
  readonly id: string;
  readonly title: string;
  readonly description: string;
  readonly date: Date;
}

/** Starlightのサイドバーの項目のうち，ここで作る形． */
interface SidebarGroup {
  readonly label: string;
  // Starlightの設定の型は，readonlyの配列を受け付けない．
  readonly items: (SidebarGroup | Autogenerate)[];
}

/** ディレクトリの中のページを並べる，サイドバーの項目． */
interface Autogenerate {
  readonly autogenerate: { readonly directory: string };
}

function toArticle({ id, data }: ArticleEntry): Article {
  if (data.date === undefined) {
    throw new Error(`${id}：記事のfrontmatterにdateがない．新着の順に並べるため，公開日を書く．`);
  }
  return { id, title: data.title, description: data.description ?? '', date: data.date };
}

/** 新しい順に並べる．同じ日付のものは，idの順にして，ビルドのたびに並びが変わらないようにする． */
function byNewest(a: Article, b: Article): number {
  return b.date.getTime() - a.date.getTime() || a.id.localeCompare(b.id);
}

/** ページのid(`topics/exam/locus`)から，baseを付けたURLを作る． */
function pageHref(base: string, id: string): string {
  return `${base.replace(/\/$/u, '')}/${id}/`;
}

/** ディレクトリの下にある記事を，新しい順に返す．カテゴリのページ(index.mdx)は含めない． */
function articlesIn(entries: readonly ArticleEntry[], directory: string): Article[] {
  return entries
    .filter((entry) => entry.id.startsWith(`${directory}/`))
    .map((entry) => toArticle(entry))
    .toSorted(byNewest);
}

/** すべてのカテゴリの記事から，新しいものを返す． */
function recentArticles(entries: readonly ArticleEntry[], count: number): Article[] {
  return CATEGORIES.flatMap((category) => articlesIn(entries, category.directory))
    .toSorted(byNewest)
    .slice(0, count);
}

function findCategory(directory: string): Category {
  const category = CATEGORIES.find((candidate) => candidate.directory === directory);
  if (category === undefined) {
    throw new Error(`${directory}：カテゴリの一覧(categories.ts)にない．`);
  }
  return category;
}

/** カテゴリのページで，見出しごとに並べる記事．小分類がなければ，見出しのない1組にする． */
function articleGroups(
  entries: readonly ArticleEntry[],
  category: Category,
): { readonly section?: Section; readonly articles: Article[] }[] {
  if (category.sections.length === 0) {
    return [{ articles: articlesIn(entries, category.directory) }];
  }
  return category.sections.map((section) => ({
    section,
    articles: articlesIn(entries, `${category.directory}/${section.directory}`),
  }));
}

/** トップページのカード． */
interface OverviewCard {
  readonly title: string;
  readonly description: string;
  readonly href: string;
}

/**
 * トップページに並べる，小分類ごとのカード．小分類がなければ，カテゴリのページへの1枚にする．
 * 記事が増えても，枚数は変わらない．
 */
function overviewCards(
  entries: readonly ArticleEntry[],
  category: Category,
  base: string,
): OverviewCard[] {
  const href = pageHref(base, category.directory);
  const count = (directory: string): string => `記事は${articlesIn(entries, directory).length}件．`;
  if (category.sections.length === 0) {
    return [{ title: `${category.label}の記事`, description: count(category.directory), href }];
  }
  return category.sections.map((section) => ({
    title: section.label,
    description: section.description + count(`${category.directory}/${section.directory}`),
    href: `${href}#${section.directory}`,
  }));
}

/** 記事の置き場所を，「カテゴリ／小分類」の形で返す． */
function placeOf(id: string): string {
  const [categoryDirectory = '', sectionDirectory] = id.split('/');
  const category = findCategory(categoryDirectory);
  const section = category.sections.find((candidate) => candidate.directory === sectionDirectory);
  return section === undefined ? category.label : `${category.label}／${section.label}`;
}

function autogenerate(directory: string): Autogenerate {
  return { autogenerate: { directory } };
}

/** カテゴリのサイドバーのグループ．小分類ごとに，入れ子のグループにする． */
function sidebarGroup(category: Category): SidebarGroup {
  if (category.sections.length === 0) {
    return { label: category.label, items: [autogenerate(category.directory)] };
  }
  return {
    label: category.label,
    items: category.sections.map((section) => ({
      label: section.label,
      items: [autogenerate(`${category.directory}/${section.directory}`)],
    })),
  };
}

/** 公開日を「2026年9月26日」の形にする．frontmatterの日付は，UTCの0時として読まれる． */
function formatDate(date: Date): string {
  return `${date.getUTCFullYear()}年${date.getUTCMonth() + 1}月${date.getUTCDate()}日`;
}

export {
  CATEGORIES,
  articleGroups,
  articlesIn,
  findCategory,
  formatDate,
  overviewCards,
  pageHref,
  placeOf,
  recentArticles,
  sidebarGroup,
};
export type { Article, ArticleEntry, Category, OverviewCard, Section, SidebarGroup };
