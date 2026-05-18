# Markdown Display Math No Box

- Symptom: Block math was rendered inside a light framed box, which made formulas look boxed in instead of inline with the page flow.
- Root cause: `draw_display_math_block()` painted a filled rectangle and border around both KaTeX-rendered math and the text fallback path.
- Fix: Removed the background fill and stroke entirely and kept only centered math rendering with vertical padding.
- Regression prevention: Display math now uses the existing math rendering tests, and the visual fixture should be checked to ensure no frame returns around formulas.
