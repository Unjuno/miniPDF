# Bug Note: プレビューで改行・区切り線・ネストリストが欠落していた

- **日付**: 2026-05-15
- **種別**: Bug
- **ステータス**: 修正済み
- **対象**: `src-tauri/src/commands/markdown_preview.rs`（Markdown → プレビュー PDF）

## 症状

- 段落内の単一改行が空白に潰れ、エディタ上の改行と PDF の見え方が一致しない。
- `---` などの **thematic break** が何も出ない。
- 箇条書きの **ネストした項目**が親の先頭段落に吸い込まれ、欠落する。

## 根因

- Comrak の `SoftBreak` を常に半角スペースへ変換していた（HTML の CommonMark 既定に近い挙動）。
- `NodeValue::ThematicBreak` を `push_block` で無視していた。
- リストをフラット化する際、`Item` の **先頭段落だけ**を取り、子 `List` を再帰していなかった。

## 修正

- `SoftBreak` を `\n` にし、`layout_spans_lines` の既存ロジックで複数行として描画。
- `ThematicBreak` を `MarkdownBlock::ThematicBreak` として保持し、細い水平線を描画。
- `flatten_list_blocks` で子 `List` を `indent+1` で再帰展開。`ListItem` / `OrderedListItem` に `indent` を追加し描画で字下げ。
- 引用内のリストも同じフラット化ロジックを使い、字下げ＋`•` / 番号の接頭辞で段落化。

## 再発防止

- `markdown_preview::tests` に **単一改行**・**ネスト箇条**・**区切り線**のパース検証を追加。

## 検証

- `cargo test --lib`（`src-tauri`）が成功すること。

## 残存リスク / 未対応

- **同一リスト項目内の複数段落**（空行で区切る継続段落）は、番号リストで「番号の重複表示」などが残る可能性がある（別イシューで扱う）。
- 取り消し線・脚注などは引き続き描画を簡略化している。
