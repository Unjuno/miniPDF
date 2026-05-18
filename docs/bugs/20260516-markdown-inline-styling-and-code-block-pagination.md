# Markdown preview styling and code block pagination

## Symptom
- Japanese emphasis text was shown as normal text instead of italic-looking text.
- Links were rendered as plain text with no visual indication.
- Blockquotes were wider than the actual quoted content.
- Code blocks looked like plain wrapped text and long blocks did not preserve page splitting after the block-frame update.

## Root cause
- Japanese italic text fell back to the regular body font because the PDF font set did not provide a Japanese oblique face.
- Link nodes were parsed and then flattened into normal spans without any link-specific styling metadata.
- Blockquote width was hard-coded close to the full printable width instead of being derived from the quote content.
- Code block rendering used per-line text output and did not preserve blank lines or page-wise framing when the layout changed.

## Fix
- Added a link style flag to inline spans and rendered links in blue with an underline.
- Applied a faux italic slant for non-ASCII italic runs so Japanese emphasis remains visibly italic.
- Calculated blockquote width from the measured content width.
- Rendered code blocks inside a framed container, preserved blank lines, and paginated long blocks into per-page chunks.

## Regression prevention
- Added tests for link styling, Japanese faux italic selection, blockquote width, blank-line preservation in code wrapping, and long code block pagination.
- Kept the PDF output path under `cargo test` so the renderer regressions fail in CI.
