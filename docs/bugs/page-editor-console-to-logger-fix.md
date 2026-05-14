# PageEditor.tsxのconsole.log/warn/errorをloggerに統一

## 症状
- `console.log`、`console.warn`、`console.error`が直接使用されている
- ログの一貫性がなく、本番環境でのログレベル制御ができない

## 根本原因
`PageEditor.tsx`で`console.log`、`console.warn`、`console.error`が直接使用されており、構造化ログシステム（`logger`）が使用されていませんでした。これにより、ログの一貫性がなく、本番環境でのログレベル制御ができませんでした。

## 修正内容
`PageEditor.tsx`で、以下の置き換えを行いました：

1. `console.log` → `logger.debug`
2. `console.warn` → `logger.warn`
3. `console.error` → `logger.error`

## 修正ファイル
- `src/components/PageEditor.tsx`

## 回帰防止
- 構造化ログシステムを使用することで、ログの一貫性が向上しました
- 本番環境でのログレベル制御が可能になりました
- ログに構造化されたコンテキスト情報が含まれるようになりました

