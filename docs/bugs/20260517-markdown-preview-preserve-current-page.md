# Markdown Preview Preserves Current Page

- Symptom: Editing Markdown and inserting a line break caused the preview to jump back to page 1, so the preview did not feel synchronized with the editor.
- Root cause: `loadPdf` always reset `currentPage` to `1`, even when the reload was only a Markdown preview refresh.
- Fix: Added an optional `preserveCurrentPage` flag to `loadPdf` and used it for Markdown preview refreshes in `App.tsx`.
- Regression prevention: `src/stores/pdfStore.test.ts` now verifies that preview reload can preserve the current page, while normal PDF loading still resets to page 1.
