# Markdown preview auto-scroll preserves current page

## Symptom
- Editing Markdown after the preview had already moved to a later page caused the preview to jump back to page 1.
- The editor cursor and preview pane no longer felt synchronized after a rebuild.

## Root Cause
- `PDFViewer` reset the shared `currentPage` to `1` after every successful preview PDF load.
- That forced the preview pane back to the first page even when the editor had already selected a later page.

## Fix
- Removed the unconditional reset to page 1 after preview PDF rendering completes.
- Kept page clamping for out-of-range pages, and synchronized both local viewer state and the shared store when clamping is required.

## Regression Prevention
- Added a test that loads a refreshed preview while the shared page state is already on page 3 and verifies the page stays on 3.
- `npm test` now covers the preview reload path that previously broke auto-scroll.
