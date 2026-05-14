# PageEditor.tsxのconsole.warnをlogger.warnに置き換え

## 症状
- `PageEditor.tsx`で、`console.warn`が残っている
- ログシステムが統一されていない

## 根本原因
`PageEditor.tsx`で、`handleDrop`関数内で`console.warn`が使用されていました。ログシステムを統一するため、`logger.warn`に置き換える必要がありました。

## 修正内容
以下の箇所で`console.warn`を`logger.warn`に置き換えました：

1. `handleDrop`関数:
   - `console.warn('PDF構造が存在しません')` → `logger.warn('PDF構造が存在しません')`

## 修正ファイル
- `src/components/PageEditor.tsx`

## 回帰防止
- ログシステムが統一され、すべてのログが`logger`を通じて出力されるようになりました
- ログレベルの制御が可能になりました
- ログの一貫性が向上しました

