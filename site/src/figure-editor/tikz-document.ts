/**
 * TikZの絵を，単体でコンパイルできるLuaLaTeXの文書に入れる．日本語のラベルを使えるよう，LuaTeX-jaを読み込む．
 * ラベルはMathJaxが読むTeXなので，`\boldsymbol`などを使えるよう，amsmathも読み込む．
 */
function standaloneDocument(tikz: string): string {
  return [
    String.raw`\documentclass[tikz,border=5pt]{standalone}`,
    String.raw`\usepackage{amsmath}`,
    String.raw`\usepackage[haranoaji]{luatexja-preset}`,
    String.raw`\usetikzlibrary{arrows.meta}`,
    String.raw`\begin{document}`,
    tikz.trimEnd(),
    String.raw`\end{document}`,
    '',
  ].join('\n');
}

/** ダウンロードするファイルの名前．`stem`は，拡張子を除いた名前である． */
function fileName(stem: string, extension: 'json' | 'tikz' | 'tex'): string {
  return `${stem}.${extension}`;
}

export { fileName, standaloneDocument };
