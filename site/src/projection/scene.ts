import patchScene from '@/figures/bezier-patch.json?raw';
import staircaseScene from '@/figures/projection-staircase.json?raw';

import type { Vector3 } from './camera';

/** 解説の図の点Pの座標．`projection-staircase.json`の点Pと同じである． */
const PX = 3;
const PY = 1;
const PZ = 2;
const EXAMPLE_POINT: Vector3 = [PX, PY, PZ];

/** 図の縮尺(1目盛りの長さ)． */
const UNIT = '1.1cm';

interface SceneOptions {
  azimuth: number;
  elevation: number;
  /** ベジエ曲面を描くか． */
  patch: boolean;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

/** 図のファイルを読み，オブジェクトの一覧と，それ以外の項目に分ける． */
function readScene(
  raw: string,
  name: string,
): { rest: Record<string, unknown>; objects: unknown[] } {
  const scene: unknown = JSON.parse(raw);
  if (!isRecord(scene) || !Array.isArray(scene.objects)) {
    throw new Error(`${name}が，シーンの形ではない`);
  }
  const { objects, ...rest } = scene;
  return { rest, objects };
}

/** ベジエ曲面のオブジェクト．解説の図(`bezier-patch.json`)と同じ曲面である． */
function patchObject(): unknown {
  const found = readScene(patchScene, 'bezier-patch.json').objects.find(
    (object) => isRecord(object) && object.id === 'patch',
  );
  if (found === undefined) {
    throw new Error('bezier-patch.jsonに曲面patchがない');
  }
  return found;
}

/**
 * 解説の図に使うシーン(JSON)．解説の図(`projection-staircase.json`)の，座標軸と，点Pへの階段を，指定の向きから描く．
 * ベジエ曲面を加えると，曲面が隠す線は，点線になる．
 */
function projectionScene({ azimuth, elevation, patch }: SceneOptions): string {
  const { rest, objects } = readScene(staircaseScene, 'projection-staircase.json');
  return JSON.stringify({
    ...rest,
    description: `方位角${azimuth}度，仰角${elevation}度で見た，座標軸と点Pへの階段．`,
    view: { azimuth, elevation, unit: UNIT },
    objects: patch ? [...objects, patchObject()] : objects,
  });
}

export { EXAMPLE_POINT, projectionScene };
export type { SceneOptions };
