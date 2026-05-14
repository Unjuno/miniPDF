import React, { useState, useCallback, useEffect, useRef } from 'react';
import { Page } from '../types/pdf';
import { logger } from '../utils/logger';
import './PageBreakEditor.css';

interface PageBreakEditorProps {
  page: Page;
  onAdjust: (pageNumber: number, newPosition: number) => void;
}

export const PageBreakEditor: React.FC<PageBreakEditorProps> = ({
  page,
  onAdjust,
}) => {
  const [isDragging, setIsDragging] = useState(false);
  const [currentPosition, setCurrentPosition] = useState(page.height);
  const containerRef = useRef<HTMLDivElement>(null);
  const startYRef = useRef<number>(0);
  const startPositionRef = useRef<number>(0);
  const rafIdRef = useRef<number | null>(null);

  useEffect(() => {
    setCurrentPosition(page.height);
  }, [page.height]);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (!containerRef.current) return;
    
    setIsDragging(true);
    startYRef.current = e.clientY;
    startPositionRef.current = currentPosition;
  }, [currentPosition]);

  useEffect(() => {
    if (!isDragging || !containerRef.current) return;

    // why: マウス移動処理をrequestAnimationFrameでスロットルしてパフォーマンス向上
    // alt: 即座に実行（頻繁な更新が発生）
    // evidence: requestAnimationFrameによりスムーズなアニメーションとパフォーマンス向上
    const handleMouseMove = (e: MouseEvent) => {
      if (rafIdRef.current !== null) {
        cancelAnimationFrame(rafIdRef.current);
      }
      rafIdRef.current = requestAnimationFrame(() => {
        const rect = containerRef.current?.getBoundingClientRect();
        if (!rect) {
          rafIdRef.current = null;
          return;
        }
        
        const deltaY = e.clientY - startYRef.current;
        // スケールを考慮（PDFViewerのオーバーレイスケールを取得）
        // why: page.heightが0の場合をチェックして除算エラーを防ぐ
        // alt: チェックなし（page.heightが0の場合にInfinityが発生する）
        // evidence: ゼロ除算によりInfinityが発生し、改ページ位置が正しく計算されない
        if (page.height === 0 || !Number.isFinite(page.height)) {
          logger.warn('Invalid page height', { pageHeight: page.height });
          rafIdRef.current = null;
          return;
        }
        const scaleY = rect.height / page.height;
        if (scaleY === 0 || !Number.isFinite(scaleY)) {
          logger.warn('Invalid scaleY', { scaleY });
          rafIdRef.current = null;
          return;
        }
        const deltaPosition = deltaY / scaleY;
        const newPosition = Math.max(0, Math.min(page.height, startPositionRef.current + deltaPosition));
        
        setCurrentPosition(newPosition);
        onAdjust(page.pageNumber, newPosition);
        rafIdRef.current = null;
      });
    };

    const handleMouseUp = () => {
      if (rafIdRef.current !== null) {
        cancelAnimationFrame(rafIdRef.current);
        rafIdRef.current = null;
      }
      setIsDragging(false);
    };

    globalThis.addEventListener('mousemove', handleMouseMove);
    globalThis.addEventListener('mouseup', handleMouseUp);

    return () => {
      if (rafIdRef.current !== null) {
        cancelAnimationFrame(rafIdRef.current);
        rafIdRef.current = null;
      }
      globalThis.removeEventListener('mousemove', handleMouseMove);
      globalThis.removeEventListener('mouseup', handleMouseUp);
    };
  }, [isDragging, page.pageNumber, page.height, onAdjust]);

  const ariaValuemax = Math.round(page.height);
  const ariaValuenow = Math.round(currentPosition);

  return (
    <div ref={containerRef} className="page-break-editor">
      <div
        className={`page-break-line ${isDragging ? 'dragging' : ''}`}
        style={{ '--page-break-position': `${currentPosition}px` } as React.CSSProperties}
        onMouseDown={handleMouseDown}
        role="slider"
        aria-label="改ページ位置"
        aria-valuemin={0}
        aria-valuemax={ariaValuemax}
        aria-valuenow={ariaValuenow}
        tabIndex={0}
      >
        <div className="page-break-handle"></div>
      </div>
    </div>
  );
};
