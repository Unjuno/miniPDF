# Preview page sync overrides manual navigation

## Symptom
- In the Markdown preview screen, clicking to another page could immediately snap back to the cursor-estimated page.

## Root Cause
- `src/App.tsx` used `currentPage` as a dependency of the cursor-sync effect.
- Any manual page change re-rendered `App`, which re-ran the effect and forced the preview back to the page estimated from the editor cursor.

## Fix
- Keep the latest preview page in a ref.
- Re-run the cursor-sync effect only when the Markdown content or cursor line changes.

## Regression Guard
- Added `src/App.preview-sync.test.tsx` to verify that a manual page change remains in place until the editor cursor changes again.
