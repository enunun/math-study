import { AmsConfiguration } from '@mathjax/src/js/input/tex/ams/AmsConfiguration.js';
import { BaseConfiguration } from '@mathjax/src/js/input/tex/base/BaseConfiguration.js';
import { BoldsymbolConfiguration } from '@mathjax/src/js/input/tex/boldsymbol/BoldsymbolConfiguration.js';
import { ConfigMacrosConfiguration } from '@mathjax/src/js/input/tex/configmacros/ConfigMacrosConfiguration.js';
import { HtmlConfiguration } from '@mathjax/src/js/input/tex/html/HtmlConfiguration.js';
import { NewcommandConfiguration } from '@mathjax/src/js/input/tex/newcommand/NewcommandConfiguration.js';
import { TextMacrosConfiguration } from '@mathjax/src/js/input/tex/textmacros/TextMacrosConfiguration.js';

/**
 * 図のラベルを組むMathJax(`label-svg.ts`)が使う，TeXのパッケージの名前．読み込むと，MathJaxに登録される．
 * ページの数式を組むMathJax(`tex-chtml`)が最初から持つものと，自作マクロが使うもの(`\boldsymbol`，
 * `\colored`が使う`\class`)である．ページのMathJaxと違い，未定義のマクロは失敗にする(`noundefined`を使わない)．
 */
const LABEL_TEX_PACKAGES = [
  BaseConfiguration,
  AmsConfiguration,
  NewcommandConfiguration,
  TextMacrosConfiguration,
  ConfigMacrosConfiguration,
  BoldsymbolConfiguration,
  HtmlConfiguration,
].map((configuration) => configuration.name);

export { LABEL_TEX_PACKAGES };
