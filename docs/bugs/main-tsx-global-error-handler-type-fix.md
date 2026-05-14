# main.tsxのグローバルエラーハンドラーの型安全性改善

## 症状
- `ErrorEvent`と`PromiseRejectionEvent`の型が正しく処理されていない
- リンターエラーが発生している

## 根本原因
`main.tsx`のグローバルエラーハンドラーで、`ErrorEvent`と`PromiseRejectionEvent`の型が明示的に指定されていませんでした。また、`typeof globalThis !== 'undefined'`のチェックが冗長でした。

## 修正内容
1. `ErrorEvent`と`PromiseRejectionEvent`の型を明示的に指定
2. `typeof globalThis !== 'undefined'`のチェックを`globalThis !== undefined`に変更（より簡潔）

## 修正ファイル
- `src/main.tsx`

## 回帰防止
- 型安全性が向上しました
- リンターエラーが解消されました
- より明確な型指定により、IDEの補完が効くようになりました

