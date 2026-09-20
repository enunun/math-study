/** 折り畳み(details 要素)と，インラインの補足(.detail)の，どちらも指すセレクタ． */
const HOLDERS = 'details, .detail';

/** 折り畳みまたは補足を，開いた状態または閉じた状態にする． */
function setOpen(holder: Element, open: boolean): void {
  if (holder instanceof HTMLDetailsElement) {
    holder.open = open;
    return;
  }
  holder.classList.toggle('is-open', open);
  holder.querySelector(':scope > .detail-toggle')?.setAttribute('aria-expanded', String(open));
}

/** 要素を含む，閉じた折り畳みと補足をすべて開く． */
function reveal(target: Element | undefined): void {
  let holder = target?.closest(HOLDERS) ?? undefined;
  while (holder) {
    setOpen(holder, true);
    holder = holder.parentElement?.closest(HOLDERS) ?? undefined;
  }
}

/** 補足の切り替えボタンが押されたとき，その補足を開閉する． */
function toggleDetail(event: Event): void {
  const target = event.target instanceof Element ? event.target : undefined;
  const detail = target?.closest('.detail-toggle')?.closest('.detail');
  if (detail) {
    setOpen(detail, !detail.classList.contains('is-open'));
  }
}

/** URLのハッシュを，デコードした要素のidとして返す．不正な文字列のときは空文字を返す． */
function readHashId(): string {
  try {
    return decodeURIComponent(location.hash.slice(1));
  } catch {
    return '';
  }
}

/** URLのハッシュが指す要素が，閉じた折り畳みや補足の中にある場合に開く． */
function revealHashTarget(): void {
  const id = readHashId();
  if (id) {
    reveal(document.querySelector(`#${CSS.escape(id)}`) ?? undefined);
  }
}

/** 検索でハイライトされた語が，閉じた折り畳みや補足の中にある場合に開く． */
function revealHighlights(): void {
  for (const mark of document.querySelectorAll('mark[data-pagefind-highlight]')) {
    reveal(mark);
  }
}

/**
 * 折り畳みと補足を，切り替えボタン，ハッシュ，検索のハイライト，印刷に合わせて開閉するハンドラを登録する．
 * ハイライトはPagefindのスクリプトが後から挿入するため，DOMの変化を監視する．
 * 印刷では，閉じた折り畳みや補足の中身も出力されるよう，印刷の間だけすべて開く．
 */
function installAutoOpen(): void {
  document.addEventListener('click', toggleDetail);
  revealHashTarget();
  addEventListener('hashchange', revealHashTarget);

  if (location.search.includes('pagefind-highlight')) {
    const observer = new MutationObserver(revealHighlights);
    observer.observe(document.body, { childList: true, subtree: true });
  }

  let openedForPrint: Element[] = [];
  addEventListener('beforeprint', () => {
    openedForPrint = [
      ...document.querySelectorAll('details.fold:not([open]), .detail:not(.is-open)'),
    ];
    for (const holder of openedForPrint) {
      setOpen(holder, true);
    }
  });
  addEventListener('afterprint', () => {
    for (const holder of openedForPrint) {
      setOpen(holder, false);
    }
    openedForPrint = [];
  });
}

export { installAutoOpen, reveal };
