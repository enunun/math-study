import { cp, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import type { AstroIntegration } from 'astro';

import { FONT_DIRECTORY, STYLESHEET_FILE } from '../math/constants';
import { getRenderer, shutdownRenderer } from '../math/renderer';
import type { RendererOptions } from '../math/renderer';

const STATUS_OK = 200;
const STATUS_ERROR = 500;

/**
 * MathJaxのCHTMLが必要とするフォントとCSSを用意し，終了時にMathJaxを止める．
 * - フォントは，パッケージからpublic/へ複製する．
 * - CSSは，全ページの描画が終わった後に，使った文字の分を1つのファイルへまとめる．
 *   開発サーバーでは，同じURLで，その時点のCSSを返す．
 * - ビルドと開発サーバーの終了時に，MathJaxのWorkerを終了する．
 */
function mathjaxIntegration(options: RendererOptions & { base: string }): AstroIntegration {
  const stylesheetPath = `${options.base}/${STYLESHEET_FILE}`;
  return {
    name: 'mathjax',
    hooks: {
      'astro:config:setup': async ({ config }) => {
        const packageJson = createRequire(import.meta.url).resolve(
          '@mathjax/mathjax-newcm-font/package.json',
        );
        const from = path.join(path.dirname(packageJson), 'chtml', 'woff2');
        const to = path.join(fileURLToPath(config.publicDir), FONT_DIRECTORY);
        await cp(from, to, { recursive: true });
      },
      'astro:server:setup': ({ server }) => {
        // Viteの開発サーバーは，baseを取り除いたURLでミドルウェアを呼ぶことがあるため，どちらのURLでも受け付ける．
        const paths = new Set([stylesheetPath, `/${STYLESHEET_FILE}`]);
        server.middlewares.use(async (request, response, next) => {
          const [requested] = (request.url ?? '').split('?');
          if (requested === undefined || !paths.has(requested)) {
            next();
            return;
          }
          try {
            const css = await getRenderer(options).stylesheet();
            response.writeHead(STATUS_OK, { 'content-type': 'text/css; charset=utf-8' });
            response.end(css);
          } catch (error) {
            response.writeHead(STATUS_ERROR);
            response.end(String(error));
          }
        });
      },
      'astro:build:done': async ({ dir }) => {
        const css = await getRenderer(options).stylesheet();
        await writeFile(path.join(fileURLToPath(dir), STYLESHEET_FILE), css);
        await shutdownRenderer();
      },
      'astro:server:done': async () => {
        await shutdownRenderer();
      },
    },
  };
}

export { mathjaxIntegration };
