/** 共有先のSNS．icon は Starlight の Icon の名前で，対応する図柄がないものは持たない． */
interface ShareLink {
  readonly name: string;
  readonly icon?: 'x.com' | 'blueSky' | 'facebook';
  readonly href: string;
}

/** ページの題名とURLから，各SNSの共有画面へのリンクを作る．外部のスクリプトは読み込まず，ただのリンクにする． */
function shareLinks(title: string, url: string): ShareLink[] {
  const u = encodeURIComponent(url);
  const t = encodeURIComponent(title);
  return [
    { name: 'X', icon: 'x.com', href: `https://x.com/intent/post?text=${t}&url=${u}` },
    {
      name: 'Bluesky',
      icon: 'blueSky',
      href: `https://bsky.app/intent/compose?text=${encodeURIComponent(`${title} ${url}`)}`,
    },
    {
      name: 'Facebook',
      icon: 'facebook',
      href: `https://www.facebook.com/sharer/sharer.php?u=${u}`,
    },
    { name: 'LINE', href: `https://social-plugins.line.me/lineit/share?url=${u}` },
    {
      name: 'はてなブックマーク',
      href: `https://b.hatena.ne.jp/add?mode=confirm&url=${u}&title=${t}`,
    },
  ];
}

/** 共有する文に使う題名．ホームはサイト名だけにし，ほかのページは「ページ名 | サイト名」にする． */
function shareTitle(pageTitle: string, siteTitle: string): string {
  return pageTitle === siteTitle ? siteTitle : `${pageTitle} | ${siteTitle}`;
}

export { shareLinks, shareTitle };
export type { ShareLink };
