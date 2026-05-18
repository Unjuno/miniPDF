# Markdown editor Enter preserves cursor position

## Symptom
- Markdown editor で Enter を押したあと、カーソル位置が見えなくなり、プレビューが一番下のページへ飛ぶことがあった。

## Root Cause
- Enter で hard break を挿入したあと、textarea の selection を明示的に元の位置へ戻していなかった。
- そのため、controlled textarea の再描画後に caret が不安定になり、プレビュー同期の基準もずれていた。

## Fix
- Enter 直後に textarea の cursor offset を復元するようにした。
- `insertMarkdownHardBreak()` の戻り値を使って、入力位置とプレビュー同期の基準を揃えた。

## Regression Prevention
- `restoreTextareaCursor()` の unit test を追加した。
- `npm test` で Enter 後の cursor selection の復元を確認する。
