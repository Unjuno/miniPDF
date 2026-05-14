# PDFViewer.tsxのrafIdをuseRefで管理する修正

## 症状
- `useEffect`内で`rafId`がローカル変数として宣言されていたため、クリーンアップ関数から正しく参照できない可能性がある
- クロージャの問題により、古い`rafId`が参照される可能性がある

## 根本原因
`PDFViewer.tsx`で、`rafId`が`useEffect`内で`const rafId = requestAnimationFrame(...)`として宣言されていました。これにより、クリーンアップ関数が古い`rafId`を参照する可能性がありました。

## 修正内容
`rafId`を`useRef`で管理するように変更しました：

1. `zoomRafIdRef`を`useRef<number | null>(null)`として宣言
2. `requestAnimationFrame`の前に既存の`rafId`をキャンセル
3. `requestAnimationFrame`の結果を`zoomRafIdRef.current`に保存
4. コールバック内で`zoomRafIdRef.current`を`null`に設定
5. クリーンアップ関数で`zoomRafIdRef.current`を使用して最新の値を参照

## 修正ファイル
- `src/components/PDFViewer.tsx`

## 回帰防止
- `rafId`を`useRef`で管理することで、クリーンアップ関数が最新の`rafId`を参照できるようになりました
- クロージャの問題を回避し、メモリリークを防ぎました
- `requestAnimationFrame`のキャンセルが確実に実行されるようになりました

