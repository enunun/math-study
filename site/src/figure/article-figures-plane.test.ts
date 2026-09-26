import { describe, expect, it } from 'vitest';

import { CM_PER_PT } from './svg';
import {
  TOLERANCE,
  bottomEnd,
  bounds,
  circleCenter,
  dashed,
  draw,
  expectNear,
  label,
  leftEnd,
  nth,
  tip,
  widthPt,
} from './test-support';
import type { Point } from './test-support';

/**
 * 記事に置いた平面の図(site/src/figures/)の形を確かめる．
 * 図は，ビルドと同じく，Wasmで描いてHTMLの木にする．座標は，SVGと同じく，単位がcmで，yが下向きである．
 * ブラウザでの見え方(ラベルの箱の置き方，実寸，色)は，e2e/figure.spec.tsが確かめる．
 */
describe('平面の図', () => {
  describe('y=sin xのグラフ(sine-and-shifted-sine)', () => {
    const drawn = draw('sine-and-shifted-sine');

    it('SVGは，画像として説明を持ち，軸2本と曲線2本，軸の矢じり2つを描く', () => {
      expect(drawn.svg.properties.role).toBe('img');
      expect(drawn.svg.properties.ariaLabel).toMatch(/sin x のグラフ/u);
      expect(drawn.paths).toHaveLength(4);
      expect(drawn.polygons).toHaveLength(2);
    });

    it('元の曲線は実線で，平行移動した曲線は点線である', () => {
      expect(dashed(nth(drawn.paths, 2))).toBe(false);
      expect(nth(drawn.paths, 3).properties.strokeDasharray).toMatch(/^[\d.]+ [\d.]+$/u);
    });

    it('yの名前は，y軸の上の端に置き，箱の下の辺の真ん中を合わせる', () => {
      // 軸の線は矢じりの手前で止まるので，矢じりの先端を，軸の端とする．
      const y = label(drawn, '$y$');
      expectNear(y.at, tip(nth(drawn.polygons, 1)));
      expect(y.anchor).toBe('south');
    });

    it('Oの名前は，原点に置き，箱の左上の角を合わせる', () => {
      const origin: Point = [bounds(nth(drawn.paths, 1)).x, bounds(nth(drawn.paths, 0)).y];
      const o = label(drawn, '$O$');
      expectNear(o.at, origin);
      expect(o.anchor).toBe('north west');
    });
  });

  describe('目盛(sine-with-ticks)', () => {
    const drawn = draw('sine-with-ticks');

    it('目盛の線は，軸に直角な，3ptずつ両側に出る短い線である', () => {
      // x軸1本，目盛4本，y軸1本，目盛2本，sin x．
      expect(drawn.paths).toHaveLength(9);
      const xTick = bounds(nth(drawn.paths, 1));
      expect(xTick.width).toBe(0);
      expect(xTick.height).toBeCloseTo(2 * 3 * CM_PER_PT, 4);
      const yTick = bounds(nth(drawn.paths, 6));
      expect(yTick.height).toBe(0);
      expect(yTick.width).toBeCloseTo(2 * 3 * CM_PER_PT, 4);
    });

    it('目盛の名前は，anchorに従い，x軸の目盛では下の端に，y軸の目盛では左の端に置く', () => {
      // `north east`の名前は，目盛の左下に延び，`north west`の名前は，右下に延びる．
      const pi = label(drawn, String.raw`$\pi$`);
      expectNear(pi.at, bottomEnd(nth(drawn.paths, 3)));
      expect(pi.anchor).toBe('north east');
      const twoPi = label(drawn, String.raw`$2\pi$`);
      expectNear(twoPi.at, bottomEnd(nth(drawn.paths, 4)));
      expect(twoPi.anchor).toBe('north west');
      // 目盛の名前は，x軸より下にある．
      expect(pi.at[1]).toBeGreaterThan(bounds(nth(drawn.paths, 0)).y);
      const one = label(drawn, '$1$');
      expectNear(one.at, leftEnd(nth(drawn.paths, 6)));
      expect(one.anchor).toBe('east');
    });
  });

  describe('格子(parabola-on-grid)', () => {
    const drawn = draw('parabola-on-grid');

    it('格子は，細い点線で，1cmおきに，見える範囲の端から端まで引く', () => {
      // 縦7本，横7本，軸2本，曲線2本．
      expect(drawn.paths).toHaveLength(18);
      // 格子の点線14本と，破線の接線．
      expect(drawn.paths.filter((path) => dashed(path))).toHaveLength(15);
      const first = bounds(nth(drawn.paths, 0));
      const second = bounds(nth(drawn.paths, 1));
      expect(second.x - first.x).toBeCloseTo(1, 5);
      expect(first.height).toBeCloseTo(6, 5);
    });

    it('曲線の頂は，格子の上の辺に届き，それより上へ出ない', () => {
      const top = bounds(nth(drawn.paths, 13));
      const curve = bounds(nth(drawn.paths, 16));
      expect(curve.y).toBeCloseTo(top.y, 5);
    });

    it('線の太さは，指定した長さで描く．放物線は1.2pt，格子は0.3ptである', () => {
      expect(widthPt(nth(drawn.paths, 16))).toBeCloseTo(1.2, 2);
      expect(widthPt(nth(drawn.paths, 0))).toBeCloseTo(0.3, 2);
    });

    it('色の名前は，CSSの変数にする．放物線は青，接線は赤，格子は灰色，軸は文字の色である', () => {
      expect(nth(drawn.paths, 16).properties.stroke).toBe('var(--figure-blue)');
      expect(nth(drawn.paths, 17).properties.stroke).toBe('var(--figure-red)');
      expect(nth(drawn.paths, 0).properties.stroke).toBe('var(--figure-gray)');
      expect(nth(drawn.paths, 14).properties.stroke).toBe('currentColor');
    });
  });

  describe('ベクトルの和(vector-addition)', () => {
    const drawn = draw('vector-addition');

    it('破線の辺2本と，ベクトル3本，矢じり3つ，点の印1つを描く', () => {
      expect(drawn.paths.map((path) => dashed(path))).toEqual([true, true, false, false, false]);
      expect(drawn.polygons).toHaveLength(3);
      expect(drawn.circles).toHaveLength(1);
    });

    it('対角線の矢じりの先端は，Dの印の中心にあり，Dの名前は，印の右上にある', () => {
      const dot = circleCenter(nth(drawn.circles, 0));
      expectNear(tip(nth(drawn.polygons, 2)), dot);
      const d = label(drawn, '$D$');
      expectNear(d.at, dot);
      expect(d.anchor).toBe('south west');
    });
  });

  describe('領域(sine-cosine-region)', () => {
    const drawn = draw('sine-cosine-region');

    it('領域は，青い薄い色で塗り，線は引かない', () => {
      const filled = drawn.paths.filter((path) => path.properties.fillOpacity !== undefined);
      expect(filled).toHaveLength(1);
      expect(nth(filled, 0).properties).toMatchObject({
        fill: 'var(--figure-blue)',
        stroke: 'none',
        fillOpacity: '0.25',
      });
    });

    it('斜線は，0.4ptの45度の線で，2つのグラフの交点(-3π/4とπ/4)の間に収まる', () => {
      const hatches = drawn.paths.filter(
        (path) => path.properties.stroke !== 'none' && Math.abs(widthPt(path) - 0.4) < 1e-3,
      );
      expect(hatches.length).toBeGreaterThanOrEqual(6);
      const originX = bounds(nth(drawn.paths, 1)).x;
      for (const hatch of hatches) {
        const box = bounds(hatch);
        expect(box.x).toBeGreaterThanOrEqual(originX - 0.75 * Math.PI - TOLERANCE);
        expect(box.x + box.width).toBeLessThanOrEqual(originX + 0.25 * Math.PI + TOLERANCE);
        expect(box.width).toBeCloseTo(box.height, 4);
      }
    });
  });
});
