# Bug Note: Long code blocks clipped at page boundaries

## Symptom
Markdown preview PDFs could clip long code blocks at page boundaries instead of continuing them cleanly onto the next page.

## Root Cause
The code-block renderer treated a code block as one fixed-height region and only checked space before drawing the whole block. When the block exceeded the remaining page height, later lines could be rendered outside the visible area.

## Fix
Code blocks are now rendered line by line with page-space checks before each wrapped line. This lets long blocks flow onto additional pages instead of clipping.

## Regression Prevention
- Added a test that generates a long code block and asserts it spans multiple pages.
- Kept the existing blank-leading-page regression test so oversized blocks still do not create an empty first page.
