# Markdown Preview Sync Page Boundary

- Symptom: The preview pane often stayed on the previous page while editing Markdown, so cursor movement did not feel reflected in the preview.
- Root cause: The page estimator used a floor-based boundary (`floor(normalized * pageCount) + 1`), which delayed page transitions until the cursor moved well past the midpoint of a visual section.
- Fix: Switched the estimator to a ceiling-based boundary so preview pages advance earlier and stay closer to the editor position.
- Regression prevention: `src/utils/markdownPositionSync.test.ts` now covers midpoint mapping and structural-line weighting, and `npm test` verifies the estimator stays stable.
