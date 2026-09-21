import { useMemo, useState } from 'react';
import type { ReactElement } from 'react';

import { MathView } from '@/components/calculator/math-view';
import { renderFigure } from '@/components/figure/render-figure';
import type { Rendered } from '@/components/figure/render-figure';
import { useLoaded } from '@/components/use-loaded';
import { loadSceneEngine } from '@/figure/wasm';
import { camera, screenOf } from '@/projection/camera';
import type { Camera, Vector3 } from '@/projection/camera';
import { EXAMPLE_POINT, projectionScene } from '@/projection/scene';

import './projection-explorer.css';

const INITIAL_AZIMUTH = 60;
const INITIAL_ELEVATION = 20;
const AZIMUTH_LIMIT = 180;
const ELEVATION_LIMIT = 85;
/** 行列に書く数の，小数点以下の桁数． */
const DIGITS = 2;

/** 小数にする．-0は0にする． */
function num(value: number): string {
  const text = value.toFixed(DIGITS);
  return Number(text) === 0 ? '0' : text;
}

function row(vector: Vector3): string {
  return vector.map((value) => num(value)).join(' & ');
}

function column(vector: Vector3): string {
  return vector.map((value) => num(value)).join(String.raw` \\ `);
}

/** 行列の式(TeX)と，読み上げ用の平文． */
function matrixTexts(view: Camera): { basis: [string, string]; product: [string, string] } {
  const { r, u, d } = view;
  const { x, y, depth } = screenOf(view, EXAMPLE_POINT);
  const result: Vector3 = [x, y, depth];
  return {
    basis: [
      String.raw`M = \begin{bmatrix} ${row(r)} \\ ${row(u)} \\ ${row(d)} \end{bmatrix}`,
      `行列Mの3つの行は，r=(${row(r).replaceAll(' & ', ', ')})，u=(${row(u).replaceAll(' & ', ', ')})，d=(${row(d).replaceAll(' & ', ', ')})`,
    ],
    product: [
      String.raw`M\begin{bmatrix} ${column(EXAMPLE_POINT)} \end{bmatrix} = \begin{bmatrix} ${column(result)} \end{bmatrix}`,
      `点Pの位置ベクトル(3, 1, 2)の画面の位置は(${num(x)}, ${num(y)})，奥行きは${num(depth)}`,
    ],
  };
}

interface SliderProps {
  label: string;
  value: number;
  limit: number;
  onChange: (value: number) => void;
}

function Slider({ label, value, limit, onChange }: SliderProps): ReactElement {
  return (
    <label className="projection-slider">
      <span>{label}</span>
      <input
        type="range"
        min={-limit}
        max={limit}
        step={1}
        value={value}
        onChange={(event) => {
          onChange(Number(event.target.value));
        }}
      />
      <output>{value}°</output>
    </label>
  );
}

/**
 * 方位角と仰角を動かして，同じ図と，行列$M$の数値が，どう変わるかを見る．
 * 図は，Rustのエンジンをブラウザで動かして描き，ラベルは，ブラウザのMathJaxで組む．
 */
function ProjectionExplorer(): ReactElement {
  const [azimuth, setAzimuth] = useState(INITIAL_AZIMUTH);
  const [elevation, setElevation] = useState(INITIAL_ELEVATION);
  const [patch, setPatch] = useState(false);
  const engine = useLoaded(loadSceneEngine);
  const rendered = useMemo<Rendered | undefined>(
    () =>
      engine.value === undefined
        ? undefined
        : renderFigure(engine.value, projectionScene({ azimuth, elevation, patch })),
    [engine.value, azimuth, elevation, patch],
  );
  const texts = matrixTexts(camera(azimuth, elevation));
  return (
    <div className="projection-explorer">
      <div className="projection-figure">
        {rendered?.figure}
        {rendered?.failure !== undefined && <p role="alert">{rendered.failure.message}</p>}
        {engine.failed && <p role="alert">図の読み込みに失敗した．</p>}
        {rendered === undefined && !engine.failed && <p>図を読み込んでいる．</p>}
      </div>
      <div className="projection-controls">
        <Slider label="方位角 a" value={azimuth} limit={AZIMUTH_LIMIT} onChange={setAzimuth} />
        <Slider label="仰角 e" value={elevation} limit={ELEVATION_LIMIT} onChange={setElevation} />
        <label className="projection-check">
          <input
            type="checkbox"
            checked={patch}
            onChange={(event) => {
              setPatch(event.target.checked);
            }}
          />
          <span>自由な曲面（ベジエ曲面）を加える</span>
        </label>
        <MathView tex={texts.basis[0]} label={texts.basis[1]} />
        <MathView tex={texts.product[0]} label={texts.product[1]} />
      </div>
    </div>
  );
}

export { ProjectionExplorer };
