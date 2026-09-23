import { SCENE_VERSION } from './draft';
import type { SceneDraft } from './draft';
import type { JsonObject } from './json';
import type { SceneTemplate } from './template-types';

/** 空間の図の見る向き．既定(方位角60度，仰角20度)より，少し上から見る． */
const SPACE_AZIMUTH = 60;
const SPACE_ELEVATION = 30;
const SPACE_UNIT = '1cm';

/** 空間の図の座標軸と，原点Oの名前．軸は，`range`の範囲に引く． */
function spaceAxes(
  horizontal: readonly [number, number],
  vertical: readonly [number, number],
): JsonObject[] {
  return [
    { id: 'x_axis', type: 'axis', direction: 'x', range: [...horizontal], label: 'x' },
    { id: 'y_axis', type: 'axis', direction: 'y', range: [...horizontal], label: 'y' },
    { id: 'z_axis', type: 'axis', direction: 'z', range: [...vertical], label: 'z' },
    { id: 'origin_label', type: 'label', at: [0, 0, 0], anchor: 'north east', tex: '$O$' },
  ];
}

/** 1周する角度の範囲．継ぎ目(`±pi`)を，見る向きの裏側(x軸の負の側)に置く． */
const ANGLE_DOMAIN = ['-pi', 'pi'];
/** 角度の方向のワイヤーフレームの刻み．30度ごとに断面を引く． */
const ANGLE_STEP = 'pi/6';

/** メビウスの帯の軸の範囲と，幅の方向の刻み． */
const MOBIUS_AXIS = 3;
const MOBIUS_Z_AXIS = 1.5;
const MOBIUS_WIDTH_STEP = 0.5;

/**
 * メビウスの帯．帯の中心の半径2，帯の幅1の，パラメータ表示の曲面．`u`が帯に沿った角度，`v`が幅方向．
 * `u`の端(継ぎ目)は本当の縁ではないので`boundary`は使わず，ただ1本の縁を，`v = 1`の線を
 * 帯に沿って2周させた曲線で描く(1周すると`v = -1`の側に移る)．
 */
const MOBIUS_STRIP_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    'メビウスの帯．半周ひねりながら1周する帯で，表と裏の区別がない(向き付け不可能な)曲面の代表例．縁は1本の閉じた曲線である．',
  view: { azimuth: SPACE_AZIMUTH, elevation: SPACE_ELEVATION, unit: SPACE_UNIT },
  objects: [
    ...spaceAxes([-MOBIUS_AXIS, MOBIUS_AXIS], [-MOBIUS_Z_AXIS, MOBIUS_Z_AXIS]),
    {
      id: 'band',
      type: 'surface',
      vars: ['u', 'v'],
      expr: [
        '(2 + v / 2 * cos(u / 2)) * cos(u)',
        '(2 + v / 2 * cos(u / 2)) * sin(u)',
        'v / 2 * sin(u / 2)',
      ],
      domain: [ANGLE_DOMAIN, [-1, 1]],
      wireframe: {},
      wireframe_step: [ANGLE_STEP, MOBIUS_WIDTH_STEP],
    },
    {
      id: 'edge',
      type: 'curve',
      var: 't',
      expr: ['(2 + cos(t / 2) / 2) * cos(t)', '(2 + cos(t / 2) / 2) * sin(t)', 'sin(t / 2) / 2'],
      domain: ['-pi', '3*pi'],
    },
  ],
};

/** コッホ曲線の見える範囲と，基本図形(線分)の長さ，繰り返す回数． */
const KOCH_VIEW_X_MAX = 5;
const KOCH_VIEW_Y_MAX = 2;
const KOCH_VIEW_Y_MIN = 0.5;
const KOCH_BASE_LENGTH = 4;
const KOCH_MIDPOINT_X = 2;
const KOCH_BUMP_ANGLE = 60;
const KOCH_DEPTH = 5;

/**
 * コッホ曲線．長さ4の線分を，3等分した真ん中を，正三角形の2辺で置き換える変換を，5回繰り返す．
 * 変換は，拡大縮小(1/3)・回転・平行移動を式(分数と`sqrt`)で正確に書く．
 */
const KOCH_CURVE_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    'コッホ曲線．線分を3等分し，真ん中を正三角形の2辺で置き換える操作を繰り返してできる，どこも微分できない曲線．',
  view: {
    x: [-1, KOCH_VIEW_X_MAX],
    y: [-KOCH_VIEW_Y_MIN, KOCH_VIEW_Y_MAX],
    unit: { x: '1.5cm', y: '1.5cm' },
  },
  objects: [
    { id: 'x_axis', type: 'axis', direction: 'x', label: 'x' },
    { id: 'y_axis', type: 'axis', direction: 'y', label: 'y' },
    {
      id: 'koch',
      type: 'fractal',
      base: [
        [0, 0],
        [KOCH_BASE_LENGTH, 0],
      ],
      closed: false,
      transforms: [
        [{ scale: ['1/3', '1/3'] }, { translate: [0, 0] }],
        [{ scale: ['1/3', '1/3'] }, { rotate: KOCH_BUMP_ANGLE }, { translate: ['4/3', 0] }],
        [
          { scale: ['1/3', '1/3'] },
          { rotate: -KOCH_BUMP_ANGLE },
          { translate: [KOCH_MIDPOINT_X, '2*sqrt(3)/3'] },
        ],
        [{ scale: ['1/3', '1/3'] }, { translate: ['8/3', 0] }],
      ],
      depth: KOCH_DEPTH,
      style: { color: 'blue' },
    },
  ],
};

/** 楕円と焦点の図の見える範囲． */
const ELLIPSE_VIEW_X = 4;
const ELLIPSE_VIEW_Y = 3;

/**
 * 楕円x^2/9 + y^2/4 = 1と，2つの焦点(±√5, 0)，楕円の上の点P．Pは，媒介変数`t = pi/3`の点で，
 * 座標を式で書く．Pから2つの焦点までの線分を，色を分けて引く．
 */
const ELLIPSE_FOCI_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '楕円と2つの焦点F，F′．楕円の上の点Pから2つの焦点までの距離の和は，Pによらず一定(長軸の長さ6)である．',
  view: {
    x: [-ELLIPSE_VIEW_X, ELLIPSE_VIEW_X],
    y: [-ELLIPSE_VIEW_Y, ELLIPSE_VIEW_Y],
    unit: { x: '1cm', y: '1cm' },
  },
  objects: [
    { id: 'x_axis', type: 'axis', direction: 'x', label: 'x' },
    { id: 'y_axis', type: 'axis', direction: 'y', label: 'y' },
    { id: 'origin_label', type: 'label', at: [0, 0], anchor: 'north east', tex: '$O$' },
    {
      id: 'ellipse',
      type: 'curve',
      var: 't',
      expr: ['3*cos(t)', '2*sin(t)'],
      domain: [0, '2*pi'],
      style: { color: 'blue' },
    },
    { id: 'F', type: 'point', at: ['sqrt(5)', 0], dot: true, label: 'F', anchor: 'north' },
    { id: 'G', type: 'point', at: ['-sqrt(5)', 0], dot: true, label: "F'", anchor: 'north' },
    {
      id: 'P',
      type: 'point',
      at: ['3*cos(pi/3)', '2*sin(pi/3)'],
      dot: true,
      label: 'P',
      anchor: 'south west',
    },
    { id: 'PF', type: 'segment', from: 'P', to: 'F', style: { color: 'red' } },
    { id: 'PG', type: 'segment', from: 'P', to: 'G', style: { color: 'red' } },
  ],
};

/** トーラスの軸の範囲．管の中心の半径2，管の半径0.7のトーラスを，軸が貫く． */
const TORUS_AXIS = 4;
const TORUS_Z_AXIS = 2;
/** 管を回る方向の刻み．裏側の断面も点線で描くので，60度ごとに減らす． */
const TORUS_TUBE_STEP = 'pi/3';

/** トーラス．閉じた曲面なので，定義域の端はどれも継ぎ目で，縁を描かない． */
const TORUS_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    'トーラス．円を，同じ平面の上にあって円と交わらない直線のまわりに回した曲面．点線は，経線と緯線にあたる断面である．',
  view: { azimuth: SPACE_AZIMUTH, elevation: SPACE_ELEVATION, unit: SPACE_UNIT },
  objects: [
    ...spaceAxes([-TORUS_AXIS, TORUS_AXIS], [-TORUS_Z_AXIS, TORUS_Z_AXIS]),
    {
      id: 'torus',
      type: 'surface',
      vars: ['u', 'v'],
      expr: ['(2 + 0.7*cos(v))*cos(u)', '(2 + 0.7*cos(v))*sin(u)', '0.7*sin(v)'],
      domain: [ANGLE_DOMAIN, ANGLE_DOMAIN],
      wireframe: {},
      wireframe_step: [ANGLE_STEP, TORUS_TUBE_STEP],
    },
  ],
};

/**
 * 円柱の図の軸の範囲と，円柱の高さ，高さの方向の刻み．円柱の半径は1.5．切り口やらせんが
 * 見えやすいように，高さの方向の断面は少なくする．
 */
const CYLINDER_AXIS = 3;
const CYLINDER_Z_AXIS_MIN = -1;
const CYLINDER_Z_AXIS_MAX = 4;
const CYLINDER_HEIGHT = 3;
const CYLINDER_HEIGHT_STEP = 1;

/** 円柱の曲面(側面)．角度の端は継ぎ目なので`boundary`を使わず，上下の縁を曲線で描く． */
function cylinderObjects(): JsonObject[] {
  return [
    {
      id: 'cylinder',
      type: 'surface',
      vars: ['t', 'z'],
      expr: ['1.5*cos(t)', '1.5*sin(t)', 'z'],
      domain: [ANGLE_DOMAIN, [0, CYLINDER_HEIGHT]],
      wireframe: {},
      wireframe_step: [ANGLE_STEP, CYLINDER_HEIGHT_STEP],
    },
    {
      id: 'bottom_rim',
      type: 'curve',
      var: 't',
      expr: ['1.5*cos(t)', '1.5*sin(t)', '0'],
      domain: ANGLE_DOMAIN,
    },
    {
      id: 'top_rim',
      type: 'curve',
      var: 't',
      expr: ['1.5*cos(t)', '1.5*sin(t)', `${CYLINDER_HEIGHT}`],
      domain: ANGLE_DOMAIN,
    },
  ];
}

/** 切る平面z - y/2 = 1.5．円柱の軸に斜めなので，切り口は楕円になる． */
const CUT_NORMAL_Y = -0.5;
const CUT_OFFSET = 1.5;

const CYLINDER_CUT_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '円柱と，軸に斜めな平面による切り口．切り口は楕円である．円柱の裏側にある部分は点線で描く．',
  view: { azimuth: SPACE_AZIMUTH, elevation: SPACE_ELEVATION, unit: SPACE_UNIT },
  objects: [
    ...spaceAxes([-CYLINDER_AXIS, CYLINDER_AXIS], [CYLINDER_Z_AXIS_MIN, CYLINDER_Z_AXIS_MAX]),
    ...cylinderObjects(),
    {
      id: 'slanted_cut',
      type: 'cut',
      surface: 'cylinder',
      normal: [0, CUT_NORMAL_Y, 1],
      offset: CUT_OFFSET,
      style: { color: 'red' },
    },
  ],
};

const HELIX_SCENE: SceneDraft = {
  version: SCENE_VERSION,
  description:
    '円柱の上を2周するらせん．1周ごとに，高さが一定の割合で上がる．円柱の裏側にある部分は点線で描く．',
  view: { azimuth: SPACE_AZIMUTH, elevation: SPACE_ELEVATION, unit: SPACE_UNIT },
  objects: [
    ...spaceAxes([-CYLINDER_AXIS, CYLINDER_AXIS], [CYLINDER_Z_AXIS_MIN, CYLINDER_Z_AXIS_MAX]),
    ...cylinderObjects(),
    {
      id: 'helix',
      type: 'curve',
      var: 't',
      expr: ['1.5*cos(t)', '1.5*sin(t)', `${CYLINDER_HEIGHT}*t/(4*pi)`],
      domain: [0, '4*pi'],
      style: { color: 'blue' },
    },
  ],
};

/** 記事の図とは別に，ここで書いた見本．`samples.ts`が，記事の図の見本と並べる． */
const PLANE_SAMPLE_SCENES: readonly SceneTemplate[] = [
  { id: 'ellipseFoci', label: '楕円と焦点', scene: ELLIPSE_FOCI_SCENE },
  { id: 'koch', label: 'コッホ曲線', scene: KOCH_CURVE_SCENE },
];

const SPACE_SAMPLE_SCENES: readonly SceneTemplate[] = [
  { id: 'cylinderCut', label: '円柱と平面の切り口', scene: CYLINDER_CUT_SCENE },
  { id: 'helix', label: '円柱の上のらせん', scene: HELIX_SCENE },
  { id: 'torus', label: 'トーラス', scene: TORUS_SCENE },
  { id: 'mobius', label: 'メビウスの帯', scene: MOBIUS_STRIP_SCENE },
];

export { PLANE_SAMPLE_SCENES, SPACE_SAMPLE_SCENES };
