import { describe, expect, it } from 'vitest';

import { IMAGE_FORMATS, isImageFormat, isTransparent, pixelSize } from './image-formats';

describe('画像の形式', () => {
  it('SVG，PNG，JPEG，WebPを書き出せ，JPEGだけは透過できない', () => {
    expect(Object.keys(IMAGE_FORMATS)).toEqual(['svg', 'png', 'jpeg', 'webp']);
    expect(IMAGE_FORMATS.jpeg.extension).toBe('jpg');
    expect(isTransparent({ format: 'png', transparent: true })).toBe(true);
    expect(isTransparent({ format: 'svg', transparent: false })).toBe(false);
    expect(isTransparent({ format: 'jpeg', transparent: true })).toBe(false);
  });

  it('知らない形式の名前は，形式として認めない', () => {
    expect(isImageFormat('webp')).toBe(true);
    expect(isImageFormat('gif')).toBe(false);
    expect(isImageFormat('toString')).toBe(false);
  });

  it('実寸と解像度から画素の数を決め，上限と下限に収める', () => {
    // 2.54cmは1インチなので，300dpiで300画素である．
    expect(pixelSize(2.54, 300)).toBe(300);
    expect(pixelSize(0, 300)).toBe(1);
    expect(pixelSize(1000, 600)).toBe(16_384);
  });
});
