# Markdown save dialog uses MD extension

## Symptom
- The button label said `MD保存`, but the save dialog opened with a PDF filename and PDF filter.
- That made Markdown saves look like PDF saves.

## Root Cause
- Markdown save and PDF save shared the same dialog preset.
- The dialog defaulted to `output.pdf` and a PDF-only filter even when the user was saving Markdown source.

## Fix
- Split the save dialog preset by target.
- Markdown saves now open with `output.md` and a Markdown filter.
- PDF saves still use `output.pdf`.

## Regression Prevention
- Added unit tests for the dialog preset selection.
- `npm test` should keep the save targets separated.
