# pdfImageExtractor.tsの非nullアサーション修正

## 症状
- `transformStack.pop()!`で非nullアサーション演算子を使用していた
- 型安全性が低下している

## 根本原因
`src/utils/pdfImageExtractor.ts`で、`transformStack.pop()!`を使用していました。`if (transformStack.length > 0)`チェックの後なので実行時エラーは発生しませんが、型安全性の観点から改善の余地がありました。

## 修正内容
非nullアサーション演算子を削除し、より安全な方法に変更しました：

1. `transformStack.pop()`の結果を変数に保存
2. 結果が存在する場合のみ`currentTransform`を更新

## 修正ファイル
- `src/utils/pdfImageExtractor.ts`

## 回帰防止
- 非nullアサーション演算子を削除することで、型安全性が向上しました
- 実行時エラーのリスクを低減しました

