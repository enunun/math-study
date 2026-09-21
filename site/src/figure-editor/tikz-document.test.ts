import { describe, expect, it } from 'vitest';

import { fileName, standaloneDocument } from './tikz-document';

describe('単体の文書', () => {
  it('絵を，standaloneの文書に入れる．先頭に必要なパッケージと，TikZのライブラリを読み込む', () => {
    const document = standaloneDocument('\\begin{tikzpicture}\n\\end{tikzpicture}\n');
    expect(document.startsWith(String.raw`\documentclass[tikz,border=5pt]{standalone}`)).toBe(true);
    expect(document).toContain(String.raw`\usepackage{amsmath}`);
    expect(document).toContain(String.raw`\usepackage[haranoaji]{luatexja-preset}`);
    expect(document).toContain(String.raw`\usetikzlibrary{arrows.meta}`);
    expect(document).toContain('\\begin{document}\n\\begin{tikzpicture}');
    expect(document.endsWith('\\end{tikzpicture}\n\\end{document}\n')).toBe(true);
  });

  it('ファイルの名前は，拡張子を付けたものになる', () => {
    expect(fileName('figure', 'tikz')).toBe('figure.tikz');
    expect(fileName('figure', 'json')).toBe('figure.json');
  });
});
