import { readdirSync } from 'node:fs';
import path from 'node:path';

const DIST = path.resolve('site/dist');
const INDEX = 'index.html';

/**
 * ビルドしたサイトの，すべてのページを，baseURLからの相対パスで返す．
 * ホームは空文字，ディレクトリのページは`dev/notation/`，それ以外は`404.html`の形である．
 * 新しいページを足しても，検査の対象に，自動で入る．
 */
function listRoutes(): string[] {
  return readdirSync(DIST, { recursive: true, encoding: 'utf8' })
    .filter((file) => file.endsWith('.html') && !file.startsWith('pagefind'))
    .map((file) => file.split(path.sep).join('/'))
    .map((file) => (file.endsWith(INDEX) ? file.slice(0, -INDEX.length) : file))
    .toSorted();
}

/** 検査の名前に使う，ページの呼び名． */
function routeName(route: string): string {
  return route === '' ? '(ホーム)' : route;
}

export { listRoutes, routeName };
