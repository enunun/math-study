import { readFile } from 'node:fs/promises';

import { beforeAll, describe, expect, it } from 'vitest';

import { initSync, renderScene } from '@/wasm/figure';

import { emptyDraft, stringifyDraft } from './draft';
import { PLANE_SAMPLE_SCENES, SPACE_SAMPLE_SCENES } from './sample-scenes';
import { EDITOR_SAMPLES } from './samples';
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

describe('図のテンプレートは，そのまま描ける', () => {
  it.each(SCENE_TEMPLATES)('$labelを描ける', (template) => {
    const outcome = renderScene(stringifyDraft(template.scene));
    expect(outcome.status, JSON.stringify(outcome)).toBe('ok');
  });
});

describe('見本は，そのまま描ける', () => {
  it.each(EDITOR_SAMPLES)('$labelを描ける', (sample) => {
    const outcome = renderScene(stringifyDraft(sample.scene));
    expect(outcome.status, JSON.stringify(outcome)).toBe('ok');
  });

  it('見本の識別子と名前は，重ならない', () => {
    expect(new Set(EDITOR_SAMPLES.map((sample) => sample.id)).size).toBe(EDITOR_SAMPLES.length);
    expect(new Set(EDITOR_SAMPLES.map((sample) => sample.label)).size).toBe(EDITOR_SAMPLES.length);
  });
});

describe('テンプレートと見本の曲面と球は，縁とワイヤーフレームを描く', () => {
  const objects = [
    ...OBJECT_TEMPLATE_GROUPS.flatMap((group) =>
      group.templates.flatMap((template) => template.objects),
    ),
    ...EDITOR_SAMPLES.flatMap((sample) => sample.scene.objects),
  ];
  const surfaces = objects.filter((object) => object.type === 'surface');
  const spheres = objects.filter((object) => object.type === 'sphere');

  it.each(surfaces)('曲面$idは，boundaryとwireframeを持つ', (surface) => {
    expect(surface).toHaveProperty('boundary', true);
    expect(surface).toHaveProperty('wireframe');
  });

  it.each(spheres)('球$idは，wireframeを持つ', (sphere) => {
    expect(sphere).toHaveProperty('wireframe');
  });
});

// 記事の図の見本は，記事と同じファイルを使うので，刻みは確かめない．
describe('テンプレートと，ここで書いた見本の曲面は，ワイヤーフレームの刻みを持つ', () => {
  const surfaces = [
    ...OBJECT_TEMPLATE_GROUPS.flatMap((group) =>
      group.templates.flatMap((template) => template.objects),
    ),
    ...[...PLANE_SAMPLE_SCENES, ...SPACE_SAMPLE_SCENES].flatMap((sample) => sample.scene.objects),
  ].filter((object) => object.type === 'surface');

  it.each(surfaces)('曲面$idは，wireframe_stepを持つ', (surface) => {
    expect(surface).toHaveProperty('wireframe_step');
  });
});

describe('座標軸のある図のテンプレートは，原点Oを持つ', () => {
  const withAxes = SCENE_TEMPLATES.filter((template) =>
    template.scene.objects.some((object) => object.type === 'axis'),
  );

  it.each(withAxes)('$labelは，原点の名前Oを持つ', (template) => {
    expect(template.scene.objects).toContainEqual(expect.objectContaining({ tex: '$O$' }));
  });
});
