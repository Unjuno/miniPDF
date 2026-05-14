import { useState, useCallback, useEffect } from 'react';

export const useDragResize = (
  initialWidth: number,
  initialHeight: number,
  onResize: (width: number, height: number) => void,
  maintainAspectRatio: boolean = true
) => {
  const [isDragging, setIsDragging] = useState(false);
  const aspectRatio = initialWidth / initialHeight;

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    
    const startPos = { x: e.clientX, y: e.clientY };
    const startSize = { width: initialWidth, height: initialHeight };
    
    setIsDragging(true);

    const handleMouseMove = (moveEvent: MouseEvent) => {
      const deltaX = moveEvent.clientX - startPos.x;
      const deltaY = moveEvent.clientY - startPos.y;

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

      const minWidth = Math.max(initialWidth * 0.1, 10);
      const minHeight = Math.max(initialHeight * 0.1, 10);
      newWidth = Math.max(newWidth, minWidth);
      newHeight = Math.max(newHeight, minHeight);

      requestAnimationFrame(() => {
        onResize(newWidth, newHeight);
      });
    };

    const handleMouseUp = () => {
      setIsDragging(false);
      globalThis.removeEventListener('mousemove', handleMouseMove);
      globalThis.removeEventListener('mouseup', handleMouseUp);
    };

    globalThis.addEventListener('mousemove', handleMouseMove);
    globalThis.addEventListener('mouseup', handleMouseUp);
  }, [initialWidth, initialHeight, aspectRatio, maintainAspectRatio, onResize]);

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
  };
};
