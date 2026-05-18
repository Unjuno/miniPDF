# Emoji preview font fallback

## Symptom
- Emoji in Markdown preview could disappear or turn into missing-glyph placeholders in the PDF preview pane.
- A mixed line such as `日本語 English 123 العربية 🚀` could render the emoji poorly even though surrounding text was correct.
- Even when the emoji was present, the rasterized fallback could look too small or faint in the generated PDF.

## Root cause
- The preview path routed emoji through a font-backed text path that oxidize-pdf / PDF.js handled unreliably.
- Emoji were not isolated as a separate rendering case, so the renderer could not fall back to a more stable bitmap representation when the font path failed.

## Fix
- Added a Windows emoji font fallback and used it as the source for emoji rasterization.
- Preferred `seguiemj.ttf` over `seguisym.ttf` on Windows because it covers a broader emoji set, including symbols like `🧪`.
- Split text into grapheme clusters and render emoji clusters as PNG images inside the PDF preview.
- Updated width estimation so line wrapping stays aligned with the emoji image width.
- Increased the internal raster density and display scale so the fallback emoji stays legible in PDF viewers.

## Regression prevention
- Keep emoji detection covered by unit tests.
- Keep the emoji list-item regression test that verifies the PDF embeds rasterized emoji images.
- Verify the visual fixture that includes emoji list items and mixed-script text when changing emoji rendering.
