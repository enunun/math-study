import { useRef } from 'react';
import type { PointerEvent as ReactPointerEvent } from 'react';

/** ドラッグの移動量(px)を，方位角と仰角の移動量(度)にする比率． */
const DEGREES_PER_PIXEL = 0.4;

/** 直前のポインタの位置と，ドラッグ中かどうか． */
interface DragState {
  x: number;
  y: number;
  active: boolean;
}

interface DragRotateOptions {
  /** 空間の図が描けているときだけ，真にする． */
  enabled: boolean;
  onRotate: (deltaAzimuth: number, deltaElevation: number) => void;
}

interface DragHandlers {
  onPointerDown?: (event: ReactPointerEvent<HTMLDivElement>) => void;
  onPointerMove?: (event: ReactPointerEvent<HTMLDivElement>) => void;
  onPointerUp?: (event: ReactPointerEvent<HTMLDivElement>) => void;
  onPointerCancel?: (event: ReactPointerEvent<HTMLDivElement>) => void;
}

/**
 * プレビューの上のドラッグを，方位角と仰角の移動量にする．横のドラッグが方位角，縦のドラッグが仰角を動かす．
 * ポインタを押した要素が動きを捕える(`setPointerCapture`)ので，指やポインタが要素の外へ出ても追える．
 */
function useDragRotate({ enabled, onRotate }: DragRotateOptions): DragHandlers {
  const state = useRef<DragState>({ x: 0, y: 0, active: false });

  if (!enabled) {
    return {};
  }

  return {
    onPointerDown(event) {
      event.currentTarget.setPointerCapture(event.pointerId);
      state.current = { x: event.clientX, y: event.clientY, active: true };
    },
    onPointerMove(event) {
      if (!state.current.active) {
        return;
      }
      const deltaX = event.clientX - state.current.x;
      const deltaY = event.clientY - state.current.y;
      state.current = { x: event.clientX, y: event.clientY, active: true };
      if (deltaX !== 0 || deltaY !== 0) {
        onRotate(-deltaX * DEGREES_PER_PIXEL, deltaY * DEGREES_PER_PIXEL);
      }
    },
    onPointerUp() {
      state.current = { ...state.current, active: false };
    },
    onPointerCancel() {
      state.current = { ...state.current, active: false };
    },
  };
}

export { useDragRotate };
