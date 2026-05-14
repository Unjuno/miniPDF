# Bug Note: preview-reorder-not-updating

## Symptom
- Drag-and-drop reorder in the page list updates the list, but the preview continues to show the old page order.

## Root Cause
- The preview reuse key for continuous mode used only the display page number, so React reused DOM nodes even when the source page changed after reordering.

## Fix
- Include the source page number in the preview key and re-trigger rendering when page order changes.

## Prevent Regression
- Keep preview keys stable per source page and ensure re-render is triggered when page order changes.
