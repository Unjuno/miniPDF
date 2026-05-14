# ブラウザモードでの `invoke` 呼び出しエラー防止

- status: fixed
- date: 2026-04-25
- type: bugfix
- target: `src/App.tsx`, `src/stores/pdfStore.ts`, `src/utils/tauriRuntime.ts`

## symptom

- `http://localhost:5173/` の通常ブラウザ実行で、Markdown入力や保存操作時に `TypeError: Cannot read properties of undefined (reading 'invoke')` が発生する。
- プレビュー更新処理とファイルダイアログ処理が失敗し、エラーログが連続して出る。

## root cause

- Tauri API (`@tauri-apps/api/core` の `invoke`) は Tauri WebView 環境でのみ有効だが、通常ブラウザ実行時のガードが不足していた。
- `App` 側のファイル操作ハンドラとプレビュー更新フロー、`pdfStore` 側のMarkdownプレビュー要求処理が、実行環境を判定せずに `invoke` を呼んでいた。

## fix

- `src/utils/tauriRuntime.ts` に `isTauriRuntimeAvailable()` を追加し、Tauri実行判定を共通化。
- `src/stores/pdfStore.ts` の `requestMarkdownPreview()` でTauri未実行時は早期 return し、`previewError` に案内メッセージを設定。
- `src/App.tsx` で以下を実施:
  - Tauri未実行時は `MDを開く / MD保存 / PDF保存` を無効化。
  - Tauri未実行時は `Ctrl+O / Ctrl+S` をショートカット登録しない。
  - 各ファイル操作ハンドラでTauri未実行時は `invoke` を呼ばず、エラー通知のみ行う。
  - ライブプレビュー `useEffect` をTauri実行時のみ動作させる。

## prevention

- `src/stores/pdfStore.test.ts` に以下の回帰テストを追加:
  - ブラウザモードではMarkdownプレビュー生成をスキップし、`invoke` が呼ばれないこと。
  - Tauriモードでは `render_markdown_to_pdf_preview` が呼ばれること。
- lint/test/build を実行し、修正後の回帰がないことを確認。
