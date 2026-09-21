import type { Environments, Macros, MacroModule } from './types';

/** 種類(マクロか環境か)ごとの，まとめの一覧と，名前の持ち主． */
interface Registry<T> {
  kind: string;
  /** 名前から，それを定義したモジュールの名前． */
  owners: Map<string, string>;
  values: Record<string, T>;
}

function createRegistry<T>(kind: string): Registry<T> {
  return { kind, owners: new Map(), values: {} };
}

/** 1つのモジュールの定義を，一覧に加える．名前が重なると，持ち主を添えて，誤りにする． */
function addDefinitions<T>(
  registry: Registry<T>,
  definitions: Record<string, T>,
  owner: string,
): void {
  for (const [name, value] of Object.entries(definitions)) {
    const previous = registry.owners.get(name);
    if (previous !== undefined) {
      throw new Error(
        `${registry.kind}「${name}」が，${previous}と${owner}で重複している．どちらかの名前を変える．`,
      );
    }
    registry.owners.set(name, owner);
    registry.values[name] = value;
  }
}

/**
 * 分野ごとのモジュールを，1つのマクロの一覧と，環境の一覧にまとめる．
 * 同じ名前を，複数のモジュールが定義していると，誤りにする．後の定義が，黙って前の定義を上書きすることはない．
 * @param modules モジュールの名前(ファイルのパスなど)をキーにした，モジュールの一覧．
 */
function mergeMacroModules(modules: Record<string, MacroModule>): {
  macros: Macros;
  environments: Environments;
} {
  const macros = createRegistry<Macros[string]>('マクロ');
  const environments = createRegistry<Environments[string]>('環境');
  // 読み込みの順序に依存しないよう，モジュールの名前の順に処理する．
  const sorted = Object.entries(modules).toSorted(([a], [b]) => a.localeCompare(b));
  for (const [owner, module] of sorted) {
    addDefinitions(macros, module.macros, owner);
    addDefinitions(environments, module.environments ?? {}, owner);
  }
  return { macros: macros.values, environments: environments.values };
}

export { mergeMacroModules };
