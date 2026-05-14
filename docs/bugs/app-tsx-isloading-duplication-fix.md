# App.tsxのisLoading状態の重複を解消

## 症状
- `App.tsx`でローカルの`isLoading` stateと`pdfStore`の`isLoading`フラグが重複している
- 状態の不整合が発生する可能性がある

## 根本原因
`App.tsx`で、ローカルの`isLoading` stateを使用していましたが、`pdfStore`にも`isLoading`フラグがあります。これにより、状態の重複と不整合が発生する可能性がありました。

## 修正内容
1. ローカルの`isLoading` stateを削除
2. `pdfStore`の`isLoading`フラグを使用するように変更
3. `setIsLoading`の呼び出しを削除（`loadPdf`と`savePdf`関数内で`isLoading`フラグが管理されるため）

## 修正ファイル
- `src/App.tsx`

## 回帰防止
- 状態の重複を解消し、一貫性を保つようになりました
- `loadPdf`と`savePdf`関数内で`isLoading`フラグが適切に管理されるため、状態の不整合を防ぎました

