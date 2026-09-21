/** 展開後の式，引数の数，先頭の引数を省略したときの既定値(`[…]`で書く任意引数)． */
type Macro = string | [string, number] | [string, number, string];
type Macros = Record<string, Macro>;

/** 環境の名前と，`\begin`と`\end`の位置に置く式． */
type Environments = Record<string, [string, string]>;

/**
 * 分野や機能ごとの，マクロの集まり．`modules/`の下のファイルが，これをdefault exportする．
 * 値が文字列のマクロは展開後の式，`[式, 引数の数]`のマクロは，`#1`などの引数を取る式を表す．
 * 3つ目の要素があるときは，先頭の引数が，`[…]`で書く任意引数になる．
 */
interface MacroModule {
  macros: Macros;
  environments?: Environments;
}

export type { Environments, Macro, Macros, MacroModule };
