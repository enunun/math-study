import { ANALYSIS_SAMPLE_SCENES } from './analysis-sample-scenes';
import { parseDraft } from './draft';
import type { SceneDraft } from './draft';
import { FOUR_SPACE_SAMPLE_SCENES } from './four-space-sample-scenes';
import type { JsonObject } from './json';
import {
  ELLIPSE_FOCI_SAMPLE,
  PLANE_CRYSTAL_SAMPLES,
  SPACE_CRYSTAL_SAMPLES,
  SPACE_CURVE_SAMPLES,
  SURFACE_SAMPLES,
} from './sample-scenes';
import { SPACE_SYMMETRY_SAMPLES } from './space-symmetry-sample-scenes';
import { SYMMETRY_SAMPLES } from './symmetry-sample-scenes';
import type { SceneTemplate } from './template-types';
import { SPACE_VECTOR_FIELD_SAMPLES, VECTOR_FIELD_SAMPLES } from './vector-field-sample-scenes';

const files = import.meta.glob<string>('../figures/*.json', {
  eager: true,
  import: 'default',
  query: '?raw',
});

/**
 * 曲面には縁(`boundary`)とワイヤーフレームを，球にはワイヤーフレームを描かせる．見本で，
 * どちらの項目も有効な状態から始められるようにするためである．すでにある指定は変えない．
 */
function withSurfaceAids(object: JsonObject): JsonObject {
  if (object.type === 'surface') {
    return { ...object, boundary: true, wireframe: object.wireframe ?? {} };
  }
  if (object.type === 'sphere') {
    return { ...object, wireframe: object.wireframe ?? {} };
  }
  return object;
}

function withAllSurfaceAids(scene: SceneDraft): SceneDraft {
  return { ...scene, objects: scene.objects.map(withSurfaceAids) };
}

/**
 * 記事の図(`site/src/figures/*.json`)を，見本として読む．記事の図を作った手順を見せるため，
 * 記事と同じファイルを使い，曲面と球の縁とワイヤーフレームだけを足す(`withSurfaceAids`)．
 */
function articleSample(id: string, label: string, file: string): SceneTemplate {
  const json = files[`../figures/${file}.json`];
  if (json === undefined) {
    throw new Error(`図の見本が見つからない：${file}`);
  }
  const parsed = parseDraft(json);
  if (!parsed.ok) {
    throw new Error(`図の見本を読めない：${file}：${parsed.message}`);
  }
  return { id, label, scene: withAllSurfaceAids(parsed.draft) };
}

/** 見本のまとまり．見本の一覧は，平面・空間と分野で分けて見せる． */
interface SampleGroup {
  label: string;
  samples: readonly SceneTemplate[];
}

/**
 * 見本．そのまま使える，完成した図である．記事の図と，ここで書いた図(`sample-scenes.ts`)があり，
 * 平面の図，空間の図の順に，分野ごとにまとめる．選ぶと，今の図を置き換える．
 */
const SAMPLE_GROUPS: readonly SampleGroup[] = [
  {
    label: '平面：関数とグラフ',
    samples: [
      articleSample('graphs', '関数のグラフ', 'sine-and-shifted-sine'),
      articleSample('region', '2つのグラフの間の領域', 'sine-cosine-region'),
      articleSample('tangentLine', '接線', 'tangent-line-on-parabola'),
      ...ANALYSIS_SAMPLE_SCENES,
    ],
  },
  {
    label: '平面：図形',
    samples: [
      articleSample('vectors', 'ベクトルの和', 'vector-addition'),
      ELLIPSE_FOCI_SAMPLE,
      articleSample('sierpinski', 'Sierpińskiの三角形', 'sierpinski-triangle'),
    ],
  },
  { label: '平面：ベクトル場', samples: VECTOR_FIELD_SAMPLES },
  { label: '平面：格子の対称性', samples: SYMMETRY_SAMPLES },
  { label: '平面：結晶と逆格子', samples: PLANE_CRYSTAL_SAMPLES },
  {
    label: '空間：曲面',
    samples: [
      articleSample('sphere', '球と座標軸', 'sphere-with-axes'),
      articleSample('paraboloid', '放物面と座標軸', 'paraboloid-with-axes'),
      articleSample('circles', '球の上の円', 'sphere-with-circles'),
      articleSample('tangentPlane', '接平面', 'tangent-plane-on-paraboloid'),
      articleSample('cone', '円錐と切り口', 'cone-with-cuts'),
      articleSample('cylinder', '球と円柱の交線', 'sphere-and-cylinder'),
      ...SURFACE_SAMPLES,
    ],
  },
  {
    label: '空間：曲線とベクトル',
    samples: [
      articleSample('spaceVectors', '空間のベクトルの和', 'space-vector-addition'),
      ...SPACE_CURVE_SAMPLES,
    ],
  },
  { label: '空間：4次元', samples: FOUR_SPACE_SAMPLE_SCENES },
  { label: '空間：ベクトル場', samples: SPACE_VECTOR_FIELD_SAMPLES },
  { label: '空間：格子の対称性', samples: SPACE_SYMMETRY_SAMPLES },
  { label: '空間：結晶と逆格子', samples: SPACE_CRYSTAL_SAMPLES },
];

/** 見本の全部(まとまりの順)． */
const EDITOR_SAMPLES: readonly SceneTemplate[] = SAMPLE_GROUPS.flatMap((group) => group.samples);

export { EDITOR_SAMPLES, SAMPLE_GROUPS };
export type { SampleGroup };
