import type { ViewKind } from './draft';
import type { FieldSpec } from './field-spec';

/** 座標の成分の名前． */
const COORDINATE_NAMES = { plane: ['x', 'y'], space: ['x', 'y', 'z'] } as const;

/** 範囲は，下端と上端の2つの数である． */
const RANGE_LENGTH = 2;

/** 格子の項目．空間の図では，z方向の間隔と範囲(あれば3次元の格子)も持つ． */
function gridFields(kind: ViewKind): readonly FieldSpec[] {
  const names = COORDINATE_NAMES[kind];
  const steps: FieldSpec[] = names.map((name) => ({
    kind: 'bound',
    key: `${name}_step`,
    label: `${name}方向の間隔`,
    optional: true,
  }));
  const ranges: FieldSpec[] = names.map((name) => ({
    kind: 'list',
    key: `${name}_range`,
    label: `${name}の範囲`,
    item: 'number',
    count: RANGE_LENGTH,
    optional: true,
  }));
  return [...steps, ...ranges];
}

export { COORDINATE_NAMES, gridFields };
