# PDFViewer.tsxのリンターエラー修正

## 症状
- `isFinite`の使用（`Number.isFinite`を使用すべき）
- `window`の使用（`globalThis.window`を使用すべき）
- `typeof`チェックの使用（`undefined`と直接比較すべき）
- 不要な型アサーション
- ゼロ分数の使用（`1.0` → `1`）
- オプショナルチェーンの未使用
- `findIndex`の使用（単純な値の検索では`indexOf`を使用すべき）
- `for`ループの使用（`for-of`ループを使用すべき）
- 不要な変数代入

## 根本原因
リンターが推奨するベストプラクティスに従っていませんでした。

## 修正内容

### 1. `isFinite`を`Number.isFinite`に置き換え
- `isFinite()`は文字列を数値に変換してからチェックするため、予期しない動作を引き起こす可能性がある
- `Number.isFinite()`は型変換を行わず、より厳密なチェックを行う
- 8箇所を修正

### 2. `window`を`globalThis.window`に置き換え
- `globalThis.window`を使用することで、SSR環境でもエラーを防ぐ
- 9箇所を修正

### 3. `typeof`チェックを`undefined`との直接比較に変更
- `typeof globalThis.window !== 'undefined'` → `globalThis.window !== undefined`
- より簡潔で読みやすいコードに

### 4. オプショナルチェーンの使用
- `!pdfStructure || !pdfStructure.pages` → `!pdfStructure?.pages`
- `!currentPageStructure || !currentPageStructure.textBlocks` → `!currentPageStructure?.textBlocks`
- `!currentPageStructure || !currentPageStructure.images` → `!currentPageStructure?.images`
- `pdfStructure && pdfStructure.pages && pdfStructure.pages.length > 0` → `pdfStructure?.pages && pdfStructure.pages.length > 0`
- より簡潔で読みやすいコードに

### 5. `findIndex`を`indexOf`に置き換え
- 単純な値の検索では`indexOf`を使用することで、パフォーマンスが向上
- 4箇所を修正

### 6. `for`ループを`for-of`ループに置き換え
- `for (let i = 0; i < pdfStructure.pages.length; i++)` → `for (const page of pdfStructure.pages)`
- より簡潔で読みやすいコードに

### 7. ゼロ分数を削除
- `1.0` → `1`
- 整数値にはゼロ分数は不要

### 8. catchブロックのエラーハンドリング改善
- キャンセルエラー以外のエラーをログに記録するように修正

### 9. 不要な変数代入の削除
- `const [pdfJsLoaded, setPdfJsLoaded]` → `const [, setPdfJsLoaded]`
- 使用されていない変数を削除

## 修正ファイル
- `src/components/PDFViewer.tsx`

## 回帰防止
- すべてのテストが通過
- ビルドが成功
- リンターエラーが65個から26個に減少

## 残りのリンター警告
以下の警告は、設計上の問題やリファクタリングが必要なため、今回は対応していません：
- Cognitive Complexity（関数の複雑度が高い）
- CSS inline styles（外部CSSファイルへの移行が必要）
- アクセシビリティの問題（キーボードリスナーの追加が必要）

