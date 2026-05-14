import { memo, useMemo, useCallback, useState, useRef, useEffect } from 'react';
import { TextBlock } from '../types/pdf';
import { usePdfStore } from '../stores/pdfStore';
import { logger } from '../utils/logger';

interface TextBlockOverlayProps {
  textBlock: TextBlock;
  scaleX: number;
  scaleY: number;
  pageHeight: number;
  pageWidth: number;
  isSelected: boolean;
  onClick: (textBlock: TextBlock) => void;
  onDoubleClick?: (textBlock: TextBlock) => void;
}

export const TextBlockOverlay = memo<TextBlockOverlayProps>(({
  textBlock,
  scaleX,
  scaleY,
  pageHeight,
  pageWidth,
  isSelected,
  onClick,
  onDoubleClick,
}) => {
  const { moveTextBlock } = usePdfStore();
  const [isDragging, setIsDragging] = useState(false);
  const dragStartRef = useRef<{ x: number; y: number } | null>(null);
  const startPosRef = useRef<{ x: number; y: number } | null>(null);
  const startTextPosRef = useRef<{ x: number; y: number } | null>(null);
  
  // why: 位置とサイズをメモ化して再計算を削減
  // alt: 毎回計算（不要な再計算が発生）
  // evidence: メモ化により不要な再計算が削減され、パフォーマンスが向上
  const screenY = useMemo(() => pageHeight - textBlock.y - textBlock.height, [pageHeight, textBlock.y, textBlock.height]);
  
  const position = useMemo(() => {
    // why: scaleX/scaleYが0または無効な値の場合をチェックしてNaNを防ぐ
    // alt: チェックなし（scaleX/scaleYが0の場合にNaNが発生する）
    // evidence: ゼロ除算によりNaNが発生し、レンダリング時にエラーになる
    if (scaleX === 0 || scaleY === 0 || !Number.isFinite(scaleX) || !Number.isFinite(scaleY)) {
      return { left: 0, top: 0, width: 0, height: 0 };
    }
    return {
      left: textBlock.x * scaleX,
      top: screenY * scaleY,
      width: textBlock.width * scaleX,
      height: textBlock.height * scaleY,
    };
  }, [textBlock.x, textBlock.y, textBlock.width, textBlock.height, scaleX, scaleY, screenY]);

  const handleClick = useCallback(() => {
    if (!isDragging) {
      onClick(textBlock);
    }
  }, [textBlock, onClick, isDragging]);

  const handleDoubleClick = useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    if (onDoubleClick) {
      onDoubleClick(textBlock);
    }
  }, [textBlock, onDoubleClick]);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (!isSelected) return;
    e.preventDefault();
    e.stopPropagation();
    
    dragStartRef.current = { x: e.clientX, y: e.clientY };
    startPosRef.current = { x: e.clientX, y: e.clientY };
    startTextPosRef.current = { x: textBlock.x, y: textBlock.y };
    setIsDragging(true);
  }, [isSelected, textBlock.x, textBlock.y]);

  // why: ドラッグ中のマウス移動とマウスアップを処理するイベントリスナーを追加
  // alt: handleMouseDown内でイベントリスナーを追加（useCallbackからクリーンアップ関数を返すべきではない）
  // evidence: useEffect内でイベントリスナーを管理することで、適切にクリーンアップできる
  useEffect(() => {
    if (!isDragging || !isSelected) return;

    const handleMouseMove = async (moveEvent: MouseEvent) => {
      if (!dragStartRef.current || !startPosRef.current || !startTextPosRef.current) return;
      
      // why: scaleX/scaleYが0の場合をチェックして除算エラーを防ぐ
      // alt: チェックなし（scaleX/scaleYが0の場合にInfinityが発生する）
      // evidence: ゼロ除算によりInfinityが発生し、PDF生成時にエラーになる
      if (scaleX === 0 || scaleY === 0 || !Number.isFinite(scaleX) || !Number.isFinite(scaleY)) {
        logger.warn('Invalid scale values', { scaleX, scaleY });
        return;
      }
      
      const deltaX = (moveEvent.clientX - startPosRef.current.x) / scaleX;
      const deltaY = -(moveEvent.clientY - startPosRef.current.y) / scaleY; // Y軸は反転
      
      let newX = startTextPosRef.current.x + deltaX;
      let newY = startTextPosRef.current.y + deltaY;
      
      // ページ境界チェック
      const maxX = pageWidth - textBlock.width;
      const maxY = pageHeight - textBlock.height;
      newX = Math.max(0, Math.min(newX, maxX));
      newY = Math.max(0, Math.min(newY, maxY));
      
      // リアルタイムで位置を更新
      try {
        await moveTextBlock(textBlock.id, newX, newY);
      } catch (error) {
        logger.error('Failed to move text block', error instanceof Error ? error : new Error(String(error)), { textBlockId: textBlock.id, newX, newY });
        // エラーが発生した場合は、ドラッグを停止してユーザーに通知
        setIsDragging(false);
        dragStartRef.current = null;
        startPosRef.current = null;
        startTextPosRef.current = null;
      }
    };
    
    const handleMouseUp = () => {
      setIsDragging(false);
      dragStartRef.current = null;
      startPosRef.current = null;
      startTextPosRef.current = null;
    };
    
    globalThis.addEventListener('mousemove', handleMouseMove);
    globalThis.addEventListener('mouseup', handleMouseUp);
    
    return () => {
      globalThis.removeEventListener('mousemove', handleMouseMove);
      globalThis.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isDragging, isSelected, textBlock, scaleX, scaleY, pageWidth, pageHeight, moveTextBlock]);

  // why: コンポーネントがアンマウントされたときにドラッグ中のイベントリスナーをクリーンアップ
  // alt: クリーンアップしない（メモリリークが発生する可能性がある）
  // evidence: クリーンアップにより、コンポーネントがアンマウントされたときにイベントリスナーが削除される
  useEffect(() => {
    return () => {
      if (isDragging) {
        setIsDragging(false);
        dragStartRef.current = null;
        startPosRef.current = null;
        startTextPosRef.current = null;
      }
    };
  }, [isDragging]);

  const title = useMemo(() => `テキストブロック: ${textBlock.text.substring(0, 50)}...`, [textBlock.text]);
  
  return (
    <div
      className={`text-block-overlay ${isSelected ? 'selected' : ''}`}
      style={{
        '--overlay-left': `${position.left}px`,
        '--overlay-top': `${position.top}px`,
        '--overlay-width': `${position.width}px`,
        '--overlay-height': `${position.height}px`,
        border: isSelected ? '2px solid rgba(0, 123, 255, 0.8)' : '1px dashed rgba(0, 123, 255, 0.5)',
        backgroundColor: isSelected ? 'rgba(0, 123, 255, 0.2)' : 'rgba(0, 123, 255, 0.1)',
        cursor: isSelected ? 'move' : 'pointer',
      } as React.CSSProperties}
      title={`${title}${isSelected ? ' (ドラッグで移動)' : ''}`}
      onClick={handleClick}
      onDoubleClick={handleDoubleClick}
      onMouseDown={handleMouseDown}
    />
  );
}, (prevProps, nextProps) => {
  // カスタム比較関数：変更がない場合は再レンダリングをスキップ
  return (
    prevProps.textBlock.id === nextProps.textBlock.id &&
    prevProps.textBlock.x === nextProps.textBlock.x &&
    prevProps.textBlock.y === nextProps.textBlock.y &&
    prevProps.textBlock.width === nextProps.textBlock.width &&
    prevProps.textBlock.height === nextProps.textBlock.height &&
    prevProps.textBlock.text === nextProps.textBlock.text &&
    prevProps.scaleX === nextProps.scaleX &&
    prevProps.scaleY === nextProps.scaleY &&
    prevProps.isSelected === nextProps.isSelected
  );
});

TextBlockOverlay.displayName = 'TextBlockOverlay';
