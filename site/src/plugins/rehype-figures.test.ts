import { mkdtemp, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

import { createFigureTransformer, elementsNamed } from './figures-test-support';

const figuresDirectory = fileURLToPath(new URL('../figures', import.meta.url));
const wasmPath = fileURLToPath(new URL('../wasm/figure_bg.wasm', import.meta.url));

const transform = createFigureTransformer({ figuresDirectory, wasmPath });

async function sceneDirectory(files: Record<string, string>): Promise<string> {
  const directory = await mkdtemp(path.join(tmpdir(), 'figures-'));
  await Promise.all(
    Object.entries(files).map(([name, content]) => writeFile(path.join(directory, name), content)),
  );
  return directory;
}

const PAGE = '前の段落\n\n<Figure src="sine-and-shifted-sine" />\n\n後の段落\n';

describe('rehypeFigures', () => {
  it('<Figure src>を，シーンから描いた図に置き換える', async () => {
    const tree = await transform(PAGE);
    const [figure] = elementsNamed(tree, 'figure');
    expect(figure?.properties.className).toContain('figure');
    const inside = figure;
    expect(elementsNamed(inside, 'svg')).toHaveLength(1);
    // x軸，y軸，2つの曲線．
    expect(elementsNamed(inside, 'path')).toHaveLength(4);
    // 2本の軸の矢じり．
    expect(elementsNamed(inside, 'polygon')).toHaveLength(2);
    // x，y，O，表題．
    expect(elementsNamed(inside, 'span')).toHaveLength(4);
    expect(elementsNamed(inside, 'code')).toHaveLength(4);
  });

  it('前後の文章は，そのまま残る', async () => {
    const tree = await transform(PAGE);
    expect(elementsNamed(tree, 'p')).toHaveLength(2);
    expect(elementsNamed(tree, 'figure')).toHaveLength(1);
  });

  it('同じ図を，2回置ける', async () => {
    const tree = await transform(
      '<Figure src="sine-and-shifted-sine" />\n\n<Figure src="sine-and-shifted-sine" />\n',
    );
    expect(elementsNamed(tree, 'figure')).toHaveLength(2);
  });

  it('srcがないと，ビルドの失敗にする', async () => {
    await expect(transform('<Figure />\n')).rejects.toThrow(/src/u);
  });

  it('srcは，小文字とハイフンの名前に限る', async () => {
    await expect(transform('<Figure src="../secret" />\n')).rejects.toThrow(/小文字/u);
    await expect(transform('<Figure src="Sine" />\n')).rejects.toThrow(/小文字/u);
  });

  it('シーンのファイルがないと，図の名前を示して失敗にする', async () => {
    await expect(transform('<Figure src="no-such-figure" />\n')).rejects.toThrow(/no-such-figure/u);
  });

  it('シーンの誤りは，図の名前と，原因のオブジェクトを示して失敗にする', async () => {
    const directory = await sceneDirectory({
      'broken.json': JSON.stringify({
        version: '0.1.0',
        description: '誤りのある図',
        view: { x: [0, 1], y: [0, 1], unit: { x: '1cm', y: '1cm' } },
        objects: [{ id: 'f', type: 'graph', var: 'x', expr: '1 +', domain: [0, 1] }],
      }),
    });
    const broken = createFigureTransformer({ figuresDirectory: directory, wasmPath });
    await expect(broken('<Figure src="broken" />\n')).rejects.toThrow(/broken.*f/su);
  });

  it('Figure以外のJSX要素には，触れない', async () => {
    const tree = await transform('<Other src="x" />\n');
    expect(elementsNamed(tree, 'figure')).toHaveLength(0);
  });
});
