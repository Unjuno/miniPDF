# Clamp preview blank-line spacers

## Symptom
- A Markdown document with many consecutive blank lines could create oversized gaps in the preview layout.
- The visual-check fixture contained long blank-line runs after thematic breaks, which made the preview drift far more than intended.

## Root cause
- Blank lines were converted directly into spacer blocks using their raw line count.
- That preserved spacing intent, but it also let long blank-line runs inflate the rendered layout excessively.

## Fix
- Clamp blank-line spacers to at most two lines per run.
- Keep single blank lines effective so they can still push content downward when needed.

## Regression prevention
- Keep a unit test that verifies excessive blank lines clamp to the same spacer size.
- Keep the visual-check fixture in sync with the renderer behavior so huge blank runs do not regress into giant gaps again.
