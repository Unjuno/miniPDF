# Bug Note: 引用内の fenced code がプレビューから落ちる問題

- **日付**: 2026-05-18
- **種別**: Bug
- **ステータス**: 修正済み
- **対象**: `src-tauri/src/commands/markdown_preview.rs`

## 症状

- `> ```js` のような引用内コードブロックが、Markdown Renderer Visual Check のプレビューで表示されず、引用の途中で内容が抜けることがあった。
- 引用の背景や左罫線は出ているのに、コードブロック部分だけ欠落して見えることがあった。

## 修正

- 引用ノード収集中に `CodeBlock` を捨てず、`BlockquoteLine::CodeBlock` として保持するようにした。
- PDF 側では引用内コードブロックを引用コンテナ内の独立したコードボックスとして描画するようにした。
- HTML 側でも `blockquote-code` として同じ見え方になるようにした。
- 回帰テストとして、引用内 fenced code が保持されることを追加した。

## 再発防止

- 引用内に段落・数式・リスト・コードブロックを混在させた fixture を通し、欠落がないか確認する。
- `collect_blockquote_content` に新しい子ノード種別を追加するときは、必ず描画側の `BlockquoteLine` も更新する。
