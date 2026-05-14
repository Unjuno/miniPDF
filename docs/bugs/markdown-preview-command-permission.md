# Bug Note: markdown-preview-command-permission

## Symptom
- Markdown を入力すると、Tauri アプリ側で `render_markdown_to_pdf_preview not allowed. Command not found` が表示され、プレビュー生成に失敗していた。

## Root Cause
- `render_markdown_to_pdf_preview` は Rust の `invoke_handler` に登録されていたが、`src-tauri/permissions/custom-commands.toml` の許可リストに含まれていなかった。

## Fix
- `custom-commands` 権限セットに `render_markdown_to_pdf_preview` を追加し、フロントエンドから Tauri コマンドを呼び出せるようにした。

## Prevent Regression
- `src/tauriPermissions.test.ts` を追加し、Markdown プレビューコマンドが ACL に含まれていることを検証するようにした。
