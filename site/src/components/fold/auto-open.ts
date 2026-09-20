/** 要素を含む、閉じた折り畳みをすべて開く。 */
function reveal(target: Element | undefined): void {
  let details = target?.closest('details') ?? undefined;
  while (details) {
    details.open = true;
    details = details.parentElement?.closest('details') ?? undefined;
  }
}

/** URL のハッシュを、デコードした要素の id として返す。不正な文字列のときは空文字を返す。 */
function readHashId(): string {
  try {
    return decodeURIComponent(location.hash.slice(1));
  } catch {
    return '';
  }
}

/** URL のハッシュが指す要素が、閉じた折り畳みの中にある場合に開く。 */
function revealHashTarget(): void {
  const id = readHashId();
  if (id) {
    reveal(document.querySelector(`#${CSS.escape(id)}`) ?? undefined);
  }
}

/** 検索でハイライトされた語が、閉じた折り畳みの中にある場合に開く。 */
function revealHighlights(): void {
  for (const mark of document.querySelectorAll('mark[data-pagefind-highlight]')) {
    reveal(mark);
  }
}

/**
 * 折り畳みを、ハッシュ、検索のハイライト、印刷に合わせて開くハンドラを登録する。
 * ハイライトは Pagefind のスクリプトが後から挿入するため、DOM の変化を監視する。
 * 印刷では、閉じた折り畳みの中身も出力されるよう、印刷の間だけすべて開く。
 */
function installAutoOpen(): void {
  revealHashTarget();
  addEventListener('hashchange', revealHashTarget);

  if (location.search.includes('pagefind-highlight')) {
    const observer = new MutationObserver(revealHighlights);
    observer.observe(document.body, { childList: true, subtree: true });
  }

  let openedForPrint: HTMLDetailsElement[] = [];
  addEventListener('beforeprint', () => {
    openedForPrint = [...document.querySelectorAll<HTMLDetailsElement>('details.fold:not([open])')];
    for (const details of openedForPrint) {
      details.open = true;
    }
  });
  addEventListener('afterprint', () => {
    for (const details of openedForPrint) {
      details.open = false;
    }
    openedForPrint = [];
  });
}

export { installAutoOpen, reveal };
