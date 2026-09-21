/** 文字列を，ファイルとして，ダウンロードさせる．ブラウザだけで動く． */
function downloadText(name: string, text: string, mime: string): void {
  const url = URL.createObjectURL(new Blob([text], { type: `${mime};charset=utf-8` }));
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = name;
  document.body.append(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
}

/** 文字列を，クリップボードへ写す．できなければ，偽を返す． */
async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}

export { copyText, downloadText };
