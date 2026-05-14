import type { Page } from '../types/pdf';

export const getSourcePageNumber = (page: Page): number | null => {
  if (typeof page.sourcePageNumber === 'number') {
    return page.sourcePageNumber;
  }
  if (typeof page.pageNumber === 'number') {
    return page.pageNumber;
  }
  return null;
};
