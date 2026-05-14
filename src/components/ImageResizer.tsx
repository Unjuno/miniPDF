import React, { useCallback } from 'react';
import { ImageElement } from '../types/pdf';
import { useDragResize } from '../hooks/useDragResize';
import './ImageResizer.css';

interface ImageResizerProps {
  image: ImageElement;
  onResize: (imageId: string, width: number, height: number) => void;
  maintainAspectRatio?: boolean;
}

export const ImageResizer: React.FC<ImageResizerProps> = ({
  image,
  onResize,
  maintainAspectRatio = true,
}) => {
  const handleResize = useCallback((width: number, height: number) => {
    onResize(image.id, width, height);
  }, [image.id, onResize]);

  // 各ハンドル用のリサイズハンドラー
  const { handleMouseDown: handleMouseDownSE } = useDragResize(
    image.width,
    image.height,
    handleResize,
    maintainAspectRatio
  );

  // why: 重複コードを削減し、共通のリサイズロジックを関数化
  // alt: 各方向ごとに個別のハンドラーを実装（コードサイズが大きい）
  // evidence: 方向パラメータのみが異なり、ロジックは同一
  // assumption: 方向パラメータ（deltaX, deltaYの符号）を渡すことで共通化可能
  const createResizeHandler = useCallback((
    invertX: boolean,
    invertY: boolean
  ) => {
    return (e: React.MouseEvent) => {
      e.preventDefault();
      e.stopPropagation();
      const startPos = { x: e.clientX, y: e.clientY };
      const startSize = { width: image.width, height: image.height };
      const aspectRatio = image.width / image.height;

      const handleMouseMove = (moveEvent: MouseEvent) => {
        const rawDeltaX = moveEvent.clientX - startPos.x;
        const rawDeltaY = moveEvent.clientY - startPos.y;
        const deltaX = invertX ? -rawDeltaX : rawDeltaX;
        const deltaY = invertY ? -rawDeltaY : rawDeltaY;

        let newWidth: number;
        let newHeight: number;

        if (maintainAspectRatio) {
          const absDeltaX = Math.abs(deltaX);
          const absDeltaY = Math.abs(deltaY);
          if (absDeltaX > absDeltaY) {
            newWidth = startSize.width + deltaX;
            newHeight = newWidth / aspectRatio;
          } else {
            newHeight = startSize.height + deltaY;
            newWidth = newHeight * aspectRatio;
          }
        } else {
          newWidth = startSize.width + deltaX;
          newHeight = startSize.height + deltaY;
        }

        // why: 最小サイズを画面座標系で計算（imageは画面座標系のサイズ）
        // alt: PDF座標系で計算（画面表示との不整合が発生する）
        // evidence: ImageResizerは画面座標系で動作するため、最小サイズも画面座標系で計算する必要がある
        const minWidth = Math.max(image.width * 0.1, 10);
        const minHeight = Math.max(image.height * 0.1, 10);
        newWidth = Math.max(newWidth, minWidth);
        newHeight = Math.max(newHeight, minHeight);
        
        // why: サイズが0以下またはNaN/Infinityの場合をチェックしてエラーを防ぐ
        // alt: チェックなし（無効な値がhandleResizeに渡される）
        // evidence: 無効な値がhandleResizeに渡されると、PDF生成時にエラーになる
        if (newWidth <= 0 || newHeight <= 0 || !Number.isFinite(newWidth) || !Number.isFinite(newHeight)) {
          return;
        }

        handleResize(newWidth, newHeight);
      };

      const handleMouseUp = () => {
        globalThis.removeEventListener('mousemove', handleMouseMove);
        globalThis.removeEventListener('mouseup', handleMouseUp);
      };

      globalThis.addEventListener('mousemove', handleMouseMove);
      globalThis.addEventListener('mouseup', handleMouseUp);
    };
  }, [image.width, image.height, maintainAspectRatio, handleResize]);

  // NE (右上): Y方向を反転
  const handleMouseDownNE = useCallback(
    createResizeHandler(false, true),
    [createResizeHandler]
  );

  // SW (左下): X方向を反転
  const handleMouseDownSW = useCallback(
    createResizeHandler(true, false),
    [createResizeHandler]
  );

  // NW (左上): XとYを両方反転
  const handleMouseDownNW = useCallback(
    createResizeHandler(true, true),
    [createResizeHandler]
  );

  return (
    <div className="image-resizer">
      <div
        className="resize-handle resize-handle-se"
        onMouseDown={handleMouseDownSE}
      >
        <div className="resize-handle-inner"></div>
      </div>
      <div
        className="resize-handle resize-handle-ne"
        onMouseDown={handleMouseDownNE}
      >
        <div className="resize-handle-inner"></div>
      </div>
      <div
        className="resize-handle resize-handle-sw"
        onMouseDown={handleMouseDownSW}
      >
        <div className="resize-handle-inner"></div>
      </div>
      <div
        className="resize-handle resize-handle-nw"
        onMouseDown={handleMouseDownNW}
      >
        <div className="resize-handle-inner"></div>
      </div>
    </div>
  );
};
