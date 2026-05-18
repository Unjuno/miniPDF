# Markdown Enter cursor sync overwrite

## Symptom
- Markdown editor で Enter を押すと、プレビューが一番下のページへ飛び、エディタのカーソル位置も消えたように見えた。

## Root Cause
- Enter の hard break 挿入直後に、`onChange` / `onSelect` / `onKeyUp` が発火して、意図したカーソル位置ではなく DOM 側の一時的な selection を使ってしまっていた。
- その結果、プレビュー同期用の currentPage が最後の行基準に更新されることがあった。

## Fix
- Enter 時に確定した cursor offset を一時保存し、後続イベントではその値を優先するようにした。
- textarea の selection 復元も明示的に行い、カーソルを見失わないようにした。

## Regression Prevention
- `resolveEditorCursorOffset()` の unit test を追加した。
- `restoreTextareaCursor()` の unit test と合わせて、Enter 後の caret 復元と同期順序を監視する。
