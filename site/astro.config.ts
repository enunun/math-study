import starlight from '@astrojs/starlight';
import { defineConfig } from 'astro/config';

// https://astro.build/config
export default defineConfig({
  site: 'https://enunun.github.io',
  base: '/math-study',
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
  ],
});
