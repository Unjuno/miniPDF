# Bug Note: 未閉じ Mermaid fence が後続 Markdown を飲み込む問題

- **日付**: 2026-05-18
- **種別**: Bug
- **ステータス**: 修正済み
- **対象**: `src-tauri/src/commands/markdown_preview.rs`

## 症状

- ` ```mermaid ` で始まる Mermaid ブロックを閉じ忘れた Markdown で、後続の見出しや表まで 1 つのコードブロックとして扱われることがあった。
- visual-check 用の長い fixture では、末尾の Unicode / 変数表 / 単位チェックが見出しとして残らず、レイアウト確認がしづらかった。

## 原因

- Markdown パーサは未閉じ fenced code block を文末まで 1 ブロックとして扱う。
- Mermaid の壊れた図を安全にフォールバックする処理はあったが、未閉じ fence の終端を補う処理がなかった。

## 修正

- Markdown 正規化で未閉じの Mermaid fence を検出し、次の見出しの前で自動的に閉じるようにした。
- visual-check fixture に 17 章以降の後半セクションを追加し、末尾の見出しが残ることを確認できるようにした。
- Markdown→PDF を直接吐く CLI を追加し、生成された PDF をそのまま確認できるようにした。

## 再発防止

- `unclosed_mermaid_fence_stops_before_following_heading` を保持する。
- `markdown_renderer_visual_check_fixture_is_stable_and_safe` で visual-check fixture の末尾見出しも検証する。
- CLI で `fixtures/markdown-renderer-visual-check.md` を出力して、PDF の末尾ページを目視確認する。
