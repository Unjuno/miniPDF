# PDFViewer.tsxのresize rafIdをuseRefで管理する修正

## 症状
- `useEffect`内の`updateSize`関数で`rafId`がローカル変数として宣言されていたため、クリーンアップ関数から正しく参照できない可能性がある
- クロージャの問題により、古い`rafId`が参照される可能性がある

## 根本原因
`PDFViewer.tsx`で、`updateSize`関数内で`rafId`が`let rafId: number | null = null;`として宣言されていました。これにより、クリーンアップ関数が古い`rafId`を参照する可能性がありました。

## 修正内容
`rafId`を`useRef`で管理するように変更しました：

1. `resizeRafIdRef`を`useRef<number | null>(null)`として宣言
2. `updateSize`関数内で`resizeRafIdRef.current`を使用
3. `requestAnimationFrame`の前に既存の`rafId`をキャンセル
4. `requestAnimationFrame`の結果を`resizeRafIdRef.current`に保存
5. コールバック内で`resizeRafIdRef.current`を`null`に設定
6. クリーンアップ関数で`resizeRafIdRef.current`を使用して最新の値を参照

## 修正ファイル
- `src/components/PDFViewer.tsx`

## 回帰防止
- `rafId`を`useRef`で管理することで、クリーンアップ関数が最新の`rafId`を参照できるようになりました
- クロージャの問題を回避し、メモリリークを防ぎました
- `requestAnimationFrame`のキャンセルが確実に実行されるようになりました

