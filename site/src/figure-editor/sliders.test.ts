import { describe, expect, it } from 'vitest';

import { emptyDraft } from './draft';
import type { SceneDraft } from './draft';
import { playedValue, slidersOf, withParameterValue } from './sliders';

function draftWith(objects: SceneDraft['objects']): SceneDraft {
  return { ...emptyDraft('space'), objects };
}

describe('媒介変数のスライダー', () => {
  it('範囲のある媒介変数だけが，並びの順にスライダーになる', () => {
    const draft = draftWith([
      { id: 'a', type: 'parameter', value: 1 },
      { id: 't', type: 'parameter', value: 0.5, range: [-1, 2] },
      { id: 'x_axis', type: 'axis', direction: 'x' },
      { id: 's', type: 'parameter', value: 0, range: [0, 1] },
    ]);
    expect(slidersOf(draft)).toEqual([
      { id: 't', value: 0.5, min: -1, max: 2 },
      { id: 's', value: 0, min: 0, max: 1 },
    ]);
  });

  it('範囲の形が崩れている媒介変数は，スライダーにしない', () => {
    const draft = draftWith([
      { id: 'a', type: 'parameter', value: 0, range: [1] },
      { id: 'b', type: 'parameter', value: 0, range: [1, 0] },
      { id: 'c', type: 'parameter', value: 'x', range: [0, 1] },
    ]);
    expect(slidersOf(draft)).toEqual([]);
  });

  it('値を書き換えると，その媒介変数だけが変わる', () => {
    const draft = draftWith([
      { id: 't', type: 'parameter', value: 0, range: [0, 1] },
      { id: 's', type: 'parameter', value: 0, range: [0, 1] },
    ]);
    const next = withParameterValue(draft, 't', 0.25);
    expect(next.objects).toEqual([
      { id: 't', type: 'parameter', value: 0.25, range: [0, 1] },
      { id: 's', type: 'parameter', value: 0, range: [0, 1] },
    ]);
    expect(draft.objects[0]).toMatchObject({ value: 0 });
  });

  it('書き込む値は，範囲の幅の1万分の1に丸める', () => {
    const draft = draftWith([{ id: 't', type: 'parameter', value: 0, range: [0, 2] }]);
    const next = withParameterValue(draft, 't', 1 / 3);
    expect(next.objects[0]).toMatchObject({ value: 0.3334 });
  });

  it('再生すると，値は始めの値から増え，上端を越えると下端に戻る', () => {
    const slider = { id: 't', value: 0.5, min: 0, max: 2 };
    const period = 4000;
    expect(playedValue(slider, 0, period)).toBeCloseTo(0.5);
    expect(playedValue(slider, 1000, period)).toBeCloseTo(1);
    expect(playedValue(slider, 2900, period)).toBeCloseTo(1.95);
    expect(playedValue(slider, 3500, period)).toBeCloseTo(0.25);
  });
});
