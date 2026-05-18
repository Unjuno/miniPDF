# Markdown save write permission missing

## Symptom
- Clicking `MD保存 (Ctrl+S)` opened the save dialog, but the actual write failed.
- The UI showed `Markdownの保存に失敗しました: fs.write_file not allowed`.

## Root Cause
- The Markdown save flow uses `@tauri-apps/plugin-fs` `writeFile`, which requires the `fs:allow-write-file` capability permission.
- The main window capability only allowed file reads, so the write command was rejected by Tauri.

## Fix
- Added `fs:allow-write-file` to `src-tauri/capabilities/main-capability.json`.
- Kept the existing file scope limited to the desktop, home, and documents folders.

## Regression Prevention
- Added a test that asserts `fs:allow-write-file` remains enabled in the main capability.
- Re-run `npm test` to catch future capability regressions.
