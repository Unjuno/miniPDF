# Bug Note: Markdown の単一改行が一部で崩れるように見えた

- **日付**: 2026-05-16
- **種別**: Bug
- **ステータス**: 修正済み
- **対象**: `src-tauri/src/commands/markdown_preview.rs`

## 症状

- Markdown で半角英数字を入力したあとに `Enter` を押して 1 行進めたつもりでも、プレビューで正しく行が分かれて見えないケースがあった。
- 見た目としては、改行が潰れたように見えたり、検証しづらい状態だった。

## 根因

- Markdown の単一改行を `SoftBreak` と `LineBreak` の両方で `\n` として扱っていたが、回帰テストが不足していた。
- そのため、ASCII と CJK を含む実際の入力で「改行が保持される」ことを固定できていなかった。

## 修正

- `paragraph_soft_breaks_survive_ascii_and_cjk_input` を追加し、ASCII と CJK の両方で単一改行が維持されることを確認した。
- `fixtures/preview-consistency-test.md` に、半角入力後の改行確認用の Markdown を追加した。
- `preview_consistency_fixture_renders_repeatedly_without_drift` を追加し、同じ Markdown を複数回 PDF 化してもページ数が安定することを確認した。

## 再発防止

- 単一改行の扱いは、ASCII と CJK の両方を含む入力でテストする。
- 目視確認用 fixture は `fixtures/preview-consistency-test.md` を使う。
- 生成結果の安定性は、同じ Markdown を複数回通してもページ数が変わらないことを確認する。
