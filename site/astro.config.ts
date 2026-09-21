import { fileURLToPath } from 'node:url';

import { unified } from '@astrojs/markdown-remark';
import starlight from '@astrojs/starlight';
import { defineConfig } from 'astro/config';
import remarkMath from 'remark-math';

import { FONT_DIRECTORY, mathjaxIntegration, STYLESHEET_FILE } from './src/integrations/mathjax';
import { environments, macros } from './src/math/macros';
import { rehypeFocusableTables } from './src/plugins/rehype-focusable-tables';
import { rehypeMathjax } from './src/plugins/rehype-mathjax';
import { rehypeStatements } from './src/plugins/rehype-statements';

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
      rehypePlugins: [
        [rehypeStatements, statements],
        rehypeFocusableTables,
        [rehypeMathjax, { ...math, cssUrl: `${base}/${STYLESHEET_FILE}` }],
      ],
    }),
  },
  integrations: [
    starlight({
      title: '数学の学習サイト',
      // コードブロックは折り返す．横にスクロールする領域は，キーボードで操作できない．
      expressiveCode: { defaultProps: { wrap: true } },
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
