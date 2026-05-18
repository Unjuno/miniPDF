# Bug note: Markdown プレビューの Mermaid 図が開発起動時に見つからない

- **日付**: 2026-05-15
- **種別**: Bug
- **ステータス**: 修正済み

## 症状

`mmdc` がインストールされていても、`tauri:dev` 実行時の Markdown プレビューで Mermaid 図が画像化されず、コードブロックのまま扱われることがあった。

## 原因

Mermaid CLI の探索が `current_exe()` 基準に偏っており、開発実行時にリポジトリ直下の `node_modules/.bin/mmdc` を拾えないケースがあった。

## 修正

- `MINIPDF_MERMAID_CLI` を最優先にする。
- その次に、ワークスペース直下の `node_modules/.bin/mmdc` を探索する。
- さらに、`current_dir()` の祖先ディレクトリもたどって `node_modules/.bin/mmdc` を探す。

## 再発防止

- `resolve_mmdc_cwd_node_modules_bin()` の unit test を追加し、`cwd` から上位にある `node_modules/.bin/mmdc` を解決できることを検証する。
- `cargo test markdown_preview` を実行する。
