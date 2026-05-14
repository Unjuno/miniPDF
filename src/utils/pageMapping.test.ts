import { describe, expect, it } from 'vitest';
import { getSourcePageNumber } from './pageMapping';

describe('getSourcePageNumber', () => {
  it('prefers sourcePageNumber when present', () => {
    const page = {
      pageNumber: 2,
      sourcePageNumber: 5,
      width: 10,
      height: 10,
      images: [],
      textBlocks: [],
    };

    expect(getSourcePageNumber(page)).toBe(5);
  });

  it('falls back to pageNumber when sourcePageNumber is missing', () => {
    const page = {
      pageNumber: 3,
      width: 10,
      height: 10,
      images: [],
      textBlocks: [],
    };

    expect(getSourcePageNumber(page)).toBe(3);
  });
});
