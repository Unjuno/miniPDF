import { useState, useCallback, useEffect } from 'react';

/**
 * 要素のドラッグ移動を実現するフック
 * Wordライクなブロック移動機能を提供
 */
export const useDragMove = (
  initialX: number,
  initialY: number,
  onMove: (x: number, y: number) => void,
  bounds?: { minX?: number; minY?: number; maxX?: number; maxY?: number }
) => {
  const [isDragging, setIsDragging] = useState(false);
  const [currentPosition, setCurrentPosition] = useState({ x: initialX, y: initialY });

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    
    const startPos = { x: e.clientX, y: e.clientY };
    const startElementPos = { x: initialX, y: initialY };
    
    setIsDragging(true);

    const handleMouseMove = (moveEvent: MouseEvent) => {
      const deltaX = moveEvent.clientX - startPos.x;
      const deltaY = moveEvent.clientY - startPos.y;

      let newX = startElementPos.x + deltaX;
      let newY = startElementPos.y + deltaY;

      // 境界チェック
      if (bounds) {
        if (bounds.minX !== undefined) newX = Math.max(newX, bounds.minX);
        if (bounds.maxX !== undefined) newX = Math.min(newX, bounds.maxX);
        if (bounds.minY !== undefined) newY = Math.max(newY, bounds.minY);
        if (bounds.maxY !== undefined) newY = Math.min(newY, bounds.maxY);
      }

      setCurrentPosition({ x: newX, y: newY });
      
      requestAnimationFrame(() => {
        onMove(newX, newY);
      });
    };

    const handleMouseUp = () => {
      setIsDragging(false);
      globalThis.removeEventListener('mousemove', handleMouseMove);
      globalThis.removeEventListener('mouseup', handleMouseUp);
    };

    globalThis.addEventListener('mousemove', handleMouseMove);
    globalThis.addEventListener('mouseup', handleMouseUp);
  }, [initialX, initialY, onMove, bounds]);

  useEffect(() => {
    setCurrentPosition({ x: initialX, y: initialY });
  }, [initialX, initialY]);

  useEffect(() => {
    if (!isDragging) return;

    const handleBlur = () => {
      setIsDragging(false);
    };

    globalThis.addEventListener('blur', handleBlur);
    return () => {
      globalThis.removeEventListener('blur', handleBlur);
    };
  }, [isDragging]);

  return {
    handleMouseDown,
    isDragging,
    currentPosition,
  };
};

