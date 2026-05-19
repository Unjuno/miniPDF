import { RefObject, useCallback, useLayoutEffect } from 'react';
import { estimatePreviewScrollTopFromMarkdown } from '../utils/markdownPositionSync';

interface UseSinglePagePreviewScrollSyncParams {
  enabled: boolean;
  markdownText: string;
  editorCursorLine: number;
  linePageMap?: number[] | null;
  containerRef: RefObject<HTMLDivElement | null>;
}

export const useSinglePagePreviewScrollSync = ({
  enabled,
  markdownText,
  editorCursorLine,
  linePageMap,
  containerRef,
}: UseSinglePagePreviewScrollSyncParams): void => {
  const syncScroll = useCallback(() => {
    if (!enabled) return;

    const container = containerRef.current;
    if (!container) return;
    if (typeof editorCursorLine !== 'number' || editorCursorLine < 1) return;

    const syncLineCount = markdownText.split(/\r?\n/).length;
    if (linePageMap && linePageMap.length !== syncLineCount) {
      return;
    }
    const cursorLine = Math.min(editorCursorLine, syncLineCount);

    const nextScrollTop = estimatePreviewScrollTopFromMarkdown(
      markdownText,
      cursorLine,
      container.clientHeight,
      container.scrollHeight,
      linePageMap,
    );

    if (!Number.isFinite(nextScrollTop)) return;
    if (Math.abs(container.scrollTop - nextScrollTop) < 1) return;

    container.scrollTop = nextScrollTop;
  }, [enabled, containerRef, editorCursorLine, linePageMap, markdownText]);

  useLayoutEffect(() => {
    const rafId = requestAnimationFrame(syncScroll);
    return () => cancelAnimationFrame(rafId);
  }, [syncScroll]);
};
