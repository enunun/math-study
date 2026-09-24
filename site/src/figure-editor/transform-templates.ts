import type { ViewKind } from './draft';
import type { JsonObject } from './json';

/**
 * 変換の手順のテンプレート．フォームの変換の欄で選ぶと，手順の並びの最後に足す．数は，足したあとに
 * JSONの欄で直す．写像(`map`)は，図の中の写像を指すので，別に作る(`mapStep`)．
 */
interface TransformTemplate {
  id: string;
  label: string;
  step: JsonObject;
}

const PLANE_TRANSFORMS: readonly TransformTemplate[] = [
  { id: 'translate', label: '平行移動', step: { translate: [1, 0] } },
  { id: 'rotate', label: '回転(原点のまわり)', step: { rotate: 90 } },
  { id: 'rotateAbout', label: '回転(点のまわり)', step: { rotate: 90, center: [1, 0] } },
  { id: 'scale', label: '拡大縮小', step: { scale: 2 } },
  { id: 'reflectX', label: 'x軸に関する対称移動', step: { reflect: [1, 0] } },
  { id: 'reflectY', label: 'y軸に関する対称移動', step: { reflect: [0, 1] } },
  { id: 'reflectDiagonal', label: '直線y = xに関する対称移動', step: { reflect: [1, 1] } },
  { id: 'pointSymmetry', label: '原点に関する対称移動', step: { scale: -1 } },
  { id: 'shear', label: 'せん断', step: { shear: [1, 0] } },
];

const SPACE_TRANSFORMS: readonly TransformTemplate[] = [
  { id: 'translate', label: '平行移動', step: { translate: [1, 0, 0] } },
  { id: 'rotateX', label: 'x軸のまわりの回転', step: { rotate: 90, axis: [1, 0, 0] } },
  { id: 'rotateY', label: 'y軸のまわりの回転', step: { rotate: 90, axis: [0, 1, 0] } },
  { id: 'rotateZ', label: 'z軸のまわりの回転', step: { rotate: 90, axis: [0, 0, 1] } },
  { id: 'scale', label: '拡大縮小', step: { scale: 2 } },
  { id: 'reflectXY', label: 'xy平面に関する対称移動', step: { reflect: [0, 0, 1] } },
  { id: 'reflectYZ', label: 'yz平面に関する対称移動', step: { reflect: [1, 0, 0] } },
  { id: 'reflectZX', label: 'zx平面に関する対称移動', step: { reflect: [0, 1, 0] } },
  { id: 'pointSymmetry', label: '原点に関する対称移動', step: { scale: -1 } },
];

/** 図の種類ごとの，変換の手順のテンプレート． */
function transformTemplates(kind: ViewKind): readonly TransformTemplate[] {
  return kind === 'space' ? SPACE_TRANSFORMS : PLANE_TRANSFORMS;
}

/** 写像`id`で写す手順． */
function mapStep(id: string): JsonObject {
  return { map: id };
}

export { mapStep, transformTemplates };
export type { TransformTemplate };
