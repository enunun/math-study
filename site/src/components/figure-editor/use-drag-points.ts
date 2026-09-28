import { useRef, useState } from 'react';
import type { PointerEvent as ReactPointerEvent } from 'react';

import type { SceneDraft } from '@/figure-editor/draft';
import { mathFromSvg, pointNear, withPointAt } from '@/figure-editor/point-drag';

import type { Update } from './parameter-sliders';

/** 点をつかめる，ポインタからの距離(画面のpx)． */
const HIT_PIXELS = 10;

type Handler = (event: ReactPointerEvent<HTMLDivElement>) => void;

interface DragPointHandlers {
  onPointerDown?: Handler;
  onPointerMove?: Handler;
  onPointerUp?: Handler;
  onPointerCancel?: Handler;
}

interface DragPoints {
  handlers: DragPointHandlers;
  /** ポインタが動かせる点の上にあるか(カーソルを変える)． */
  hovering: boolean;
  /** 点を動かしている途中か． */
  dragging: boolean;
}

/** ポインタの位置を，図のSVGの座標(cm)と，画面の1pxあたりのcmにする．SVGがなければ`undefined`． */
function svgPosition(
  event: ReactPointerEvent<HTMLDivElement>,
): { at: [number, number]; cmPerPixel: number } | undefined {
  const svg = event.currentTarget.querySelector('svg');
  const matrix = svg?.getScreenCTM();
  if (matrix === null || matrix === undefined) {
    return undefined;
  }
  const point = new DOMPoint(event.clientX, event.clientY).matrixTransform(matrix.inverse());
  return { at: [point.x, point.y], cmPerPixel: 1 / Math.hypot(matrix.a, matrix.b) };
}

/** ハンドラを作るのに要る，状態と更新の手段． */
interface HandlerContext {
  /** 最新の図(ハンドラを作った後に図が変わっても，最新を読む)． */
  latest: { current: SceneDraft };
  /** つかんでいる点の識別子．つかんでいなければ`undefined`． */
  dragged: string | undefined;
  setDragged: (id: string | undefined) => void;
  setHovering: (hovering: boolean) => void;
  onUpdate: Update;
}

/** ポインタの近くにある，動かせる点の識別子． */
function pointAt(event: ReactPointerEvent<HTMLDivElement>, draft: SceneDraft): string | undefined {
  const position = svgPosition(event);
  return position === undefined
    ? undefined
    : pointNear(draft, position.at, HIT_PIXELS * position.cmPerPixel)?.id;
}

function handlersOf(context: HandlerContext): DragPointHandlers {
  const { latest, dragged, setDragged, setHovering, onUpdate } = context;
  const stop = (): void => {
    setDragged(undefined);
  };
  return {
    onPointerDown(event) {
      const id = pointAt(event, latest.current);
      if (id !== undefined) {
        event.currentTarget.setPointerCapture(event.pointerId);
        event.preventDefault();
        setDragged(id);
      }
    },
    onPointerMove(event) {
      if (dragged === undefined) {
        setHovering(pointAt(event, latest.current) !== undefined);
        return;
      }
      const position = svgPosition(event);
      const at = position === undefined ? undefined : mathFromSvg(latest.current, position.at);
      if (at !== undefined) {
        onUpdate((current) => withPointAt(current, dragged, at));
      }
    },
    onPointerUp: stop,
    onPointerCancel: stop,
  };
}

/**
 * 編集中の図の上で，点をドラッグして動かす．押した所の近くに動かせる点(`point-drag.ts`)があれば，その点を
 * つかみ，動かすたびに位置を図に書き込む．点がなければ何もしない．押した要素が動きを捕える
 * (`setPointerCapture`)ので，ポインタが図の外へ出ても追える．
 */
function useDragPoints({
  enabled,
  draft,
  onUpdate,
}: {
  enabled: boolean;
  draft: SceneDraft;
  onUpdate: Update;
}): DragPoints {
  const latest = useRef(draft);
  latest.current = draft;
  const [dragged, setDragged] = useState<string | undefined>();
  const [hovering, setHovering] = useState(false);
  if (!enabled) {
    return { handlers: {}, hovering: false, dragging: false };
  }
  return {
    handlers: handlersOf({ latest, dragged, setDragged, setHovering, onUpdate }),
    hovering,
    dragging: dragged !== undefined,
  };
}

export { useDragPoints };
