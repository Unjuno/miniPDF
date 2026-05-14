# Bug Note: markdown-print-preview-defaults

## Symptom
- Markdown プレビューを開くと 1 ページ表示が初期値になっており、印刷プレビュー用途なのにページ区切りを強く意識させる UI になっていた。
- プレビュー未生成時の空状態も `PDFファイルを開いてください` となっていて、Markdown ベースの使い方と噛み合っていなかった。

## Root Cause
- `PDFViewer` の初期 `viewMode` が `single` のままだった。
- 空状態メッセージが PDF 読み込み用の文言から更新されていなかった。
- 連続表示用の CSS でもページ間の余白と影が残っていた。

## Fix
- `PDFViewer` の初期表示を `continuous` に変更した。
- 空状態メッセージを Markdown 印刷プレビュー向けの文言に変更した。
- 連続表示時はページ間余白とページ影を抑えて、区切りを感じにくい見た目に調整した。

## Prevent Regression
- `PDFViewer.clear-state.test.tsx` に、空状態文言と連続表示初期化を検証するテストを追加した。
