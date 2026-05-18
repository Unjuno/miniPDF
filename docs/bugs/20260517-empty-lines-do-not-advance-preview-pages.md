# Empty lines did not advance preview pages

## Symptom
- Adding blank lines in Markdown did not noticeably shift the preview layout.
- In practice, only inserting actual text tended to move content to the next page.

## Root Cause
- The Markdown preview renderer discarded blank top-level gaps when building the PDF.
- As a result, empty lines did not contribute to page height.

## Fix
- Preserve blank-line gaps as spacer blocks during Markdown parsing.
- Render those spacers as bounded vertical layout space in both PDF and HTML preview paths.
- Increase the preview page estimator's blank-line weight so cursor/page sync stays closer to rendered output.

## Regression Guard
- Added tests that verify blank lines emit spacer blocks and can increase the rendered page count.
