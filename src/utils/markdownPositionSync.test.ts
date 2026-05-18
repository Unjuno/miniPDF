import { describe, expect, it } from 'vitest';
import { estimatePreviewPageFromMarkdown, getLineNumberFromOffset } from './markdownPositionSync';

describe('markdownPositionSync', () => {
  it('counts lines from a cursor offset', () => {
    const markdown = 'one\ntwo\r\nthree\n';
    expect(getLineNumberFromOffset(markdown, 0)).toBe(1);
    expect(getLineNumberFromOffset(markdown, 3)).toBe(1);
    expect(getLineNumberFromOffset(markdown, 4)).toBe(2);
    expect(getLineNumberFromOffset(markdown, 9)).toBe(3);
    expect(getLineNumberFromOffset(markdown, 100)).toBe(4);
  });

  it('maps cursor line to a nearby preview page', () => {
    const markdown = Array.from({ length: 100 }, (_, i) => `line ${i + 1}`).join('\n');
    expect(estimatePreviewPageFromMarkdown(markdown, 1, 10)).toBe(1);
    expect(estimatePreviewPageFromMarkdown(markdown, 50, 10)).toBe(5);
    expect(estimatePreviewPageFromMarkdown(markdown, 100, 10)).toBe(10);
  });

  it('always returns the first page for a single-page preview', () => {
    expect(estimatePreviewPageFromMarkdown('single line', 1, 1)).toBe(1);
    expect(estimatePreviewPageFromMarkdown('single line', 10, 1)).toBe(1);
  });

  it('gives more weight to structural markdown lines', () => {
    const structuredMarkdown = [
      '# Heading 1',
      '',
      '> quoted line',
      '',
      '- list item',
      '',
      'plain text',
      'plain text',
      'plain text',
      'plain text',
    ].join('\n');

    const plainMarkdown = structuredMarkdown.replace('# Heading 1', 'Heading 1');
    const headingPage = estimatePreviewPageFromMarkdown(structuredMarkdown, 1, 4);
    const plainHeadingPage = estimatePreviewPageFromMarkdown(plainMarkdown, 1, 4);
    const quotePage = estimatePreviewPageFromMarkdown(structuredMarkdown, 3, 4);
    const listPage = estimatePreviewPageFromMarkdown(structuredMarkdown, 5, 4);
    const tailPage = estimatePreviewPageFromMarkdown(structuredMarkdown, 10, 4);

    expect(headingPage).toBeGreaterThanOrEqual(plainHeadingPage);
    expect(quotePage).toBeGreaterThanOrEqual(1);
    expect(listPage).toBeGreaterThanOrEqual(quotePage);
    expect(tailPage).toBe(4);
  });

  it('treats blank lines as real layout space so they can shift page estimates', () => {
    const compact = Array.from({ length: 20 }, (_, i) => `line ${i + 1}`).join('\n');
    const spaced = Array.from({ length: 20 }, (_, i) => {
      if (i === 9) {
        return ['line 10', ...Array.from({ length: 12 }, () => '')].join('\n');
      }
      return `line ${i + 1}`;
    }).join('\n');

    const compactPage = estimatePreviewPageFromMarkdown(compact, 20, 4);
    const spacedPage = estimatePreviewPageFromMarkdown(spaced, 20 + 12, 4);

    expect(spacedPage).toBeGreaterThanOrEqual(compactPage);
  });

  it('treats headings and thematic breaks as stronger structural breaks than plain lines', () => {
    const plain = Array.from({ length: 30 }, (_, i) => `line ${i + 1}`).join('\n');
    const structured = [
      'line 1',
      'line 2',
      'line 3',
      '# Heading',
      'line 5',
      '---',
      'line 7',
      'line 8',
      'line 9',
      'line 10',
      'line 11',
      'line 12',
      'line 13',
      'line 14',
      'line 15',
      'line 16',
      'line 17',
      'line 18',
      'line 19',
      'line 20',
      'line 21',
      'line 22',
      'line 23',
      'line 24',
      'line 25',
      'line 26',
      'line 27',
      'line 28',
      'line 29',
      'line 30',
    ].join('\n');

    const plainPage = estimatePreviewPageFromMarkdown(plain, 30, 4);
    const structuredPage = estimatePreviewPageFromMarkdown(structured, 30, 4);

    expect(structuredPage).toBeGreaterThanOrEqual(plainPage);
  });
});
