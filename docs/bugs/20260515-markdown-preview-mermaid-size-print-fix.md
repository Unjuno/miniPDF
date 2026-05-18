# Bug note: Markdown プレビューの Mermaid 図が大きすぎる / 印刷で見切れる

- **日付**: 2026-05-15
- **種別**: Bug
- **ステータス**: 修正済み

## 症状

Mermaid 図が HTML プレビューで自然サイズのまま扱われ、図がやたら大きく見えたり、印刷時にページ外へ見切れることがあった。

## 原因

- 生成した PNG を `<img>` にそのまま流し込み、ページ幅・最大高さの制限が弱かった。
- Mermaid ブロック全体に対する `break-inside` / `page-break-inside` の指定が弱く、印刷時のページ分割で安定しなかった。
- 失敗時のフォールバックが長いコードブロックになっていて、ページ見積もりと表示がずれやすかった。

## 修正

- Mermaid 画像を専用フレームで包み、ページ幅と最大高さを CSS で制限した。
- `break-inside: avoid` と `page-break-inside: avoid` を Mermaid 用に明示した。
- 失敗時はコードブロックではなく、短い説明付きプレースホルダを表示するようにした。

## 再発防止

- `cargo test markdown_preview --target-dir target-test -- --nocapture`
- `html_preview_constrains_mermaid_images_and_fallbacks` で Mermaid フレームと印刷制約を検証する
- `mermaid_fallback_is_short_and_not_a_code_block` でフォールバックの簡略化を検証する
