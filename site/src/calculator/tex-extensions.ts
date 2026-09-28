/**
 * MathJaxのTeXの拡張のファイル．ブラウザのMathJax(`mathjax.ts`)は，`\boldsymbol`のような拡張のマクロが出てきたときに，
 * 拡張を名前で読み込もうとする(autoload)．ブラウザには，その名前のファイルを探す場所がないため，パッケージの名前で読み込む．
 * 一覧は，`@mathjax/src/bundle/input/tex/extensions/`の`.js`のファイルすべてである(`tex-extensions.test.ts`が確かめる)．
 */
const TEX_EXTENSIONS: Record<string, () => Promise<unknown>> = {
  action: () => import('@mathjax/src/bundle/input/tex/extensions/action.js'),
  ams: () => import('@mathjax/src/bundle/input/tex/extensions/ams.js'),
  amscd: () => import('@mathjax/src/bundle/input/tex/extensions/amscd.js'),
  autoload: () => import('@mathjax/src/bundle/input/tex/extensions/autoload.js'),
  bbm: () => import('@mathjax/src/bundle/input/tex/extensions/bbm.js'),
  bboldx: () => import('@mathjax/src/bundle/input/tex/extensions/bboldx.js'),
  bbox: () => import('@mathjax/src/bundle/input/tex/extensions/bbox.js'),
  begingroup: () => import('@mathjax/src/bundle/input/tex/extensions/begingroup.js'),
  boldsymbol: () => import('@mathjax/src/bundle/input/tex/extensions/boldsymbol.js'),
  braket: () => import('@mathjax/src/bundle/input/tex/extensions/braket.js'),
  bussproofs: () => import('@mathjax/src/bundle/input/tex/extensions/bussproofs.js'),
  cancel: () => import('@mathjax/src/bundle/input/tex/extensions/cancel.js'),
  cases: () => import('@mathjax/src/bundle/input/tex/extensions/cases.js'),
  centernot: () => import('@mathjax/src/bundle/input/tex/extensions/centernot.js'),
  color: () => import('@mathjax/src/bundle/input/tex/extensions/color.js'),
  colortbl: () => import('@mathjax/src/bundle/input/tex/extensions/colortbl.js'),
  colorv2: () => import('@mathjax/src/bundle/input/tex/extensions/colorv2.js'),
  configmacros: () => import('@mathjax/src/bundle/input/tex/extensions/configmacros.js'),
  dsfont: () => import('@mathjax/src/bundle/input/tex/extensions/dsfont.js'),
  empheq: () => import('@mathjax/src/bundle/input/tex/extensions/empheq.js'),
  enclose: () => import('@mathjax/src/bundle/input/tex/extensions/enclose.js'),
  extpfeil: () => import('@mathjax/src/bundle/input/tex/extensions/extpfeil.js'),
  fontsizev3: () => import('@mathjax/src/bundle/input/tex/extensions/fontsizev3.js'),
  gensymb: () => import('@mathjax/src/bundle/input/tex/extensions/gensymb.js'),
  html: () => import('@mathjax/src/bundle/input/tex/extensions/html.js'),
  mathtools: () => import('@mathjax/src/bundle/input/tex/extensions/mathtools.js'),
  mhchem: () => import('@mathjax/src/bundle/input/tex/extensions/mhchem.js'),
  newcommand: () => import('@mathjax/src/bundle/input/tex/extensions/newcommand.js'),
  noerrors: () => import('@mathjax/src/bundle/input/tex/extensions/noerrors.js'),
  noundefined: () => import('@mathjax/src/bundle/input/tex/extensions/noundefined.js'),
  physics: () => import('@mathjax/src/bundle/input/tex/extensions/physics.js'),
  require: () => import('@mathjax/src/bundle/input/tex/extensions/require.js'),
  setoptions: () => import('@mathjax/src/bundle/input/tex/extensions/setoptions.js'),
  tagformat: () => import('@mathjax/src/bundle/input/tex/extensions/tagformat.js'),
  texhtml: () => import('@mathjax/src/bundle/input/tex/extensions/texhtml.js'),
  textcomp: () => import('@mathjax/src/bundle/input/tex/extensions/textcomp.js'),
  textmacros: () => import('@mathjax/src/bundle/input/tex/extensions/textmacros.js'),
  unicode: () => import('@mathjax/src/bundle/input/tex/extensions/unicode.js'),
  units: () => import('@mathjax/src/bundle/input/tex/extensions/units.js'),
  upgreek: () => import('@mathjax/src/bundle/input/tex/extensions/upgreek.js'),
  verb: () => import('@mathjax/src/bundle/input/tex/extensions/verb.js'),
};

/** 拡張のファイルのパス(`…/input/tex/extensions/<名前>.js`)から，拡張の名前を取り出す． */
const EXTENSION_PATH = /\/input\/tex\/extensions\/(?<name>[\w-]+)\.js$/u;

/** MathJaxの読み込み器が求めたファイルを読み込む．TeXの拡張でないファイルは，失敗にする． */
function loadTexExtension(file: string): Promise<unknown> {
  const name = EXTENSION_PATH.exec(file)?.groups?.name;
  const load = name === undefined ? undefined : TEX_EXTENSIONS[name];
  if (load === undefined) {
    return Promise.reject(new Error(`読み込めないファイル：${file}`));
  }
  return load();
}

export { loadTexExtension, TEX_EXTENSIONS };
