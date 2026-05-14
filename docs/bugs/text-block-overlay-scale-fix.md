# テキストブロックオーバーレイのスケール問題修正

## 症状
- テキスト編集時コードブロックがずれる
- テキスト編集時プレビュー反映がされていない
- 編集時に計算中の情報がびょうがされ表記ずれみたいになる

## 根本原因
`TextBlockOverlay`と`InlineTextEditor`で、`scaleX`や`scaleY`が0の場合にゼロ除算が発生し、`Infinity`や`NaN`が生成されていました。これにより、テキストブロックの位置やサイズが正しく計算されず、プレビューがずれる問題が発生していました。

## 修正内容
以下の2つのコンポーネントにゼロ除算チェックを追加しました：

1. `TextBlockOverlay`:
   - `handleMouseMove`: `scaleX`または`scaleY`が0の場合、早期リターン
   - `position`の計算: `scaleX`または`scaleY`が0または無効な値の場合、デフォルト値を返す

2. `InlineTextEditor`:
   - `safeScaleX`と`safeScaleY`を追加し、0または無効な値の場合に1を使用
   - スタイル計算で`safeScaleX`と`safeScaleY`を使用

## 修正ファイル
- `src/components/TextBlockOverlay.tsx`
- `src/components/InlineTextEditor.tsx`

## 回帰防止
- ゼロ除算チェックにより、`scaleX`や`scaleY`が0の場合でもエラーが発生しなくなりました
- 警告ログを出力することで、問題の原因を特定しやすくなりました
- デフォルト値を使用することで、レンダリングエラーを防ぎました

