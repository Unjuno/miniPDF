# Markdown inline math, lists, and blockquote layout

## Symptom
- Japanese italic text was drawn at the wrong position when faux slanting was applied.
- Blockquotes were too wide compared with the quoted text.
- Ordered lists could be hard to distinguish from plain paragraph text.
- Inline math and display math were emitted as raw text with no special handling.

## Root cause
- The PDF text context in `oxidize-pdf` does not inherit the graphics transform used for faux italic text.
- Blockquote width used a fixed near-page-width rectangle.
- Ordered list rendering relied on text prefixes only, with no regression test on the PDF extraction path.
- Markdown math was not parsed into a dedicated representation at all.

## Fix
- Replaced the transform-based faux italic path with per-glyph skewed placement so text stays on the intended baseline.
- Narrowed blockquote width to a measured content-based frame.
- Added PDF extraction coverage for ordered list prefixes.
- Added basic inline `$...$` detection and a dedicated display-math block for `$$...$$`.

## Regression prevention
- Added tests covering faux italic selection, ordered list prefix extraction, inline math parsing, display math rendering, and blockquote width estimation.
- Kept the checks in `cargo test`, `npm test`, and `npm run build`.

## Constraint
- The math handling is intentionally conservative and visual-first. It is not a full TeX engine, but it prevents raw math delimiters from flowing through unchanged and keeps the document layout stable.
