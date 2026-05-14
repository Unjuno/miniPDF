# 残りのコンポーネントのconsole.log/warn/errorをloggerに統一

## 症状
- 複数のコンポーネントで`console.log`、`console.warn`、`console.error`が直接使用されている
- ログの一貫性がなく、本番環境でのログレベル制御ができない

## 根本原因
以下のコンポーネントで`console.log`、`console.warn`、`console.error`が直接使用されており、構造化ログシステム（`logger`）が使用されていませんでした：

- `PageBreakEditor.tsx`
- `TextBlockOverlay.tsx`
- `ImageOverlay.tsx`
- `InlineTextEditor.tsx`
- `ImageInserter.tsx`
- `TextEditor.tsx`

## 修正内容
各コンポーネントで、以下の置き換えを行いました：

1. `logger`のインポートを追加
2. `console.log` → `logger.debug`
3. `console.warn` → `logger.warn`
4. `console.error` → `logger.error`

## 修正ファイル
- `src/components/PageBreakEditor.tsx`
- `src/components/TextBlockOverlay.tsx`
- `src/components/ImageOverlay.tsx`
- `src/components/InlineTextEditor.tsx`
- `src/components/ImageInserter.tsx`
- `src/components/TextEditor.tsx`

## 回帰防止
- 構造化ログシステムを使用することで、ログの一貫性が向上しました
- 本番環境でのログレベル制御が可能になりました
- ログに構造化されたコンテキスト情報が含まれるようになりました

