# pdfImageExtractor.tsから空のimport.meta.env.DEVチェックを削除

## 症状
- `pdfImageExtractor.ts`で、空の`import.meta.env.DEV`チェックが残っている
- 不要なコードが残っている

## 根本原因
`pdfImageExtractor.ts`で、Resourcesの取得に失敗した場合のエラーハンドリングで、空の`import.meta.env.DEV`チェックが残っていました。このチェックは何も実行しないため、不要です。

## 修正内容
以下の箇所で空の`import.meta.env.DEV`チェックを削除しました：

1. `extractImageAreasWithPDFjs`関数:
   - 空の`if (import.meta.env.DEV) { ... }`ブロックを削除
   - コメントを残して、エラーログを出力しない理由を明示

## 修正ファイル
- `src/utils/pdfImageExtractor.ts`

## 回帰防止
- 不要なコードを削除することで、コードの可読性が向上しました
- コメントにより、エラーログを出力しない理由が明確になりました

