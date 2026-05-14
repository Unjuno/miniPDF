# alert/confirmの置き換え計画

## 症状
- `alert()`と`confirm()`が複数のコンポーネントで使用されている
- ネイティブのブラウザダイアログで、UXが悪く、スタイリングもできない

## 根本原因
以下のコンポーネントで`alert()`と`confirm()`が使用されています：

- `PageEditor.tsx`: `alert()` (3箇所), `confirm()` (1箇所)
- `TextEditor.tsx`: `alert()` (3箇所)
- `InlineTextEditor.tsx`: `alert()` (1箇所)
- `ImageInserter.tsx`: `alert()` (7箇所)

## 推奨対応
1. `alert()`を`Toast`に置き換え:
   - エラーメッセージには`Toast`の`error`タイプを使用
   - 警告メッセージには`Toast`の`warning`タイプを使用
   - 情報メッセージには`Toast`の`info`タイプを使用

2. `confirm()`を確認ダイアログコンポーネントに置き換え:
   - 削除確認などで使用されている`confirm()`を適切な確認ダイアログコンポーネントに置き換え
   - モーダルダイアログコンポーネントを作成し、`confirm()`の代わりに使用

## 実装方法
1. 各コンポーネントで`useToast`を使用:
   ```typescript
   const { error: showError, warning: showWarning, info: showInfo } = useToast();
   ```

2. `alert()`を`Toast`に置き換え:
   ```typescript
   // Before
   alert('エラーメッセージ');
   
   // After
   showError('エラーメッセージ');
   ```

3. 確認ダイアログコンポーネントを作成:
   - `ConfirmDialog`コンポーネントを作成
   - `useConfirmDialog`フックを作成
   - `confirm()`を`useConfirmDialog`に置き換え

## 優先度
- P2（中優先度）: UXの改善が必要だが、機能的な問題ではない

## 注意事項
- `Toast`はグローバルに表示されるため、各コンポーネントで`useToast`を使用する必要がある
- `confirm()`の置き換えには、モーダルダイアログコンポーネントの作成が必要

