# ImageOverlayとTextBlockOverlayのエラーハンドリング追加

## 症状
- `handleMouseMove`内で`await moveImage`と`await moveTextBlock`を呼び出しているが、エラーハンドリングが不足している
- 未処理のPromise拒否が発生する可能性がある
- エラーが発生してもユーザーに通知されない

## 根本原因
`ImageOverlay.tsx`と`TextBlockOverlay.tsx`で、`handleMouseMove`内で`await moveImage`と`await moveTextBlock`を呼び出していましたが、エラーハンドリングがありませんでした。これにより、未処理のPromise拒否が発生する可能性がありました。

また、`handleResize`と`handleMove`でも同様の問題がありました。

## 修正内容
1. `handleMouseMove`内で`try-catch`ブロックを追加し、エラーをログに記録し、ドラッグを停止
2. `handleResize`と`handleMove`でも`try-catch`ブロックを追加し、エラーをログに記録
3. `scaleX`と`scaleY`のチェックに`!isFinite`チェックを追加

## 修正ファイル
- `src/components/ImageOverlay.tsx`
- `src/components/TextBlockOverlay.tsx`

## 回帰防止
- エラーハンドリングにより、未処理のPromise拒否を防ぎました
- エラーが発生した場合、ログに記録され、ドラッグが停止されるようになりました
- ユーザーにエラーが通知されるようになりました（ログを通じて）

