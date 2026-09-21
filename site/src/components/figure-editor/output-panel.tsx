import { useState } from 'react';
import type { ReactElement } from 'react';

import { parseDraft, stringifyDraft } from '@/figure-editor/draft';
import type { SceneDraft } from '@/figure-editor/draft';

interface JsonProps {
  draft: SceneDraft;
  onChange: (draft: SceneDraft) => void;
}

/** シーンのJSONを，直接編集する．読めない間は，図を直前のままにして，理由を示す． */
function JsonPanel({ draft, onChange }: JsonProps): ReactElement {
  const [text, setText] = useState(() => stringifyDraft(draft));
  const [message, setMessage] = useState('');
  return (
    <div className="fe-panel">
      <label className="fe-field fe-wide">
        <span>シーン(JSON)</span>
        <textarea
          rows={18}
          value={text}
          spellCheck={false}
          autoComplete="off"
          aria-invalid={message !== ''}
          onChange={(event) => {
            setText(event.target.value);
            const parsed = parseDraft(event.target.value);
            setMessage(parsed.ok ? '' : parsed.message);
            if (parsed.ok) {
              onChange(parsed.draft);
            }
          }}
        />
      </label>
      {message !== '' && <p role="alert">JSONを読めない：{message}</p>}
    </div>
  );
}

interface TikzProps {
  /** 図を描けているときの，TikZ．描けていなければ，空の文字列． */
  tikz: string;
}

/** 生成したTikZ．読み取り専用で示す． */
function TikzPanel({ tikz }: TikzProps): ReactElement {
  if (tikz === '') {
    return <p role="status">図を描けるようになると，TikZが表示される．</p>;
  }
  return (
    <div className="fe-panel">
      <label className="fe-field fe-wide">
        <span>TikZ</span>
        <textarea rows={18} readOnly value={tikz} spellCheck={false} />
      </label>
      <p>
        自分のTeX文書に入れるには，プリアンブルに
        <code>{String.raw`\usepackage{tikz}`}</code>，
        <code>{String.raw`\usetikzlibrary{arrows.meta}`}</code>，
        <code>{String.raw`\usepackage{amsmath}`}</code>
        を書く．日本語のラベルには，LuaLaTeXとLuaTeX-jaを使う．
      </p>
    </div>
  );
}

export { JsonPanel, TikzPanel };
