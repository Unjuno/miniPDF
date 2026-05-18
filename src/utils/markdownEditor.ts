export interface TextEditResult {
  text: string;
  cursorOffset: number;
}

export const insertMarkdownHardBreak = (
  text: string,
  selectionStart: number,
  selectionEnd: number
): TextEditResult => {
  const start = Math.max(0, Math.min(selectionStart, text.length));
  const end = Math.max(start, Math.min(selectionEnd, text.length));
  const nextText = `${text.slice(0, start)}  \n${text.slice(end)}`;
  return {
    text: nextText,
    cursorOffset: start + 3,
  };
};

export const restoreTextareaCursor = (
  element: HTMLTextAreaElement | null,
  cursorOffset: number
): void => {
  if (!element) return;
  const nextOffset = Math.max(0, Math.min(cursorOffset, element.value.length));
  element.focus();
  element.setSelectionRange(nextOffset, nextOffset);
};

export const resolveEditorCursorOffset = (
  pendingCursorOffset: number | null,
  fallbackCursorOffset: number
): number => {
  return pendingCursorOffset ?? fallbackCursorOffset;
};
