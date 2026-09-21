import bezier from '@/figures/bezier-patch.json?raw';
import cone from '@/figures/cone-with-cuts.json?raw';
import graphs from '@/figures/sine-and-shifted-sine.json?raw';
import region from '@/figures/sine-cosine-region.json?raw';
import cylinder from '@/figures/sphere-and-cylinder.json?raw';
import sphereAxes from '@/figures/sphere-with-axes.json?raw';
import vectors from '@/figures/vector-addition.json?raw';

/** 編集の出発点にする見本．サイトの記事で使っている図である． */
interface EditorSample {
  id: string;
  label: string;
  json: string;
}

const EDITOR_SAMPLES: readonly EditorSample[] = [
  { id: 'graphs', label: '関数のグラフ', json: graphs },
  { id: 'vectors', label: 'ベクトルの和', json: vectors },
  { id: 'region', label: '2つのグラフの間の領域', json: region },
  { id: 'sphere', label: '球と座標軸', json: sphereAxes },
  { id: 'cone', label: '円錐と切り口', json: cone },
  { id: 'cylinder', label: '球と円柱の交線', json: cylinder },
  { id: 'bezier', label: 'ベジエ曲面', json: bezier },
];

export { EDITOR_SAMPLES };
export type { EditorSample };
