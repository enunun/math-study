import { useMemo, useState } from 'react';
import type { ReactElement } from 'react';

import { useLoaded } from '@/components/use-loaded';
import { SCENE_SAMPLES } from '@/figure/samples';
import { loadSceneEngine } from '@/figure/wasm';
import type { SceneEngine, SceneOutcome } from '@/figure/wasm';

import './scene-checker.css';

type Failure = Extract<SceneOutcome, { status: 'error' }>;
type Success = Extract<SceneOutcome, { status: 'ok' }>;

const FIRST_SAMPLE = SCENE_SAMPLES[0]?.json ?? '';

/** 誤りの説明と，種類，場所． */
function ErrorMessage({ failure }: { failure: Failure }): ReactElement {
  return (
    <div className="scene-error" role="alert">
      <p>{failure.message}</p>
      <p>
        種類：<code>{failure.code}</code>
      </p>
      {failure.object !== null && (
        <p>
          オブジェクト：<code>{failure.object}</code>
        </p>
      )}
      {failure.line !== null && failure.column !== null && (
        <p>
          位置：{failure.line}行{failure.column}列
        </p>
      )}
      {failure.field !== null && failure.index !== null && failure.start !== null && (
        <p>
          式：<code>{failure.field}</code>の{failure.index + 1}番目の式の{failure.start + 1}文字目
        </p>
      )}
    </div>
  );
}

/** オブジェクトの，idと種類の一覧． */
function ObjectTable({ objects }: { objects: Success['objects'] }): ReactElement {
  if (objects.length === 0) {
    return <p>オブジェクトはない．</p>;
  }
  return (
    <table>
      <caption>オブジェクト</caption>
      <thead>
        <tr>
          <th scope="col">id</th>
          <th scope="col">種類</th>
        </tr>
      </thead>
      <tbody>
        {objects.map((object) => (
          <tr key={object.id}>
            <td>
              <code>{object.id}</code>
            </td>
            <td>{object.type}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

/** 折り畳みの中に，長い文字列を，キーボードでスクロールできる形で置く． */
function Folded({ title, text }: { title: string; text: string }): ReactElement {
  return (
    <details>
      <summary>{title}</summary>
      <div role="region" aria-label={title}>
        <pre tabIndex={0}>{text}</pre>
      </div>
    </details>
  );
}

/** 読み込めたシーンの，版，説明，オブジェクトの一覧，読み直したJSON，TikZ． */
function Summary({ result, tikz }: { result: Success; tikz: string | null }): ReactElement {
  return (
    <div className="scene-summary">
      <p>シーンを読み込めた．</p>
      {/* Starlightは，兄弟の要素に上の余白を付けるので，見出しと値がずれる．除外する． */}
      <dl className="not-content">
        <dt>版</dt>
        <dd>{result.version}</dd>
        <dt>エンジンの版</dt>
        <dd>{result.engineVersion}</dd>
        <dt>説明</dt>
        <dd>{result.description}</dd>
      </dl>
      <ObjectTable objects={result.objects} />
      <Folded title="読み直したJSON" text={result.canonical} />
      {tikz !== null && <Folded title="TikZ" text={tikz} />}
    </div>
  );
}

interface InputProps {
  json: string;
  invalid: boolean;
  onChange: (json: string) => void;
}

/** シーンの入力欄と，見本のボタン． */
function SceneInput({ json, invalid, onChange }: InputProps): ReactElement {
  return (
    <>
      <div className="scene-input">
        <label htmlFor="scene-input">シーン(JSON)</label>
        <textarea
          id="scene-input"
          value={json}
          rows={16}
          onChange={(event) => {
            onChange(event.target.value);
          }}
          autoComplete="off"
          autoCapitalize="off"
          spellCheck={false}
          aria-invalid={invalid}
        />
      </div>
      <div className="scene-samples">
        <span>見本：</span>
        {SCENE_SAMPLES.map((sample) => (
          <button
            key={sample.id}
            type="button"
            onClick={() => {
              onChange(sample.json);
            }}
          >
            {sample.label}
          </button>
        ))}
      </div>
    </>
  );
}

interface OutputProps {
  ready: boolean;
  loadFailed: boolean;
  blank: boolean;
  outcome: SceneOutcome | undefined;
  tikz: string | null;
}

/** 読み込みの状態に応じた，結果，誤り，案内の表示． */
function SceneOutput({ ready, loadFailed, blank, outcome, tikz }: OutputProps): ReactElement {
  return (
    <div className="scene-output" aria-live="polite">
      {loadFailed && <p role="alert">シーンの読み込みを準備できなかった．ページを開き直す．</p>}
      {!loadFailed && !ready && <p role="status">準備している．</p>}
      {ready && blank && <p>シーンのJSONを入力すると，結果が表示される．</p>}
      {outcome?.status === 'error' && <ErrorMessage failure={outcome} />}
      {outcome?.status === 'ok' && <Summary result={outcome} tikz={tikz} />}
    </div>
  );
}

/**
 * 図のシーンの確認．入力したJSONを，RustのWasmで読み，検査して，結果か誤りを表示する．
 * 読み込みと検査は，Wasmが行い，このコンポーネントは，入力と表示だけを担う．
 */
function SceneChecker(): ReactElement {
  const [json, setJson] = useState(FIRST_SAMPLE);
  const { value: engine, failed } = useLoaded<SceneEngine>(loadSceneEngine);
  const blank = json.trim() === '';
  const outcome = useMemo(
    () => (engine === undefined || blank ? undefined : engine.parseScene(json)),
    [engine, blank, json],
  );
  // 読み込めたシーンだけを描画して，TikZを取り出す．
  const tikz = useMemo(() => {
    if (engine === undefined || outcome?.status !== 'ok') {
      return null;
    }
    const rendered = engine.renderScene(json);
    return rendered.status === 'ok' ? rendered.tikz : null;
  }, [engine, outcome, json]);

  return (
    <div className="scene-checker">
      <SceneInput json={json} invalid={outcome?.status === 'error'} onChange={setJson} />
      <SceneOutput
        ready={engine !== undefined}
        loadFailed={failed}
        blank={blank}
        outcome={outcome}
        tikz={tikz}
      />
    </div>
  );
}

export default SceneChecker;
