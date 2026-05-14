# Bug Note: page-delete-render-mismatch

## Symptom
- Deleting page N removes the last page's content instead of the selected page, and page content appears to shift incorrectly.

## Root Cause
- The viewer always rendered pages from the original PDF file by sequential page number. After deletion, page numbers were renumbered, so the UI still requested page 1..N from the original PDF, leaving the deleted page visible and dropping the last page.

## Fix
- Track the original PDF page number (`sourcePageNumber`) and use it for PDF.js rendering and thumbnails while keeping display page numbers sequential.

## Prevent Regression
- Keep rendering tied to `sourcePageNumber` and cover the mapping logic with unit tests (`src/utils/pageMapping.test.ts`).
