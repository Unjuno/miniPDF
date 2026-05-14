# PDFViewer.tsxからimport.meta.env.DEVチェックを削除

## 症状
- `PDFViewer.tsx`で、`logger.debug`や`logger.warn`の呼び出しで`import.meta.env.DEV`チェックが残っている
- `console.warn`が残っている
- 本番環境でログが出力されない可能性がある

## 根本原因
`PDFViewer.tsx`で、`logger.debug`や`logger.warn`の呼び出しで`import.meta.env.DEV`チェックが残っていました。`logger`は既にログレベルを制御しているため、このチェックは不要です。また、`console.warn`が残っていました。

## 修正内容
以下の箇所で`import.meta.env.DEV`チェックを削除し、`console.warn`を`logger.warn`に置き換えました：

1. `renderPageNumber`関数:
   - `if (import.meta.env.DEV) { logger.debug(...) }` → `logger.debug(...)` (3箇所)

2. クリーンアップ処理:
   - `if (import.meta.env.DEV) { console.warn(...) }` → `logger.warn(...)` (3箇所)
   - `if (import.meta.env.DEV) { logger.warn(...) }` → `logger.warn(...)` (1箇所)

## 修正ファイル
- `src/components/PDFViewer.tsx`

## 回帰防止
- `logger`が自動的にログレベルを制御するため、`import.meta.env.DEV`チェックは不要
- 本番環境でも適切なログレベルでログが出力されるようになりました
- ログの一貫性が向上しました
- `console.warn`を`logger.warn`に置き換えることで、ログシステムが統一されました

