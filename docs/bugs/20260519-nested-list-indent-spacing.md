# Nested list indent spacing

## Symptom
Nested bullet and ordered list items in the PDF preview could appear shifted too far to the right. The marker and item text spacing looked inconsistent across levels.

## Root cause
The PDF list renderer encoded nesting as leading spaces inside the marker string, then drew the marker and text as one text run. That made indentation depend on font space width and marker measurement instead of explicit list geometry.

## Fix
List markers and item text are now drawn as separate runs. Each nesting level uses a fixed point-based indent step, and wrapped lines start at the same content x position as the first line text.

## Regression prevention
- Keep unit coverage for fixed nested-list indent steps.
- Render a nested bullet sample to PDF/PNG before changing list marker layout.
