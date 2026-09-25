import { describe, expect, it } from 'vitest';

import { minimalObject, validate } from './scene-schema-support';

/** 空間の図の，スタイルだけを変えた曲線のシーン． */
function curveWithStyle(style: Record<string, unknown>): unknown {
  return {
    ...minimalObject('space'),
    objects: [
      {
        id: 'k',
        type: 'curve',
        var: 't',
        expr: ['sin(t)', 'cos(t)', 't'],
        domain: [0, 1],
        style,
      },
    ],
  };
}

describe('シーンのJSON Schemaは，陰線処理の書式を表す', () => {
  it('陰線処理をしない線(hiddenのvisible)と，奥を通る所で切る線(crossing_gap)を受け入れる', () => {
    expect(
      validate(curveWithStyle({ hidden: 'visible', crossing_gap: '2pt' })),
      JSON.stringify(validate.errors),
    ).toBe(true);
  });

  it('crossing_gapが単位つきの長さでなければ断る', () => {
    expect(validate(curveWithStyle({ crossing_gap: 2 }))).toBe(false);
    expect(validate(curveWithStyle({ crossing_gap: '2' }))).toBe(false);
  });

  it('hiddenの値が決まった名前でなければ断る', () => {
    expect(validate(curveWithStyle({ hidden: 'shown' }))).toBe(false);
  });
});
