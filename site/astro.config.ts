import { fileURLToPath } from 'node:url';

import { unified } from '@astrojs/markdown-remark';
import react from '@astrojs/react';
import starlight from '@astrojs/starlight';
import { defineConfig } from 'astro/config';
import remarkMath from 'remark-math';

import { mathjaxIntegration } from './src/integrations/mathjax';
import { FONT_DIRECTORY, STYLESHEET_FILE } from './src/math/constants';
import { environments, macros } from './src/math/macros';
import { buildRehypePlugins } from './src/plugins/pipeline';

const base = '/math-study';
const math = { macros, environments, fontUrl: `${base}/${FONT_DIRECTORY}` };
const statements = {
  contentDirectory: fileURLToPath(new URL('src/content/docs', import.meta.url)),
  base,
};

// https://astro.build/config
export default defineConfig({
  site: 'https://enunun.github.io',
  base,
  markdown: {
    // 数式は，remark-mathで見つけ，ビルド時にMathJaxで描画する．定義や定理は，ビルド時に番号を付ける．
    processor: unified({
      remarkPlugins: [remarkMath],
      rehypePlugins: buildRehypePlugins({
        statements,
        mathjax: { ...math, cssUrl: `${base}/${STYLESHEET_FILE}` },
      }),
    }),
  },
  integrations: [
    // 計算機など，ブラウザで動く部品は，Reactで書く．
    react(),
    starlight({
      title: '数学の学習サイト',
      // コードブロックは折り返す．横にスクロールする領域は，キーボードで操作できない．
      expressiveCode: { defaultProps: { wrap: true } },
      // 和文と欧文，数式の間の隙間．
      customCss: ['./src/styles/typesetting.css'],
      defaultLocale: 'root',
      locales: { root: { label: '日本語', lang: 'ja' } },
      social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/enunun/math-study' }],
      head: [
        {
          // JavaScriptが有効なことをCSSに伝える．補足(Detail)は，無効なときだけ常に表示する．
          tag: 'script',
          content: "document.documentElement.classList.add('js');",
        },
      ],
    }),
    mathjaxIntegration({ ...math, base }),
  ],
});
