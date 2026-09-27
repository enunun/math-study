// MathJaxのバンドルは，読み込むとグローバルなMathJaxオブジェクトを初期化する．型の宣言は持たない．
declare module '@mathjax/src/bundle/tex-chtml.js';
// TeXの拡張のバンドルも，読み込むとMathJaxに登録される(`site/src/calculator/tex-extensions.ts`)．
declare module '@mathjax/src/bundle/input/tex/extensions/*';
