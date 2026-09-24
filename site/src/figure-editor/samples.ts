import { parseDraft } from './draft';
import type { SceneDraft } from './draft';
import type { JsonObject } from './json';
import { PLANE_SAMPLE_SCENES, SPACE_SAMPLE_SCENES } from './sample-scenes';
import type { SceneTemplate } from './template-types';

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

/**
 * 見本．そのまま使える，完成した図である．記事の図と，ここで書いた図(`sample-scenes.ts`)があり，
 * 平面の図，空間の図の順に並べる．選ぶと，今の図を置き換える．
 */
const EDITOR_SAMPLES: readonly SceneTemplate[] = [
  articleSample('graphs', '関数のグラフ', 'sine-and-shifted-sine'),
  articleSample('vectors', 'ベクトルの和', 'vector-addition'),
  articleSample('region', '2つのグラフの間の領域', 'sine-cosine-region'),
  articleSample('tangentLine', '接線', 'tangent-line-on-parabola'),
  articleSample('sierpinski', 'シェルピンスキーの三角形', 'sierpinski-triangle'),
  ...PLANE_SAMPLE_SCENES,
  articleSample('sphere', '球と座標軸', 'sphere-with-axes'),
  articleSample('paraboloid', '放物面と座標軸', 'paraboloid-with-axes'),
  articleSample('spaceVectors', '空間のベクトルの和', 'space-vector-addition'),
  articleSample('circles', '球の上の円', 'sphere-with-circles'),
  articleSample('tangentPlane', '接平面', 'tangent-plane-on-paraboloid'),
  articleSample('cone', '円錐と切り口', 'cone-with-cuts'),
  articleSample('cylinder', '球と円柱の交線', 'sphere-and-cylinder'),
  ...SPACE_SAMPLE_SCENES,
];

export { EDITOR_SAMPLES };
