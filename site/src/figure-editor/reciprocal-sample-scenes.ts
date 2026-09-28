import { BRILLOUIN_SAMPLE } from './brillouin-scene';
import { EWALD_BRAGG_SAMPLE } from './ewald-bragg-scene';
import { RECIPROCAL_FCC_SAMPLE } from './fcc-reciprocal-scene';
import { RECIPROCAL_PLANE_SCENES } from './reciprocal-plane-scenes';
import { RECIPROCAL_TRICLINIC_SAMPLE } from './reciprocal-space-scenes';
import type { SceneTemplate } from './template-types';

/**
 * 逆格子の見本．平面では，実格子と逆格子を並べた図，格子面の族と逆格子点の図，Ewald球とBragg反射の図，
 * 空間では，三斜晶の逆格子と，面心立方格子の逆格子(体心立方格子になる)．
 */
const RECIPROCAL_PLANE_SAMPLES: readonly SceneTemplate[] = [
  ...RECIPROCAL_PLANE_SCENES,
  BRILLOUIN_SAMPLE,
  EWALD_BRAGG_SAMPLE,
];

const RECIPROCAL_SPACE_SAMPLES: readonly SceneTemplate[] = [
  RECIPROCAL_TRICLINIC_SAMPLE,
  RECIPROCAL_FCC_SAMPLE,
];

export { RECIPROCAL_PLANE_SAMPLES, RECIPROCAL_SPACE_SAMPLES };
