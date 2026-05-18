# Markdown math baseline and norm notation

## Symptom
- Inline math sometimes sat too low relative to surrounding text.
- Fallback rendering for norm notation such as `\|x\|_2` could degrade into slash-like text instead of a clear double-bar norm.

## Root cause
- Inline math images were anchored a little too low on the text baseline.
- KaTeX rendered norm delimiters like `\|x\|_2` as slanted slash-like glyphs in this preview path unless the delimiters were normalized first.
- The fallback simplifier normalized many TeX commands, but it did not convert norm delimiters like `\|`, `\lVert`, or `\rVert` into a readable symbol.

## Fix
- Raised the inline math image anchor slightly so math sits closer to the surrounding text baseline.
- Normalized norm delimiters to `‖` before sending expressions to KaTeX.
- Normalized norm delimiters to `‖` in the fallback math simplifier.
- Added a regression test for vector norm fallback formatting.
- Added a regression test for KaTeX normalization of norm delimiters.

## Regression prevention
- Keep math fallback normalization covered by unit tests.
- When changing inline math placement, verify both normal inline formulas and norm/absolute-value expressions in the preview.
