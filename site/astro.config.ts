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
// mathjax.cssは，使った文字の分だけを含むため，デプロイのたびに中身が変わる．URLが同じだと，ブラウザやCDNが古いCSSを
// 残したまま新しいHTMLを表示し，新しい文字の寸法の規則が欠けて，文字が崩れる．設定を読むたびに変わる印を付けて，避ける．
const stylesheetUrl = `${base}/${STYLESHEET_FILE}?v=${Date.now()}`;
const statements = {
  contentDirectory: fileURLToPath(new URL('src/content/docs', import.meta.url)),
  base,
};
const figures = {
  figuresDirectory: fileURLToPath(new URL('src/figures', import.meta.url)),
  wasmPath: fileURLToPath(new URL('src/wasm/figure_bg.wasm', import.meta.url)),
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
        figures,
        mathjax: { ...math, cssUrl: stylesheetUrl },
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
      // 和文と欧文，数式の間の隙間と，図．
      customCss: ['./src/styles/typesetting.css', './src/styles/figure.css'],
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
