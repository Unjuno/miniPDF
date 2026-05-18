import { describe, expect, it } from 'vitest';
import {
  insertMarkdownHardBreak,
  restoreTextareaCursor,
  resolveEditorCursorOffset,
} from './markdownEditor';

describe('markdownEditor', () => {
  it('inserts a markdown hard break at the cursor', () => {
    const result = insertMarkdownHardBreak('line one', 8, 8);
    expect(result.text).toBe('line one  \n');
    expect(result.cursorOffset).toBe(11);
  });

  it('replaces the selected range with a hard break', () => {
    const result = insertMarkdownHardBreak('line one', 5, 8);
    expect(result.text).toBe('line   \n');
    expect(result.cursorOffset).toBe(8);
  });

  it('restores the textarea cursor to the requested offset', () => {
    const textarea = document.createElement('textarea');
    textarea.value = 'line one  \nline two';

    restoreTextareaCursor(textarea, 11);

    expect(textarea.selectionStart).toBe(11);
    expect(textarea.selectionEnd).toBe(11);
  });

  it('clamps the textarea cursor to the current text length', () => {
    const textarea = document.createElement('textarea');
    textarea.value = 'short';

    restoreTextareaCursor(textarea, 999);

    expect(textarea.selectionStart).toBe(textarea.value.length);
    expect(textarea.selectionEnd).toBe(textarea.value.length);
  });

  it('prefers a pending cursor offset when syncing editor state', () => {
    expect(resolveEditorCursorOffset(12, 3)).toBe(12);
    expect(resolveEditorCursorOffset(null, 3)).toBe(3);
  });
});
