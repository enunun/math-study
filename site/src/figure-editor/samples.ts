const files = import.meta.glob<string>('../figures/*.json', {
  eager: true,
  import: 'default',
  query: '?raw',
});

/** 見本の図のJSONを，そのソースから読む． */
function jsonOf(file: string): string {
  const json = files[`../figures/${file}.json`];
  if (json === undefined) {
    throw new Error(`図の見本が見つからない：${file}`);
  }
  return json;
}

/** 編集の出発点にする見本．サイトの記事で使っている図である． */
interface EditorSample {
  id: string;
  label: string;
  json: string;
}

const SAMPLE_FILES: readonly { id: string; label: string; file: string }[] = [
  { id: 'graphs', label: '関数のグラフ', file: 'sine-and-shifted-sine' },
  { id: 'vectors', label: 'ベクトルの和', file: 'vector-addition' },
  { id: 'region', label: '2つのグラフの間の領域', file: 'sine-cosine-region' },
  { id: 'fractal', label: 'フラクタル', file: 'sierpinski-triangle' },
  { id: 'sphere', label: '球と座標軸', file: 'sphere-with-axes' },
  { id: 'cone', label: '円錐と切り口', file: 'cone-with-cuts' },
  { id: 'cylinder', label: '球と円柱の交線', file: 'sphere-and-cylinder' },
  { id: 'bezier', label: 'ベジエ曲面', file: 'bezier-patch' },
  { id: 'bezierCurve', label: 'ベジエ曲線', file: 'bezier-curve-control-polygon' },
  { id: 'splineCurve', label: 'スプライン曲線', file: 'spline-curve-through-points' },
  { id: 'tangentLine', label: '接線', file: 'tangent-line-on-parabola' },
  { id: 'paraboloid', label: '放物面と座標軸', file: 'paraboloid-with-axes' },
  { id: 'spaceVectors', label: '空間のベクトルの和', file: 'space-vector-addition' },
  { id: 'circles', label: '球の上の円', file: 'sphere-with-circles' },
  { id: 'tangentPlane', label: '接平面', file: 'tangent-plane-on-paraboloid' },
];

const EDITOR_SAMPLES: readonly EditorSample[] = SAMPLE_FILES.map(({ id, label, file }) => ({
  id,
  label,
  json: jsonOf(file),
}));

export { EDITOR_SAMPLES };
export type { EditorSample };
