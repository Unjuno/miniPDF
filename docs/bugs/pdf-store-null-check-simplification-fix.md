# pdfStore.tsのnull/undefinedチェック簡略化

## 症状
- `loadingTask !== null && loadingTask !== undefined`という冗長なチェックが使用されていた
- `pdf !== null && pdf !== undefined`という冗長なチェックが使用されていた
- `p !== null && p !== undefined`という冗長なチェックが使用されていた

## 根本原因
`src/stores/pdfStore.ts`と`src/components/PDFViewer.tsx`で、`!== null && !== undefined`という冗長なチェックが使用されていました。TypeScriptでは、`!= null`を使用することで、`null`と`undefined`の両方を一度にチェックできます。

## 修正内容
冗長なチェックを簡潔な方法に変更しました：

1. `loadingTask !== null && loadingTask !== undefined` → `loadingTask != null`
2. `pdf !== null && pdf !== undefined` → `pdf != null`
3. `p !== null && p !== undefined` → `p != null`（型ガードとして使用）

## 修正ファイル
- `src/stores/pdfStore.ts`
- `src/components/PDFViewer.tsx`

## 回帰防止
- `!= null`は`null`と`undefined`の両方をチェックするため、動作は同じです
- コードがより簡潔になり、可読性が向上しました

