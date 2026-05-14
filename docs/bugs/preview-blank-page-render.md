# Bug Note: preview-blank-page-render

## Symptom
- After inserting pages without source content, the main preview showed stale content and continuous preview skipped or misrepresented those pages.

## Root Cause
- PDF rendering only executed when a valid `sourcePageNumber` existed, so blank/inserted pages never cleared or resized the canvas.

## Fix
- Render an explicit blank page when no source page is available and ensure blank pages update on zoom.

## Prevent Regression
- Added unit tests for blank-page canvas rendering (`src/utils/renderBlankPage.test.ts`).
