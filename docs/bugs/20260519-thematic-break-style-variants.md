# Bug Note: 水平線の記法ごとの差が消えていた問題

- **日付**: 2026-05-19
- **種別**: Bug
- **ステータス**: 修正済み
- **対象**: `src-tauri/src/commands/markdown_preview.rs`

## 症状

- `---`、`***`、`___` がすべて同じ水平線として描画されていた。
- visual check では記法ごとの差が見えず、どの入力がどの線に対応するのか分かりにくかった。

## 修正

- Thematic break の parser で、元の記法を保持するようにした。
- PDF と HTML の両方で、記法ごとに別の水平線スタイルを描画するようにした。
- visual check の説明文も、実際の表示仕様に合わせて更新した。

## 再発防止

- `thematic_break_variants_are_all_parsed_as_breaks` のテストで、3 記法が別スタイルに分類されることを確認する。
- visual check の section 19 を、3 種類の線を目視確認できる状態で維持する。
