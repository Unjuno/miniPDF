import { memo, lazy, Suspense, useCallback, useMemo, useState, useRef, useEffect } from 'react';
import { ImageElement } from '../types/pdf';
import { usePdfStore } from '../stores/pdfStore';
import { useDragMove } from '../hooks/useDragMove';
import { logger } from '../utils/logger';

// why: ImageResizerは重いコンポーネントのため、選択時のみ遅延読み込み
const ImageResizer = lazy(() => import('./ImageResizer').then(m => ({ default: m.ImageResizer })));

interface ImageOverlayProps {
  image: ImageElement;
  scaleX: number;
  scaleY: number;
  pageHeight: number;
  pageWidth: number;
  isSelected: boolean;
}

export const ImageOverlay = memo<ImageOverlayProps>(({
  image,
  scaleX,
  scaleY,
  pageHeight,
  pageWidth,
  isSelected,
}) => {
  const { resizeImage, selectImage, moveImage } = usePdfStore();
  const screenY = pageHeight - image.y - image.height;
  const [isDragging, setIsDragging] = useState(false);
  const dragStartRef = useRef<{ x: number; y: number } | null>(null);
  const startPosRef = useRef<{ x: number; y: number } | null>(null);
  const startImagePosRef = useRef<{ x: number; y: number } | null>(null);
  
  // PDF座標系から画面座標系への変換を考慮したリサイズハンドラー
  // why: ImageResizerには画面座標系のサイズを渡すため、リサイズ結果も画面座標系で返される
  // alt: PDF座標系で処理（画面表示との不整合が発生する）
  // evidence: ImageResizerは画面座標系で動作するため、結果も画面座標系で返される
  const handleResize = useCallback(async (imageId: string, width: number, height: number) => {
    // why: scaleX/scaleYが0の場合をチェックして除算エラーを防ぐ
    // alt: チェックなし（scaleX/scaleYが0の場合にInfinityが発生する）
    // evidence: ゼロ除算によりInfinityが発生し、PDF生成時にエラーになる
    if (scaleX === 0 || scaleY === 0 || !Number.isFinite(scaleX) || !Number.isFinite(scaleY)) {
      logger.warn('Invalid scale values', { scaleX, scaleY });
      return;
    }
    // 画面座標系からPDF座標系に変換
    const pdfWidth = width / scaleX;
    const pdfHeight = height / scaleY;
    try {
      await resizeImage(imageId, pdfWidth, pdfHeight);
    } catch (error) {
      logger.error('Failed to resize image', error instanceof Error ? error : new Error(String(error)), { imageId, pdfWidth, pdfHeight });
      throw error;
    }
  }, [scaleX, scaleY, resizeImage]);
  
  // ドラッグ移動ハンドラー（useDragMoveフック用）
  // why: useDragMoveフックから呼ばれる可能性があるため、画面座標系からPDF座標系への変換を実装
  // alt: 変換なし（座標系の違いにより位置がずれる）
  // evidence: PDF座標系は下から上、画面座標系は上から下なので、変換が必要
  const handleMove = useCallback(async (screenX: number, screenY: number) => {
    // why: scaleX/scaleYが0の場合をチェックして除算エラーを防ぐ
    // alt: チェックなし（scaleX/scaleYが0の場合にInfinityが発生する）
    // evidence: ゼロ除算によりInfinityが発生し、PDF生成時にエラーになる
    if (scaleX === 0 || scaleY === 0 || !Number.isFinite(scaleX) || !Number.isFinite(scaleY)) {
      logger.warn('Invalid scale values', { scaleX, scaleY });
      return;
    }
    // 画面座標系からPDF座標系に変換
    // why: screenYは画面座標系のY座標（上から下）なので、PDF座標系（下から上）に変換する必要がある
    // alt: 直接変換（座標系の違いを考慮しない）
    // evidence: PDF座標系は下から上なので、画面座標系から変換する際は反転が必要
    // screenYは画面座標系のY座標（上から下）で、pageHeight - image.y - image.heightで計算されている
    // PDF座標系のY座標（下から上）に変換するには、pageHeight - (screenY / scaleY) - image.heightが必要
    const pdfX = screenX / scaleX;
    const pdfY = pageHeight - (screenY / scaleY) - image.height;
    
    // ページ境界チェック
    const maxX = pageWidth - image.width;
    const maxY = pageHeight - image.height;
    const clampedX = Math.max(0, Math.min(pdfX, maxX));
    const clampedY = Math.max(0, Math.min(pdfY, maxY));
    
    try {
      await moveImage(image.id, clampedX, clampedY);
    } catch (error) {
      logger.error('Failed to move image', error instanceof Error ? error : new Error(String(error)), { imageId: image.id, clampedX, clampedY });
      throw error;
    }
  }, [scaleX, scaleY, pageHeight, pageWidth, image.id, image.width, image.height, moveImage]);
  
  // why: 位置とサイズをメモ化して再計算を削減
  // alt: 毎回計算（不要な再計算が発生）
  // evidence: メモ化により不要な再計算が削減され、パフォーマンスが向上
  const position = useMemo(() => ({
    left: image.x * scaleX,
    top: screenY * scaleY,
    width: image.width * scaleX,
    height: image.height * scaleY,
  }), [image.x, image.y, image.width, image.height, scaleX, scaleY, screenY]);

  const handleClick = useCallback(() => {
    if (!isDragging) {
      selectImage(image.id);
    }
  }, [image.id, selectImage, isDragging]);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (!isSelected) return;
    e.preventDefault();
    e.stopPropagation();
    
    dragStartRef.current = { x: e.clientX, y: e.clientY };
    startPosRef.current = { x: e.clientX, y: e.clientY };
    startImagePosRef.current = { x: image.x, y: image.y };
    setIsDragging(true);
  }, [isSelected, image.x, image.y]);

  // why: ドラッグ中のマウス移動とマウスアップを処理するイベントリスナーを追加
  // alt: handleMouseDown内でイベントリスナーを追加（useCallbackからクリーンアップ関数を返すべきではない）
  // evidence: useEffect内でイベントリスナーを管理することで、適切にクリーンアップできる
  useEffect(() => {
    if (!isDragging || !isSelected) return;

    const handleMouseMove = async (moveEvent: MouseEvent) => {
      if (!dragStartRef.current || !startPosRef.current || !startImagePosRef.current) return;
      
      // why: scaleX/scaleYが0の場合をチェックして除算エラーを防ぐ
      // alt: チェックなし（scaleX/scaleYが0の場合にInfinityが発生する）
      // evidence: ゼロ除算によりInfinityが発生し、PDF生成時にエラーになる
      if (scaleX === 0 || scaleY === 0 || !Number.isFinite(scaleX) || !Number.isFinite(scaleY)) {
        logger.warn('Invalid scale values', { scaleX, scaleY });
        return;
      }
      
      const deltaX = (moveEvent.clientX - startPosRef.current.x) / scaleX;
      const deltaY = -(moveEvent.clientY - startPosRef.current.y) / scaleY; // Y軸は反転
      
      let newX = startImagePosRef.current.x + deltaX;
      let newY = startImagePosRef.current.y + deltaY;
      
      // ページ境界チェック
      const maxX = pageWidth - image.width;
      const maxY = pageHeight - image.height;
      newX = Math.max(0, Math.min(newX, maxX));
      newY = Math.max(0, Math.min(newY, maxY));
      
      // リアルタイムで位置を更新（デバウンスなしで即座に反映）
      try {
        await moveImage(image.id, newX, newY);
      } catch (error) {
        logger.error('Failed to move image', error instanceof Error ? error : new Error(String(error)), { imageId: image.id, newX, newY });
        // エラーが発生した場合は、ドラッグを停止してユーザーに通知
        setIsDragging(false);
        dragStartRef.current = null;
        startPosRef.current = null;
        startImagePosRef.current = null;
      }
    };
    
    const handleMouseUp = () => {
      setIsDragging(false);
      dragStartRef.current = null;
      startPosRef.current = null;
      startImagePosRef.current = null;
    };
    
    globalThis.addEventListener('mousemove', handleMouseMove);
    globalThis.addEventListener('mouseup', handleMouseUp);
    
    return () => {
      globalThis.removeEventListener('mousemove', handleMouseMove);
      globalThis.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isDragging, isSelected, image, scaleX, scaleY, pageWidth, pageHeight, moveImage]);

  // why: コンポーネントがアンマウントされたときにドラッグ中のイベントリスナーをクリーンアップ
  // alt: クリーンアップしない（メモリリークが発生する可能性がある）
  // evidence: クリーンアップにより、コンポーネントがアンマウントされたときにイベントリスナーが削除される
  useEffect(() => {
    return () => {
      if (isDragging) {
        setIsDragging(false);
        dragStartRef.current = null;
        startPosRef.current = null;
        startImagePosRef.current = null;
      }
    };
  }, [isDragging]);

  return (
    <div
      className={`image-overlay ${isSelected ? 'selected' : ''}`}
      style={{
        '--overlay-left': `${position.left}px`,
        '--overlay-top': `${position.top}px`,
        '--overlay-width': `${position.width}px`,
        '--overlay-height': `${position.height}px`,
        border: isSelected ? '2px solid rgba(255, 0, 0, 0.8)' : '1px dashed rgba(255, 0, 0, 0.5)',
        backgroundColor: isSelected ? 'rgba(255, 0, 0, 0.2)' : 'rgba(255, 0, 0, 0.1)',
        cursor: isSelected ? 'move' : 'pointer',
      } as React.CSSProperties}
      title={`画像: ${image.id}${isSelected ? ' (ドラッグで移動)' : ''}`}
      onClick={handleClick}
      onMouseDown={handleMouseDown}
    >
      {isSelected && (
        <Suspense fallback={null}>
          <ImageResizer
            image={{
              ...image,
              width: image.width * scaleX,
              height: image.height * scaleY,
            }}
            onResize={handleResize}
            maintainAspectRatio={true}
          />
        </Suspense>
      )}
    </div>
  );
}, (prevProps, nextProps) => {
  // カスタム比較関数：変更がない場合は再レンダリングをスキップ
  return (
    prevProps.image.id === nextProps.image.id &&
    prevProps.image.x === nextProps.image.x &&
    prevProps.image.y === nextProps.image.y &&
    prevProps.image.width === nextProps.image.width &&
    prevProps.image.height === nextProps.image.height &&
    prevProps.scaleX === nextProps.scaleX &&
    prevProps.scaleY === nextProps.scaleY &&
    prevProps.isSelected === nextProps.isSelected
  );
});

ImageOverlay.displayName = 'ImageOverlay';
