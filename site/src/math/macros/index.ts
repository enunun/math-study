import { mergeMacroModules } from './merge';
import type { MacroModule } from './types';

/**
 * 自作マクロ．文書の中では，`\RealNumbers`や`\abs{x}`のように書く．
 * ビルド時の描画と，ブラウザでの描画で，同じ定義を使う．
 *
 * マクロは，分野や機能ごとに，`modules/`の下のファイルに定義する．
 * 新しい分野のマクロは，`modules/`に新しいファイルを置くだけで使える．このファイルの変更は要らない．
 * ファイルは，`MacroModule`をdefault exportする．マクロの名前は，モジュールをまたいで，一意にする．
 */
const modules = import.meta.glob<MacroModule>('./modules/*.ts', { eager: true, import: 'default' });

const { environments, macros } = mergeMacroModules(modules);

export { environments, macros };
export type { Environments, Macro, MacroModule, Macros } from './types';
