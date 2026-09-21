import first from '@/figures/sine-and-shifted-sine.json?raw';
import space from '@/figures/sphere-with-axes.json?raw';

/** 確認のページに置く，シーンの見本． */
interface SceneSample {
  id: string;
  /** ボタンに表示する名前． */
  label: string;
  json: string;
}

/** 見本の元の文章の一部を置き換える．元の文章が変わって，置き換え先がなくなったときは，見本が意味を失うので，例外にする． */
function replaceOnce(source: string, from: string, to: string): string {
  if (!source.includes(from)) {
    throw new Error(`見本の元に「${from}」がない`);
  }
  return source.replace(from, () => to);
}

const SCENE_SAMPLES: SceneSample[] = [
  { id: 'first', label: '最初の図', json: first },
  {
    id: 'newer',
    label: '新しい版',
    json: replaceOnce(first, '"version": "0.1.0"', '"version": "99.0.0"'),
  },
  {
    id: 'unknownField',
    label: '未知の項目',
    json: replaceOnce(first, '"direction": "x"', '"direction": "x", "colour": "red"'),
  },
  {
    id: 'expression',
    label: '式の誤り',
    json: replaceOnce(first, '"sin(x - shift)"', '"sin(x - shift"'),
  },
  {
    id: 'duplicateId',
    label: 'idの重なり',
    json: replaceOnce(first, '"id": "shifted_sine"', '"id": "sine"'),
  },
  {
    id: 'syntax',
    label: 'JSONの構文の誤り',
    json: replaceOnce(first, '"description": "', '"description" "'),
  },
  {
    id: 'range',
    label: '範囲の誤り',
    json: replaceOnce(first, '"x": [-7, 7]', '"x": [7, -7]'),
  },
  { id: 'space', label: '球と軸', json: space },
  {
    id: 'missingRange',
    label: '空間の軸の範囲がない',
    json: replaceOnce(space, '"range": [-5, 5], "label": "x"', '"label": "x"'),
  },
];

export { SCENE_SAMPLES };
export type { SceneSample };
