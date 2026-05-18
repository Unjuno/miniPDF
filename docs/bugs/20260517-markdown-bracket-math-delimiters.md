# Markdown bracket math delimiters

## Symptom
- Visual check PDFs showed `\(...\)` and `\[...\]` math delimiters as raw text instead of rendering them as math.

## Root Cause
- The Markdown preview parser only split inline math on `$...$` and display math on `$$...$$`.
- Raw bracket math delimiters were left in ordinary text nodes, so they flowed through to the rendered PDF unchanged.

## Fix
- Extended inline math parsing to recognize `\(...\)` and convert it into math spans.
- Added paragraph-level detection for standalone `\[...\]` blocks and mapped them to display math blocks.
- Added regression tests for both bracket math forms.

## Regression Prevention
- `npm test` now covers both bracket math delimiters and the existing dollar-delimited math paths.
- The visual check fixture should be reviewed to confirm bracket math renders as math rather than plain text.
