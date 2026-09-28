import { useEffect, useRef, useState } from 'react';
import type { ReactElement } from 'react';

import type { SceneDraft } from '@/figure-editor/draft';
import { playedValue, withParameterValue } from '@/figure-editor/sliders';
import type { Slider } from '@/figure-editor/sliders';

/** 再生で，範囲を1周するのにかける時間(ミリ秒)． */
const PLAY_PERIOD = 8000;
/** スライダーの刻みは，範囲の幅のこの数分の1である． */
const SLIDER_DIVISIONS = 1000;
/** 値を示す桁数(有効数字)． */
const SHOWN_DIGITS = 4;

type Update = (change: (draft: SceneDraft) => SceneDraft) => void;

/**
 * `slider`を再生する．始めたときの値から一定の速さで増やし，上端を越えると下端に戻る．
 * 描画はフレームごとに1回で，描画が重ければ，そのぶんコマを飛ばす．
 */
function usePlayback(slider: Slider | undefined, onUpdate: Update): void {
  const latest = useRef(slider);
  latest.current = slider;
  const id = slider?.id ?? '';
  useEffect((): (() => void) => {
    const start = latest.current;
    // 0は，どのフレームの番号でもない(取り消しても何も起きない)．
    let frame = 0;
    if (start !== undefined) {
      const begin = performance.now();
      frame = requestAnimationFrame(function step(now: number): void {
        const value = playedValue(start, now - begin, PLAY_PERIOD);
        onUpdate((draft) => withParameterValue(draft, start.id, value));
        frame = requestAnimationFrame(step);
      });
    }
    return (): void => {
      cancelAnimationFrame(frame);
    };
  }, [id, onUpdate]);
}

interface RowProps {
  slider: Slider;
  playing: boolean;
  onValue: (value: number) => void;
  onPlay: (playing: boolean) => void;
}

function SliderRow({ slider, playing, onValue, onPlay }: RowProps): ReactElement {
  const { id, value, min, max, step } = slider;
  return (
    <div className="fe-slider">
      <label>
        <span className="fe-slider-name">{id}</span>
        <input
          type="range"
          min={min}
          max={max}
          step={step ?? (max - min) / SLIDER_DIVISIONS}
          value={value}
          onChange={(event) => {
            onValue(event.currentTarget.valueAsNumber);
          }}
        />
      </label>
      <output className="fe-slider-value">{Number(value.toPrecision(SHOWN_DIGITS))}</output>
      <button
        type="button"
        aria-pressed={playing}
        aria-label={`${id}を${playing ? '止める' : '再生する'}`}
        onClick={() => {
          onPlay(!playing);
        }}
      >
        {playing ? '停止' : '再生'}
      </button>
    </div>
  );
}

interface Props {
  /** 範囲のある媒介変数(1つ以上)． */
  sliders: readonly Slider[];
  onUpdate: Update;
}

/**
 * 範囲(`range`)のある媒介変数のスライダー．動かすと，値を図に書き込む．「再生」で，値を範囲の中で
 * 繰り返し動かす(時間のように使う)．再生できるのは，一度に1つの媒介変数である．
 */
function ParameterSliders({ sliders, onUpdate }: Props): ReactElement {
  const [playingId, setPlayingId] = useState('');
  const playing = sliders.find((slider) => slider.id === playingId);
  usePlayback(playing, onUpdate);
  return (
    <div className="fe-sliders" role="group" aria-label="媒介変数のスライダー">
      {sliders.map((slider) => (
        <SliderRow
          key={slider.id}
          slider={slider}
          playing={slider.id === playing?.id}
          onValue={(value) => {
            setPlayingId('');
            onUpdate((current) => withParameterValue(current, slider.id, value));
          }}
          onPlay={(play) => {
            setPlayingId(play ? slider.id : '');
          }}
        />
      ))}
    </div>
  );
}

export { ParameterSliders };
export type { Update };
