# Japanese bold italic rendering

## Symptom
`***太字斜体***` rendered as plain-looking text or fallback squares instead of a clear bold italic span in the PDF preview.

## Root cause
The preview renderer was first selecting a custom `NotoSansJP-BoldItalic` face for Japanese bold-italic text. In this environment that face did not render reliably, so the glyphs were replaced with missing-glyph boxes.
After switching back to faux italic, the renderer still advanced the cursor by the unshifted glyph width, so the following span could overlap the slanted bold-italic text.

## Fix
Japanese bold-italic text now falls back to the body bold face plus faux italic slanting. The advance width now includes the slant offset, so the next span starts after the actual drawn right edge.

## Regression prevention
- Keep the `markdown_preview` regression test for Japanese italic selection.
- Keep the `faux_italic_advance_width_accounts_for_slant` regression test.
- Render a real sample containing `**太字**`, `*斜体*`, and `***太字斜体***` before merging bold/italic changes.
