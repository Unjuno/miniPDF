# alert/confirmの置き換え実装

## 症状
- `alert()`と`confirm()`が複数のコンポーネントで使用されている
- ネイティブのブラウザダイアログで、UXが悪く、スタイリングもできない

## 根本原因
以下のコンポーネントで`alert()`と`confirm()`が使用されていました：

- `PageEditor.tsx`: `alert()` (3箇所), `confirm()` (1箇所)
- `TextEditor.tsx`: `alert()` (3箇所)
- `InlineTextEditor.tsx`: `alert()` (1箇所)
- `ImageInserter.tsx`: `alert()` (7箇所)

## 修正内容
1. 確認ダイアログコンポーネントとフックを作成:
   - `ConfirmDialog.tsx`: 確認ダイアログコンポーネント
   - `ConfirmDialog.css`: 確認ダイアログのスタイル
   - `useConfirmDialog.ts`: 確認ダイアログフック

2. `PageEditor.tsx`で`alert()`と`confirm()`を置き換え:
   - `alert()` → `showError()` / `showWarning()` (Toast)
   - `confirm()` → `showConfirm()` (確認ダイアログ)

## 修正ファイル
- `src/components/ConfirmDialog.tsx` (新規作成)
- `src/components/ConfirmDialog.css` (新規作成)
- `src/hooks/useConfirmDialog.ts` (新規作成)
- `src/components/PageEditor.tsx`

## 回帰防止
- `Toast`と確認ダイアログにより、UXが向上しました
- スタイリングが可能になり、アプリケーションのデザインと統一されました
- 非同期処理に対応した確認ダイアログにより、より柔軟な操作が可能になりました

## 残りの作業
- `TextEditor.tsx`: `alert()`を`Toast`に置き換え
- `InlineTextEditor.tsx`: `alert()`を`Toast`に置き換え
- `ImageInserter.tsx`: `alert()`を`Toast`に置き換え

