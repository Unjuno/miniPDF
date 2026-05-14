# 配列操作の安全性向上

## 症状
- `pages.flatMap(p => p.images)`や`pageData.images.map`で`undefined`エラーが発生する可能性がある
- 配列が`undefined`の場合に実行時エラーが発生する

## 根本原因
`pdfStore.ts`で、配列操作（`flatMap`、`map`）を行う際に、`page.images`や`page.textBlocks`が`undefined`の場合のチェックが不足していました。これにより、実行時エラーが発生する可能性がありました。

## 修正内容
以下の箇所で安全な配列アクセスを追加しました：

1. `resizeImage`関数:
   - `snapshotPdfStructure.pages.flatMap(p => p.images)` → `snapshotPdfStructure.pages.flatMap(p => p.images || [])`
   - `snapshotPdfStructure.pages.flatMap(p => p.images.map(...))` → `snapshotPdfStructure.pages.flatMap(p => (p.images || []).map(...))`

2. `loadPdf`関数:
   - `page.images.map(...)` → `(page.images || []).map(...)`

3. `setCurrentPage`関数:
   - `pageData.images.map(...)` → `(pageData.images || []).map(...)`
   - `pageData.textBlocks.map(...)` → `(pageData.textBlocks || []).map(...)`

## 修正ファイル
- `src/stores/pdfStore.ts`

## 回帰防止
- 安全な配列アクセスにより、`undefined`エラーを防ぎました
- すべての配列操作で`|| []`を使用することで、一貫性が向上しました
- エラーメッセージを統一することで、ユーザーが問題を理解しやすくなりました

