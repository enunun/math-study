import { unified } from '@astrojs/markdown-remark';
import starlight from '@astrojs/starlight';
import { defineConfig } from 'astro/config';
import remarkMath from 'remark-math';

import { FONT_DIRECTORY, mathjaxIntegration, STYLESHEET_FILE } from './src/integrations/mathjax';
import { macros } from './src/math/macros';
import { rehypeMathjax } from './src/plugins/rehype-mathjax';

const base = '/math-study';
const math = { macros, fontUrl: `${base}/${FONT_DIRECTORY}` };

// https://astro.build/config
export default defineConfig({
  site: 'https://enunun.github.io',
  base,
  markdown: {
    // 数式は，remark-mathで見つけ，ビルド時にMathJaxで描画する．
    processor: unified({
      remarkPlugins: [remarkMath],
      rehypePlugins: [[rehypeMathjax, { ...math, cssUrl: `${base}/${STYLESHEET_FILE}` }]],
    }),
  },
  integrations: [
    starlight({
      title: '数学の学習サイト',
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
