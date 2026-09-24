import { useState } from 'react';
import type { ReactElement } from 'react';

import { downloadBlob } from '@/figure-editor/download';
import {
  DEFAULT_RESOLUTION,
  IMAGE_FORMATS,
  RESOLUTIONS,
  isImageFormat,
  isTransparent,
} from '@/figure-editor/image-formats';
import type { ImageOptions } from '@/figure-editor/image-formats';
import type { Figure } from '@/wasm/figure';

import { CheckboxInput, SelectInput } from './field-inputs';

interface Props {
  /** 描けているときの，図の中間表現．描けていなければ，`undefined`． */
  figure: Figure | undefined;
  /** 拡張子を除いた，ファイルの名前． */
  stem: string;
  onMessage: (message: string) => void;
}

const FORMAT_OPTIONS = Object.entries(IMAGE_FORMATS).map(
  ([format, spec]) => [format, spec.label] as const,
);
const RESOLUTION_OPTIONS = RESOLUTIONS.map(
  (resolution) => [String(resolution), `${resolution}dpi`] as const,
);

/** 失敗の理由．MathJaxは，Errorでない値で失敗を知らせることもあるので，`message`を持つかで読む． */
function reasonOf(error: unknown): string {
  if (typeof error === 'object' && error !== null && 'message' in error) {
    return String(error.message);
  }
  return String(error);
}

/** 画像を作って，ダウンロードさせる．画像を作る部分(MathJaxのSVG出力を含む)は，ここで初めて読み込む． */
async function saveImage(
  figure: Figure,
  options: ImageOptions,
  { stem, onMessage }: Props,
): Promise<void> {
  const spec = IMAGE_FORMATS[options.format];
  try {
    const { exportImage } = await import('@/figure-editor/image-export');
    downloadBlob(`${stem}.${spec.extension}`, await exportImage(figure, options));
    onMessage(`${spec.label}を書き出した．`);
  } catch (error) {
    onMessage(`${spec.label}を書き出せなかった：${reasonOf(error)}`);
  }
}

/** 形式，解像度(画素の画像だけ)，背景の透過の選択． */
function ImageOptionInputs({
  options,
  onChange,
}: {
  options: ImageOptions;
  onChange: (options: ImageOptions) => void;
}): ReactElement {
  const spec = IMAGE_FORMATS[options.format];
  return (
    <>
      <SelectInput
        label="画像の形式"
        value={options.format}
        options={FORMAT_OPTIONS}
        onChange={(value) => {
          if (isImageFormat(value)) {
            onChange({ ...options, format: value });
          }
        }}
      />
      {spec.raster && (
        <SelectInput
          label="解像度"
          value={String(options.resolution)}
          options={RESOLUTION_OPTIONS}
          onChange={(value) => {
            onChange({ ...options, resolution: Number(value) });
          }}
        />
      )}
      <CheckboxInput
        label={spec.transparency ? '背景を透過する' : '背景を透過する(JPEGは透過できない)'}
        checked={isTransparent(options)}
        disabled={!spec.transparency}
        onChange={(transparent) => {
          onChange({ ...options, transparent });
        }}
      />
    </>
  );
}

const INITIAL_OPTIONS: ImageOptions = {
  format: 'png',
  transparent: false,
  resolution: DEFAULT_RESOLUTION,
};

/** 画像(SVG，PNG，JPEG，WebP)の書き出し．透過できる形式では，背景を透過するかを選べる． */
function ImageExportControls(props: Props): ReactElement {
  const { figure } = props;
  const [options, setOptions] = useState<ImageOptions>(INITIAL_OPTIONS);
  const [busy, setBusy] = useState(false);
  const save = async (drawn: Figure): Promise<void> => {
    setBusy(true);
    await saveImage(drawn, options, props);
    setBusy(false);
  };
  return (
    <div className="fe-buttons fe-image-export" role="group" aria-label="画像の書き出し">
      <ImageOptionInputs options={options} onChange={setOptions} />
      <button
        type="button"
        disabled={figure === undefined || busy}
        onClick={() => {
          if (figure !== undefined) {
            void save(figure);
          }
        }}
      >
        画像を書き出す(.{IMAGE_FORMATS[options.format].extension})
      </button>
    </div>
  );
}

export { ImageExportControls };
