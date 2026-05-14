# ImageOverlayとTextBlockOverlayのイベントリスナー管理修正

## 症状
- `handleMouseDown`が`useCallback`でメモ化されているが、その中でイベントリスナーを追加し、クリーンアップ関数を返していた
- `useCallback`は関数を返すべきで、クリーンアップ関数を返すべきではない
- イベントリスナーが適切にクリーンアップされない可能性がある

## 根本原因
`ImageOverlay.tsx`と`TextBlockOverlay.tsx`で、`handleMouseDown`が`useCallback`でメモ化されていましたが、その中でイベントリスナーを追加し、クリーンアップ関数を返していました。これはReactの`useCallback`の正しい使い方ではありません。`useCallback`は関数を返すべきで、クリーンアップ関数を返すべきではありません。

## 修正内容
1. `handleMouseDown`を簡素化し、ドラッグ開始のみを処理するように変更
2. イベントリスナーの追加とクリーンアップを`useEffect`内で管理するように変更
3. `isDragging`が`true`のときにのみイベントリスナーを追加するように変更

## 修正ファイル
- `src/components/ImageOverlay.tsx`
- `src/components/TextBlockOverlay.tsx`

## 回帰防止
- イベントリスナーが`useEffect`内で適切に管理されるようになりました
- `useCallback`が正しく使用されるようになりました
- イベントリスナーのクリーンアップが確実に実行されるようになりました
- クロージャの問題を回避し、最新の値を参照できるようになりました

