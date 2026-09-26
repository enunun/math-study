import { describe, expect, it } from 'vitest';

import { shareLinks, shareTitle } from './share-links';

const PAGE_URL = 'https://enunun.github.io/math-study/topics/a/';
const TITLE = '連続性 & 極限 | えぬちゃんらんど';

describe('shareLinks', () => {
  const links = shareLinks(TITLE, PAGE_URL);
  const href = (name: string): URL => new URL(links.find((link) => link.name === name)?.href ?? '');

  it('Xの投稿画面に，題名とURLを別々に渡す', () => {
    const x = href('X');
    expect(x.origin + x.pathname).toBe('https://x.com/intent/post');
    expect(x.searchParams.get('text')).toBe(TITLE);
    expect(x.searchParams.get('url')).toBe(PAGE_URL);
  });

  it('Blueskyには，題名とURLを1つの文にして渡す', () => {
    expect(href('Bluesky').searchParams.get('text')).toBe(`${TITLE} ${PAGE_URL}`);
  });

  it('記号を含む題名も，クエリを壊さずに渡す', () => {
    expect(href('はてなブックマーク').searchParams.get('title')).toBe(TITLE);
    expect(href('はてなブックマーク').searchParams.get('url')).toBe(PAGE_URL);
  });

  it('URLだけを受け取るSNSには，URLを渡す', () => {
    expect(href('Facebook').searchParams.get('u')).toBe(PAGE_URL);
    expect(href('LINE').searchParams.get('url')).toBe(PAGE_URL);
  });
});

describe('shareTitle', () => {
  it('ページ名とサイト名をつなぐ', () => {
    expect(shareTitle('連続性', 'えぬちゃんらんど')).toBe('連続性 | えぬちゃんらんど');
  });

  it('ホームのように，ページ名がサイト名と同じときはサイト名だけにする', () => {
    expect(shareTitle('えぬちゃんらんど', 'えぬちゃんらんど')).toBe('えぬちゃんらんど');
  });
});
