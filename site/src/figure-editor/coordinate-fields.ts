import type { ViewKind } from './draft';
import { ARROW_LENGTHS, PIVOTS } from './field-options';
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

/**
 * ベクトル場の項目．格子点の刻みと範囲は，図の次元の数だけある(平面の図では範囲を省ける)．
 */
function vectorFieldFields(kind: ViewKind): readonly FieldSpec[] {
  const names = COORDINATE_NAMES[kind];
  const steps: FieldSpec[] = names.map((name) => ({
    kind: 'bound',
    key: `${name}_step`,
    label: `${name}の刻み`,
  }));
  const ranges: FieldSpec[] = names.map((name) => ({
    kind: 'list',
    key: `${name}_range`,
    label: `${name}の範囲`,
    item: 'number',
    count: RANGE_LENGTH,
    optional: kind === 'plane',
  }));
  return [
    { kind: 'position', key: 'field', label: '場の式(ベクトルの式か成分)' },
    { kind: 'text', key: 'var', label: '位置ベクトルの名前(既定はr)', optional: true },
    ...steps,
    ...ranges,
    { kind: 'select', key: 'length', label: '矢印の長さ', options: ARROW_LENGTHS, optional: true },
    { kind: 'bound', key: 'scale', label: '倍率(そろえるときは長さ)', optional: true },
    { kind: 'bound', key: 'max_length', label: '長さの上限', optional: true },
    { kind: 'select', key: 'pivot', label: '矢印の位置', options: PIVOTS, optional: true },
  ];
}

export { COORDINATE_NAMES, gridFields, vectorFieldFields };
