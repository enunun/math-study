import { readFile } from 'node:fs/promises';

import { beforeAll, describe, expect, it } from 'vitest';

import { initSync, renderScene } from '@/wasm/figure';

import { emptyDraft, stringifyDraft } from './draft';
import { OBJECT_TEMPLATE_GROUPS, SCENE_TEMPLATES } from './templates';

beforeAll(async () => {
  const module = await readFile(new URL('../wasm/figure_bg.wasm', import.meta.url));
  initSync({ module });
});

describe('部品のテンプレートは，単独で図に描ける', () => {
  const templates = OBJECT_TEMPLATE_GROUPS.flatMap((group) =>
    group.templates.map((template) => [group.label, template] as const),
  );

  it.each(templates)('%s：%sは，最小限の図に入れて描ける', (_group, template) => {
    const draft = { ...emptyDraft(template.kind), objects: [...template.objects] };
    const outcome = renderScene(stringifyDraft(draft));
    expect(outcome.status, JSON.stringify(outcome)).toBe('ok');
  });
});

describe('図全体のテンプレートは，そのまま描ける', () => {
  it.each(SCENE_TEMPLATES)('$labelを描ける', (template) => {
    const outcome = renderScene(stringifyDraft(template.scene));
    expect(outcome.status, JSON.stringify(outcome)).toBe('ok');
  });
});
