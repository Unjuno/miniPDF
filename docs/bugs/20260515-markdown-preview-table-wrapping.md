# Bug Note: Markdown プレビューの表セル長文が 1 行目しか出ない

- **日付**: 2026-05-15
- **種別**: Bug
- **ステータス**: 修正済み
- **対象**: `src-tauri/src/commands/markdown_preview.rs`

## 症状

Markdown 表のセルに長い文字列を入れると、最初の 1 行しか表示されず、残りの内容が切れていた。

## 根因

表描画で `wrap_text()` した結果の先頭行だけを使っており、折り返し後の 2 行目以降を破棄していた。

## 修正

- 各セルの折り返し行をすべて保持する `wrap_table_row_cells()` を追加。
- 表の各行の高さを「最大折り返し行数」に応じて可変に変更。
- 各セルの折り返し行を順番に描画するようにした。

## 再発防止

- `markdown_preview::tests` に、長い表セルが複数行に折り返されることを確認する回帰テストを追加。

## 検証

- `cargo test markdown_preview`
- `cargo test`
