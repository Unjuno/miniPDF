# Markdown line break preview sync

## Symptom
Inserting line breaks in the Markdown editor (especially Enter hard breaks) caused the preview page and PDF layout to drift away from the cursor position. Page wrapping did not track additional visual lines reliably.

## Root cause
1. Preview page sync used a frontend heuristic based on source line counts and weights, while PDF layout uses `layout_spans_lines` with hard/soft breaks and automatic wrapping.
2. Rich text blocks were paginated as a single unit (`ensure_room` once per block), so tall paragraphs with many hard breaks could overflow a page without splitting.

## Fix
1. Rust preview build records a `linePageMap` (source line → first rendered page) during layout, using comrak `sourcepos` and per-layout-line `ensure_room`.
2. `render_markdown_to_pdf_preview` returns `{ filePath, linePageMap }`.
3. Frontend uses `resolvePreviewPageFromMarkdown` with the map when it matches the debounced markdown line count; otherwise it falls back to the heuristic (browser mode).

## Follow-up (mixed content / large vertical gaps)

### Symptom
Plain-text hard breaks looked correct, but documents with lists, horizontal rules, and other blocks sometimes showed a large gap between wrapped intro lines, or the preview page did not follow the cursor.

### Root cause
1. **Blank line vs hard break:** A single empty line between two source lines starts a new Markdown paragraph. The renderer inserts a `Spacer` block, which looks like a “missing” line break compared to Enter (`  \n`) inside one paragraph.
2. **Stale `linePageMap`:** Page sync required `linePageMap.length >= sourceLineCount`. While editing (before debounced preview rebuild), the map length could lag behind the live editor, forcing the weight heuristic near lists/headings and mis-assigning pages.
3. **Spacer source recording:** `record_source` mapped every blank source line in a gap to the current page even when only up to two spacer lines were drawn.

### Additional fix
1. Use partial `linePageMap` entries (clamp index) when the map is shorter than the markdown.
2. Skip page/scroll sync until `previewLinePageMap.length` matches debounced markdown line count.
3. Record spacer pages only for visibly rendered spacer lines.
4. Tests: `intro_cjk_soft_break_before_list_*`, `intro_cjk_blank_line_between_halves_splits_into_separate_paragraphs`.

### Blank lines for page-boundary tuning (2026-05-19)
Users intentionally insert blank source lines to push the following block toward the next page. Preview previously clamped every spacer to **2 lines**, so extra blank lines had almost no layout effect.

**Change:** render up to `MAX_SPACER_LINES` (48) blank lines between blocks, one `BASE_LINE_HEIGHT` per line with per-line `ensure_room` (page breaks apply). Extreme fixture runs stay capped; moderate blank runs shift `linePageMap` and page count.

### Spacers after `---`, lists, and whitespace-only lines (2026-05-19)
Comrak often attaches trailing blank source lines to `ThematicBreak` nodes, so gaps before the next heading/list item were invisible. `finalize_block_spacers` now scans raw source lines between blocks (trimming leading/trailing blanks on non-code/math blocks) so spacers appear after `---`, between list items, and for lines that contain only spaces.

## Regression prevention
- Rust: `hard_break_lines_map_to_later_preview_pages`
- Frontend: `resolvePreviewPageFromMarkdown` map/fallback tests
- `App.preview-sync.test.tsx` Enter hard-break page sync
