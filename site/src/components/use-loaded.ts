import { useEffect, useState } from 'react';

interface Loaded<T> {
  /** 読み込み済みの値．読み込み中，または失敗したときは，未定義である． */
  value: T | undefined;
  /** 読み込みに失敗したかどうか． */
  failed: boolean;
}

/**
 * 非同期に読み込む値(Wasmの関数など)を，最初に表示されたときに読み込む．
 * `load`は，同じ読み込みを繰り返さないもの(モジュールの直下で定義した関数)を渡す．
 */
function useLoaded<T>(load: () => Promise<T>): Loaded<T> {
  // 関数を状態に入れると，Reactが呼び出してしまうので，オブジェクトに包んで持つ．
  const [loaded, setLoaded] = useState<{ value: T }>();
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    let active = true;
    const run = async (): Promise<void> => {
      try {
        const value = await load();
        if (active) {
          setLoaded({ value });
        }
      } catch {
        if (active) {
          setFailed(true);
        }
      }
    };
    void run();
    return (): void => {
      active = false;
    };
  }, [load]);

  return { value: loaded?.value, failed };
}

export { useLoaded };
