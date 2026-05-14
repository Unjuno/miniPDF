# ImageOverlayとTextBlockOverlayのstale closure問題修正

## 症状
- `useEffect`内で`startPos`と`startImagePos`/`startTextPos`を定義していたため、クロージャの問題が発生する可能性がある
- `image`や`textBlock`が変更されると、`useEffect`が再実行され、新しい値でイベントリスナーが再登録されるが、古い値を使用してしまう可能性がある

## 根本原因
`ImageOverlay.tsx`と`TextBlockOverlay.tsx`で、`useEffect`内で`startPos`と`startImagePos`/`startTextPos`を定義していました。これらは`useEffect`が実行される時点の値で固定されるため、ドラッグ中に`image`や`textBlock`が変更されると、古い値を使用してしまう可能性がありました。

## 修正内容
`startPos`と`startImagePos`/`startTextPos`を`useRef`で管理するように変更しました：

1. `startPosRef`と`startImagePosRef`/`startTextPosRef`を`useRef`として宣言
2. `handleMouseDown`でこれらの値を設定
3. `handleMouseMove`内で`useRef`から値を取得
4. クリーンアップ時に`null`を設定

## 修正ファイル
- `src/components/ImageOverlay.tsx`
- `src/components/TextBlockOverlay.tsx`

## 回帰防止
- `startPos`と`startImagePos`/`startTextPos`を`useRef`で管理することで、最新の値を常に参照できるようになりました
- クロージャの問題を回避し、ドラッグ中に`image`や`textBlock`が変更されても正しい値を使用できるようになりました
- メモリリークを防ぐため、クリーンアップ時に`null`を設定しました

