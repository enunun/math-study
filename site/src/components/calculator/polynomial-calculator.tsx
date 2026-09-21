import { useMemo, useState } from 'react';
import type { ReactElement } from 'react';

import { splitAt } from '@/calculator/highlight';
import type { Outcome } from '@/calculator/wasm';

import { MathView } from './math-view';

import './polynomial-calculator.css';
import { useCalculate } from './use-calculate';

const EXAMPLES = [
  '(x+1)^2',
  '(x+1)(x-1)',
  '(a+b)^3',
  '(x+y)^2 - (x-y)^2',
  'x^2/2 + 0.5x - 3',
] as const;

type Failure = Extract<Outcome, { status: 'error' }>;
type Success = Extract<Outcome, { status: 'ok' }>;

/** 誤りの説明と，入力の中の場所． */
function ErrorMessage({ source, failure }: { source: string; failure: Failure }): ReactElement {
  const { before, hit, after } = splitAt(source, failure.start, failure.end);
  return (
    <div className="calculator-error" role="alert">
      <p>{failure.message}</p>
      <p>
        {failure.start + 1}文字目：
        <code>
          {before}
          <mark>{hit === '' ? '␣' : hit}</mark>
          {after}
        </code>
      </p>
    </div>
  );
}

/** 展開と微分の結果． */
function Results({ result }: { result: Success }): ReactElement {
  return (
    <div className="calculator-results">
      <h3>展開</h3>
      <MathView tex={result.expanded.tex} label={result.expanded.text} />
      <p>
        テキスト：<code>{result.expanded.text}</code>
      </p>
      {result.derivatives.map((derivative) => (
        <section key={derivative.variable}>
          <h3>{derivative.variable}による偏微分</h3>
          <MathView tex={derivative.tex} label={derivative.text} />
          <p>
            テキスト：<code>{derivative.text}</code>
          </p>
        </section>
      ))}
    </div>
  );
}

interface InputProps {
  source: string;
  invalid: boolean;
  onChange: (source: string) => void;
}

/** 式の入力欄と，使える記号の説明，例のボタン． */
function CalculatorInput({ source, invalid, onChange }: InputProps): ReactElement {
  return (
    <>
      <div className="calculator-input">
        <label htmlFor="polynomial-input">式</label>
        <input
          id="polynomial-input"
          type="text"
          value={source}
          onChange={(event) => {
            onChange(event.target.value);
          }}
          autoComplete="off"
          autoCapitalize="off"
          spellCheck={false}
          aria-describedby="polynomial-help"
          aria-invalid={invalid}
        />
      </div>
      <p id="polynomial-help" className="calculator-help">
        使える記号は，+，-，*，/，^と括弧である．変数は英字1文字で，掛け算の記号は，2xや(x+1)(x-1)のように省略できる．
        割る数は，定数にする．全角でも入力できる．
      </p>
      <div className="calculator-examples">
        <span>例：</span>
        {EXAMPLES.map((example) => (
          <button
            key={example}
            type="button"
            onClick={() => {
              onChange(example);
            }}
          >
            {example}
          </button>
        ))}
      </div>
    </>
  );
}

/** 計算機の状態に応じた，結果，誤り，読み込み中の表示． */
function CalculatorOutput({
  ready,
  loadFailed,
  source,
  outcome,
}: {
  ready: boolean;
  loadFailed: boolean;
  source: string;
  outcome: Outcome | undefined;
}): ReactElement {
  return (
    <div className="calculator-output" aria-live="polite">
      {loadFailed && <p role="alert">計算機を読み込めなかった．ページを開き直す．</p>}
      {!loadFailed && !ready && <p role="status">計算機を読み込んでいる．</p>}
      {ready && source.trim() === '' && <p>式を入力すると，結果が表示される．</p>}
      {outcome?.status === 'error' && <ErrorMessage source={source} failure={outcome} />}
      {outcome?.status === 'ok' && <Results result={outcome} />}
    </div>
  );
}

/**
 * 多項式の計算機．入力した式を，RustのWasmで展開し，変数ごとに偏微分して，結果を数式で表示する．
 * 式の解析と計算は，Wasmが行い，このコンポーネントは，入力と表示だけを担う．
 */
function PolynomialCalculator(): ReactElement {
  const [source, setSource] = useState<string>(EXAMPLES[0]);
  const { calculate, loadFailed } = useCalculate();
  const outcome = useMemo(
    () => (calculate === undefined || source.trim() === '' ? undefined : calculate(source)),
    [calculate, source],
  );

  return (
    <div className="calculator">
      <CalculatorInput source={source} invalid={outcome?.status === 'error'} onChange={setSource} />
      <CalculatorOutput
        ready={calculate !== undefined}
        loadFailed={loadFailed}
        source={source}
        outcome={outcome}
      />
    </div>
  );
}

export default PolynomialCalculator;
