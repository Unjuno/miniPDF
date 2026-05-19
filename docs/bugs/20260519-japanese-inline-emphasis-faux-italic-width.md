# Japanese inline emphasis faux italic width

## Symptom
Japanese inline emphasis such as `**太字**`, `*斜体*`, and `***太字斜体***` could appear shifted or unnaturally wide in the PDF preview. The following text could start from a visually wrong position.

## Root cause
The faux italic fallback drew each Japanese grapheme at an increasing x offset. That made the span look like individual characters were drifting instead of the whole span being slanted, and the advance width no longer matched a natural inline text run.
The first raster fallback also treated the image baseline as a top-origin distance when PDF placement needs a bottom-origin baseline offset, so italic spans appeared below the surrounding text.

## Fix
Non-emoji Japanese italic and bold-italic spans now rasterize the whole text span with the body or body-bold Japanese font, shear the resulting image once, and advance by the actual drawn image width. Emoji-mixed spans keep the existing grapheme fallback path.
The raster baseline offset is now computed from the bottom of the trimmed image, matching PDF image placement.

## Regression prevention
- Keep unit coverage for the sample sentence containing bold, italic, and bold-italic Japanese spans.
- Keep unit coverage that non-emoji Japanese italic uses the full-span raster faux italic path.
- Keep unit coverage that raster faux italic baseline offsets are measured from the image bottom.
- Render the sample Markdown to PDF/PNG before merging future bold or italic preview changes.
