// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

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
		}),
	],
});
