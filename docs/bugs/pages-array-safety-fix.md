# pages配列の安全性向上

## 症状
- `structure.pages`や`pdfStructure.pages`が`undefined`の場合に実行時エラーが発生する可能性がある
- 配列操作で`pages`が`undefined`の場合のチェックが不足している

## 根本原因
`pdfStore.ts`で、`structure.pages`や`pdfStructure.pages`に直接アクセスしており、`undefined`の場合のチェックが不足していました。これにより、実行時エラーが発生する可能性がありました。

## 修正内容
以下の箇所で安全な配列アクセスを追加しました：

1. `loadPdf`関数:
   - `structure.pages.map(...)` → `(structure.pages || []).map(...)`
   - `structure.pages.length` → `(structure.pages || []).length`

2. `resizeImage`関数:
   - `currentState.pdfStructure.pages.map(...)` → 既に`currentState.pdfStructure`のnullチェックがあるため、`pages`も安全

3. `addPage`関数:
   - `state.pdfStructure.pages.length` → 既に`state.pdfStructure`のnullチェックがあるため、`pages`も安全

## 修正ファイル
- `src/stores/pdfStore.ts`

## 回帰防止
- 安全な配列アクセスにより、`undefined`エラーを防ぎました
- すべての配列操作で`|| []`を使用することで、一貫性が向上しました
- エラーメッセージを統一することで、ユーザーが問題を理解しやすくなりました

