# Bug Note: thematic break の見た目が Markdown 標準より強すぎる問題

- **日付**: 2026-05-18
- **種別**: Bug
- **ステータス**: 修正済み
- **対象**: `src-tauri/src/commands/markdown_preview.rs`

## 症状

- `---`, `***`, `___` のいずれも同じ `hr` になるのは正しいが、PDF 側の水平線がやや強く、一般的な Markdown プレビューより目立って見えた。

## 修正

- PDF 側の thematic break の線幅と余白を調整し、一般的な Markdown の `hr` に近い見た目へ寄せた。
- `---` / `***` / `___` がすべて同じ `ThematicBreak` として扱われ、HTML 側でも `<hr />` として出ることを回帰テストで固定した。

## 再発防止

- horizontal rule の見た目を調整するときは、HTML 側の `<hr>` と PDF 側の線幅・余白を同時に確認する。
- 3 種類の記法がすべて同じ thematic break としてパースされることを保持する。
