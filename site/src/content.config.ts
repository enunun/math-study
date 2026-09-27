import { docsLoader } from '@astrojs/starlight/loaders';
import { docsSchema } from '@astrojs/starlight/schema';
import { z } from 'astro/zod';
import { defineCollection } from 'astro:content';

import { IDENTIFIER_PATTERN } from './plugins/statements/collect';

// pageIdは，定義や定理のラベルと，ほかのページからの参照に使うページの識別子．
const pageId = z.string().regex(IDENTIFIER_PATTERN).optional();
// dateは，記事の公開日．トップページの新着と，カテゴリのページの並び順に使う．カテゴリの記事には必須である(categories.ts)．
const date = z.date().optional();
const schema = docsSchema({ extend: z.object({ pageId, date }) });

export const collections = {
  docs: defineCollection({ loader: docsLoader(), schema }),
};
