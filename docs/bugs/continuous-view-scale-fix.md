# 連続表示モードのスケール計算問題修正

## 症状
- 連続表示でページの表示に問題がある問題、おそらく移動できるスクロールバーが１ページ表示と同じようになっている
- 連続表示で異なるページでの移動時に移動できない問題、画像とテキスト調整できない

## 根本原因
`PDFViewer.tsx`の連続表示モードで、`pageStructure.width`や`pageStructure.height`、`zoomLevel`が0または無効な値の場合にゼロ除算が発生し、`NaN`や`Infinity`が生成されていました。これにより、オーバーレイの位置やサイズが正しく計算されず、スクロール位置も正しく更新されませんでした。

## 修正内容
1. 連続表示モードのページレンダリング:
   - `pageStructure.width`、`pageStructure.height`、`zoomLevel`が0または無効な値の場合をチェック
   - 無効な値の場合は`continue`でスキップ

2. `overlayScale`の計算:
   - `currentPageStructure.width`や`currentPageStructure.height`が0または無効な値の場合をチェック
   - 無効な値の場合はデフォルト値`{ scaleX: 1, scaleY: 1 }`を返す

3. 改ページエディタの高さ計算:
   - `overlayScale.scaleY`が0または無効な値の場合をチェック
   - 無効な値の場合は元の`currentPageStructure.height`を使用

## 修正ファイル
- `src/components/PDFViewer.tsx`

## 回帰防止
- ゼロ除算チェックにより、無効な値の場合でもエラーが発生しなくなりました
- 警告ログを出力することで、問題の原因を特定しやすくなりました
- デフォルト値を使用することで、レンダリングエラーを防ぎました

