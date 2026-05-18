import { describe, expect, it } from 'vitest';
import { isMarkdownFilePath } from './markdownFile';

describe('isMarkdownFilePath', () => {
  it('accepts common markdown extensions', () => {
    expect(isMarkdownFilePath('note.md')).toBe(true);
    expect(isMarkdownFilePath('note.markdown')).toBe(true);
    expect(isMarkdownFilePath('note.mdown')).toBe(true);
  });

  it('rejects non-markdown files', () => {
    expect(isMarkdownFilePath('note.txt')).toBe(false);
    expect(isMarkdownFilePath('note.pdf')).toBe(false);
    expect(isMarkdownFilePath('note')).toBe(false);
  });
});
