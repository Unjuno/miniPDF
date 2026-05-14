# Bug Note: Zoom-out button rendered with fullwidth minus

- Symptom: Zoom-out control label showed a fullwidth minus (U+2212), which appears off-center or misaligned in some environments.
- Root cause: The button text used the Unicode minus instead of ASCII `-`.
- Fix: Replace the glyph with ASCII `-` in `src/App.tsx` so the label renders consistently.
- Prevention: Keep control labels ASCII unless there is a localization reason; if non-ASCII is needed, add a test or lint rule to catch unexpected glyphs.
