# pdfImageExtractor.tsのconsole.warnをloggerに統一

## 症状
- `console.warn`が直接使用されている
- ログの一貫性がなく、本番環境でのログレベル制御ができない

## 根本原因
`pdfImageExtractor.ts`で`console.warn`が直接使用されており、構造化ログシステム（`logger`）が使用されていませんでした。これにより、ログの一貫性がなく、本番環境でのログレベル制御ができませんでした。

## 修正内容
`pdfImageExtractor.ts`で、以下の置き換えを行いました：

1. `logger`のインポートを追加
2. `console.warn` → `logger.warn`
3. 開発環境チェックを削除（`logger`が自動的にログレベルを制御するため）

## 修正ファイル
- `src/utils/pdfImageExtractor.ts`

## 回帰防止
- 構造化ログシステムを使用することで、ログの一貫性が向上しました
- 本番環境でのログレベル制御が可能になりました
- ログに構造化されたコンテキスト情報が含まれるようになりました

