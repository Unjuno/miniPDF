# 型安全性の改善

## 症状
- `error: any`型の使用により型安全性が低下している
- `e.target as HTMLElement`の型アサーションが不適切

## 根本原因
`PDFViewer.tsx`で、`error: any`型が使用されており、型安全性が低下していました。また、`e.target as HTMLElement`の型アサーションが不適切で、実行時エラーが発生する可能性がありました。

## 修正内容
1. `error: any` → `error: unknown`に変更し、型安全にプロパティにアクセス
2. `e.target as HTMLElement` → `e.target instanceof HTMLElement`チェックを追加

## 修正ファイル
- `src/components/PDFViewer.tsx`

## 回帰防止
- `unknown`型を使用することで、型安全性が向上しました
- `instanceof`チェックにより、実行時エラーを防ぎました
- 型チェックにより、IDEの補完が効くようになりました

