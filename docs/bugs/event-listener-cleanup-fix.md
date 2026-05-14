# イベントリスナーのクリーンアップ問題修正

## 症状
- コンポーネントがアンマウントされたときにイベントリスナーが残る可能性がある
- メモリリークが発生する可能性がある

## 根本原因
`TextBlockOverlay`と`ImageOverlay`で、`handleMouseDown`内で`globalThis.addEventListener`を呼び出していましたが、コンポーネントがアンマウントされたときにイベントリスナーをクリーンアップする処理が不足していました。また、`PageBreakEditor`で`rafId`のクリーンアップが不完全でした。

## 修正内容
1. `TextBlockOverlay.tsx`:
   - `useEffect`のインポートを追加
   - `handleMouseDown`のコールバック内でクリーンアップ関数を返すように修正
   - コンポーネントがアンマウントされたときにドラッグ状態をリセットする`useEffect`を追加

2. `ImageOverlay.tsx`:
   - `useEffect`のインポートを追加
   - `handleMouseDown`のコールバック内でクリーンアップ関数を返すように修正
   - コンポーネントがアンマウントされたときにドラッグ状態をリセットする`useEffect`を追加

3. `PageBreakEditor.tsx`:
   - `rafId`のクリーンアップ時に`null`を設定するように修正

## 修正ファイル
- `src/components/TextBlockOverlay.tsx`
- `src/components/ImageOverlay.tsx`
- `src/components/PageBreakEditor.tsx`

## 回帰防止
- イベントリスナーのクリーンアップにより、メモリリークを防ぎました
- コンポーネントがアンマウントされたときにドラッグ状態をリセットすることで、状態の不整合を防ぎました
- `rafId`のクリーンアップを改善することで、不要なアニメーションフレームの実行を防ぎました

