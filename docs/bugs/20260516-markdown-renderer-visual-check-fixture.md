# Bug Note: Markdown レンダラー可視チェック用 fixture を追加

- **日付**: 2026-05-16
- **種別**: Bug
- **ステータス**: 修正済み
- **対象**: `fixtures/markdown-renderer-visual-check.md`

## 症状

- Markdown の各種構文が、実際の PDF プレビューで崩れていないかを確認するためのまとまった fixture がなかった。
- 改行、表、チェックボックス、Mermaid、脚注、危険 HTML の確認を一度に行いづらかった。

## 修正

- `fixtures/markdown-renderer-visual-check.md` を追加し、CommonMark / GFM / Mermaid / 数式 / HTML 混在の確認項目を 1 ファイルにまとめた。
- `src-tauri/src/commands/markdown_preview.rs` に、fixture を複数回 PDF 化してもページ数が安定することを確認する回帰テストを追加した。
- 右ペインは既に `PDFViewer` なので、この fixture は実際の PDF 表示と保存 PDF の両方で同じレンダリング経路を確認できる。

## 再発防止

- この fixture を使って、Markdown の改行、リスト、表、危険 HTML、Mermaid の挙動を定期的に確認する。
- 仕様追加やレンダラー調整の際は、まずこの fixture の通し結果を確認する。
