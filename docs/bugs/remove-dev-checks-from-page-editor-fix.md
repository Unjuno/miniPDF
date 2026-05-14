# PageEditor.tsxからimport.meta.env.DEVチェックを削除

## 症状
- `PageEditor.tsx`で、`logger.debug`や`logger.info`の呼び出しで`import.meta.env.DEV`チェックが残っている
- 本番環境でログが出力されない可能性がある

## 根本原因
`PageEditor.tsx`で、`logger.debug`や`logger.info`の呼び出しで`import.meta.env.DEV`チェックが残っていました。`logger`は既にログレベルを制御しているため、このチェックは不要です。

## 修正内容
以下の箇所で`import.meta.env.DEV`チェックを削除しました：

1. `handleAddPage`関数:
   - `if (import.meta.env.DEV) { logger.debug(...) }` → `logger.debug(...)`
   - `if (import.meta.env.DEV) { logger.info(...) }` → `logger.info(...)`

2. `handleDrop`関数:
   - `if (import.meta.env.DEV) { logger.info(...) }` → `logger.info(...)` (2箇所)

## 修正ファイル
- `src/components/PageEditor.tsx`

## 回帰防止
- `logger`が自動的にログレベルを制御するため、`import.meta.env.DEV`チェックは不要
- 本番環境でも適切なログレベルでログが出力されるようになりました
- ログの一貫性が向上しました

