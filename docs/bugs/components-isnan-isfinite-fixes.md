# コンポーネントファイルのisNaN/isFinite修正

## 症状
- `isNaN`と`isFinite`の使用（`Number.isNaN`と`Number.isFinite`を使用すべき）
- 複数のコンポーネントファイルで同様の問題が発生

## 根本原因
リンターが推奨するベストプラクティスに従っていませんでした。

## 修正内容

### 1. `isNaN`/`isFinite`を`Number.isNaN`/`Number.isFinite`に置き換え
- `isNaN()`は文字列を数値に変換してからチェックするため、予期しない動作を引き起こす可能性がある
- `Number.isNaN()`は型変換を行わず、より厳密なチェックを行う
- 同様に、`isFinite()`を`Number.isFinite()`に置き換え

### 修正ファイル
- `src/components/PageEditor.tsx` (1箇所)
- `src/components/ImageOverlay.tsx` (3箇所)
- `src/components/TextBlockOverlay.tsx` (2箇所)
- `src/components/PageBreakEditor.tsx` (2箇所)
- `src/components/InlineTextEditor.tsx` (2箇所)

## 回帰防止
- すべてのテストが通過
- ビルドが成功
- 型安全性が向上

