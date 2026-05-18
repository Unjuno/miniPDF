# Markdown editor preview sync

## Symptom
- Markdown を書いている位置とプレビュー PDF の位置が連動しづらかった

## Root Cause
- エディタ側のカーソル位置がプレビューの currentPage に反映されていなかった
- プレビューはスクロール連動を持っていたが、エディタからの追従は無かった

## Fix
- textarea のカーソル行を取得し、Markdown 全体の行数から近い PDF ページを推定して currentPage に反映した
- プレビュー側の既存スクロール同期をそのまま活かした

## Regression Prevention
- `getLineNumberFromOffset()` と `estimatePreviewPageFromMarkdown()` の unit test を追加した
- 大きな Markdown でも極端なページ飛びが出ないことを手動で確認する
