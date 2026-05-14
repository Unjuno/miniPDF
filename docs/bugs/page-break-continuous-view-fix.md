# 改ページ動作と連続表示の問題修正

## 症状
- 改ページ動作が適切でない
- 連続表示でページの表示に問題がある問題、おそらく移動できるスクロールバーが１ページ表示と同じようになっている
- 連続表示で異なるページでの移動時に移動できない問題、画像とテキスト調整できない

## 根本原因
1. `PageBreakEditor.tsx`で、`page.height`が0の場合にゼロ除算が発生し、`Infinity`が生成されていました。
2. `PDFViewer.tsx`の連続表示モードで、`page.height`や`zoomLevel`が0または無効な値の場合に`NaN`が生成され、スクロール位置が正しく計算されていませんでした。

## 修正内容
1. `PageBreakEditor.tsx`:
   - `handleMouseMove`内で`page.height`が0または無効な値の場合をチェック
   - `scaleY`が0または無効な値の場合をチェック
   - 無効な値の場合は早期リターン

2. `PDFViewer.tsx`:
   - 連続表示モードのスクロール位置計算で、`page.height`や`zoomLevel`が0または無効な値の場合をチェック
   - 無効な値の場合は`continue`でスキップ
   - スクロール位置に基づく現在のページ検出でも同様のチェックを追加

## 修正ファイル
- `src/components/PageBreakEditor.tsx`
- `src/components/PDFViewer.tsx`

## 回帰防止
- ゼロ除算チェックにより、`page.height`や`zoomLevel`が0の場合でもエラーが発生しなくなりました
- 警告ログを出力することで、問題の原因を特定しやすくなりました
- 無効な値の場合はスキップすることで、レンダリングエラーを防ぎました

