import { describe, expect, it } from 'vitest';

import {
  TOLERANCE,
  bounds,
  center,
  circleCenter,
  dashed,
  distance,
  draw,
  expectNear,
  label,
  nth,
  points,
  tip,
  widthPt,
} from './test-support';
import type { Box } from './test-support';

/**
 * 記事に置いた空間の図(site/src/figures/)の形を確かめる．
 * 図は，ビルドと同じく，Wasmで描いてHTMLの木にする．座標は，SVGと同じく，単位がcmで，yが下向きである．
 * ブラウザでの見え方(ラベルの箱の置き方，実寸，色)は，e2e/figure.spec.tsが確かめる．
 */
describe('空間の図', () => {
  describe('球と座標軸(sphere-with-axes)', () => {
    const drawn = draw('sphere-with-axes');
    const outline = (): Box => bounds(nth(drawn.paths, 9));

    it('SVGは，画像として説明を持ち，3つに分かれた軸3本と，球の輪郭と，見える先端の矢じり3つを描く', () => {
      expect(drawn.svg.properties.role).toBe('img');
      expect(drawn.svg.properties.ariaLabel).toMatch(/半径2の球/u);
      expect(drawn.paths).toHaveLength(10);
      expect(drawn.polygons).toHaveLength(3);
    });

    it('球に隠れた部分だけが点線で，輪郭と，見える部分は実線である', () => {
      expect(drawn.paths.map((path) => dashed(path))).toEqual([
        false,
        true,
        false,
        false,
        true,
        false,
        false,
        true,
        false,
        false,
      ]);
    });

    it('球の輪郭は，半径1.6cmの円で，中心は，z軸の上の原点にある', () => {
      expect(outline().width).toBeCloseTo(3.2, 3);
      expect(outline().height).toBeCloseTo(3.2, 3);
      const zAxis = bounds(nth(drawn.paths, 6));
      expect(center(outline())[0]).toBeCloseTo(center(zAxis)[0], 3);
    });

    it('軸の点線の部分は，球の輪郭の内側にある', () => {
      const middle = center(outline());
      for (const index of [1, 4, 7]) {
        for (const point of points(nth(drawn.paths, index))) {
          expect(distance(point, middle)).toBeLessThanOrEqual(1.6 + TOLERANCE);
        }
      }
    });

    it('x，z，Oの名前は，軸の見える先端と原点に置き，軸の外の側へ延ばす', () => {
      // x軸の先は，画面の左下へ向かう．名前の箱の右の辺を，軸の端に合わせる．
      const x = label(drawn, '$x$');
      expectNear(x.at, tip(nth(drawn.polygons, 0)));
      expect(x.anchor).toBe('east');
      const z = label(drawn, '$z$');
      expectNear(z.at, tip(nth(drawn.polygons, 2)));
      expect(z.anchor).toBe('south');
      const o = label(drawn, '$O$');
      expectNear(o.at, center(outline()));
      expect(o.anchor).toBe('north east');
    });
  });

  describe('球の面の上の円(sphere-with-circles)', () => {
    const drawn = draw('sphere-with-circles');

    it('円は，遠い側が点線で，球の輪郭に外接する正方形の内側にある', () => {
      // 軸9つ，円2つが3つずつ，球の輪郭1本．
      expect(drawn.paths).toHaveLength(16);
      expect(drawn.paths.filter((path) => dashed(path))).toHaveLength(5);
      const outline = bounds(nth(drawn.paths, 15));
      for (const index of [10, 13]) {
        const hidden = bounds(nth(drawn.paths, index));
        expect(hidden.x).toBeGreaterThanOrEqual(outline.x - TOLERANCE);
        expect(hidden.y).toBeGreaterThanOrEqual(outline.y - TOLERANCE);
        expect(hidden.x + hidden.width).toBeLessThanOrEqual(outline.x + outline.width + TOLERANCE);
        expect(hidden.y + hidden.height).toBeLessThanOrEqual(
          outline.y + outline.height + TOLERANCE,
        );
      }
    });
  });

  describe('式で書いた曲面(paraboloid-with-axes)', () => {
    const drawn = draw('paraboloid-with-axes');

    it('輪郭と縁を0.8ptの実線で描き，曲面に隠れた軸を点線で描く', () => {
      expect(drawn.polygons).toHaveLength(3);
      expect(drawn.paths.some((path) => dashed(path))).toBe(true);
      const outlines = drawn.paths.filter(
        (path) => !dashed(path) && Math.abs(widthPt(path) - 0.8) < 1e-3,
      );
      expect(outlines.length).toBeGreaterThanOrEqual(2);
    });
  });

  describe('切り口(cone-with-cuts)', () => {
    const drawn = draw('cone-with-cuts');

    it('切り口は，色を付けた線で描き，曲面に隠れる部分は点線になる', () => {
      const blue = drawn.paths.filter((path) => path.properties.stroke === 'var(--figure-blue)');
      const red = drawn.paths.filter((path) => path.properties.stroke === 'var(--figure-red)');
      expect(blue.length).toBeGreaterThanOrEqual(2);
      expect(red.length).toBeGreaterThanOrEqual(2);
      expect(blue.some((path) => dashed(path))).toBe(true);
      expect(blue.some((path) => !dashed(path))).toBe(true);
    });
  });

  describe('空間のベクトルの和(space-vector-addition)', () => {
    const drawn = draw('space-vector-addition');

    it('軸3本，破線の辺2本，ベクトル3本を描き，対角線の矢じりの先端が，Dの印の中心にある', () => {
      expect(drawn.paths).toHaveLength(8);
      // 矢じりは，軸とベクトルの6つ．
      expect(drawn.polygons).toHaveLength(6);
      expect(drawn.circles).toHaveLength(1);
      expectNear(tip(nth(drawn.polygons, 5)), circleCenter(nth(drawn.circles, 0)));
    });
  });

  describe('曲面どうしの交線(sphere-and-cylinder)', () => {
    const drawn = draw('sphere-and-cylinder');

    it('交線は，1.2ptの赤い線で，隠れる部分が点線になる', () => {
      const red = drawn.paths.filter((path) => path.properties.stroke === 'var(--figure-red)');
      expect(red.length).toBeGreaterThanOrEqual(2);
      expect(red.some((path) => dashed(path))).toBe(true);
      expect(red.some((path) => !dashed(path))).toBe(true);
      for (const path of red) {
        expect(widthPt(path)).toBeCloseTo(1.2, 2);
      }
    });
  });

  describe('ベジエ曲面(bezier-patch)', () => {
    const drawn = draw('bezier-patch');

    it('縁と輪郭を実線で描き，曲面が隠す軸を点線にする', () => {
      expect(drawn.svg.properties.ariaLabel).toMatch(/ベジエ曲面/u);
      expect(drawn.paths.some((path) => dashed(path))).toBe(true);
      expect(drawn.paths.some((path) => !dashed(path))).toBe(true);
    });
  });
});
