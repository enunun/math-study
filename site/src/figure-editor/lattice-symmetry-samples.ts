import { PLANE_POINT_GROUP_SAMPLES } from './plane-point-group-scenes';
import { PLANE_SYMMETRY_OPERATION_SAMPLES } from './plane-symmetry-operation-scenes';
import { SPACE_POINT_GROUP_SAMPLES } from './space-point-group-scenes';
import { SPACE_SYMMETRY_OPERATION_SAMPLES } from './space-symmetry-operation-scenes';
import { SPACE_SYMMETRY_SAMPLES } from './space-symmetry-sample-scenes';
import { SYMMETRY_SAMPLES } from './symmetry-sample-scenes';
import type { SceneTemplate } from './template-types';

/**
 * 格子の対称性の見本のまとまり．対称操作の種類ごとの見本(回転，鏡映，並進，反転，映進，らせん)と，
 * Bravais格子ごとの点群の見本(平面の5つの格子と，空間の7つの晶系)に分ける．
 */

interface SampleGroup {
  label: string;
  samples: readonly SceneTemplate[];
}

const PLANE_SYMMETRY_GROUPS: readonly SampleGroup[] = [
  {
    label: '格子の対称操作',
    samples: [...SYMMETRY_SAMPLES, ...PLANE_SYMMETRY_OPERATION_SAMPLES],
  },
  { label: '格子の点群', samples: PLANE_POINT_GROUP_SAMPLES },
];

const SPACE_SYMMETRY_GROUPS: readonly SampleGroup[] = [
  {
    label: '格子の対称操作',
    samples: [...SPACE_SYMMETRY_SAMPLES, ...SPACE_SYMMETRY_OPERATION_SAMPLES],
  },
  { label: '格子の点群', samples: SPACE_POINT_GROUP_SAMPLES },
];

export { PLANE_SYMMETRY_GROUPS, SPACE_SYMMETRY_GROUPS };
