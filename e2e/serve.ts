import { createReadStream, existsSync, statSync } from 'node:fs';
import { createServer } from 'node:http';
import path from 'node:path';

const PORT = 4322;
const BASE = '/math-study/';
const ROOT = path.resolve('site/dist');
const STATUS_OK = 200;
const STATUS_NOT_FOUND = 404;
const CONTENT_TYPES: Record<string, string> = {
  '.css': 'text/css',
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript',
  '.json': 'application/json',
  '.svg': 'image/svg+xml',
  '.woff2': 'font/woff2',
  '.xml': 'application/xml',
};

/** URLのパス部分をデコードして返す．不正な文字列のときはundefinedを返す． */
function decodePathname(url: string): string | undefined {
  try {
    return decodeURIComponent(new URL(url, `http://127.0.0.1:${PORT}`).pathname);
  } catch {
    return undefined;
  }
}

/** URLのパスを，site/distの中のファイルに対応づける．範囲の外や存在しないときはundefinedを返す． */
function resolveFile(url: string): string | undefined {
  const pathname = decodePathname(url);
  if (pathname === undefined || !pathname.startsWith(BASE)) {
    return undefined;
  }
  const target = path.normalize(path.join(ROOT, pathname.slice(BASE.length)));
  if (!target.startsWith(ROOT) || !existsSync(target)) {
    return undefined;
  }
  return statSync(target).isDirectory() ? path.join(target, 'index.html') : target;
}

// ビルド済みのサイト(site/dist)を，公開時と同じbaseパスで配信する．
// astro previewは，エージェント環境ではバックグラウンドで起動して終了するため，使わない．
const server = createServer((request, response) => {
  const file = resolveFile(request.url ?? '/');
  if (file === undefined || !existsSync(file)) {
    response.writeHead(STATUS_NOT_FOUND, { 'content-type': 'text/plain; charset=utf-8' });
    response.end('Not Found');
    return;
  }
  response.writeHead(STATUS_OK, {
    'content-type': CONTENT_TYPES[path.extname(file)] ?? 'application/octet-stream',
  });
  createReadStream(file).pipe(response);
});
server.listen(PORT, '127.0.0.1');

export { server };
