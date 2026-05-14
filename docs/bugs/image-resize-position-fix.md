# 画像の縮尺変更と位置調整の問題修正

## 症状
- 画像の縮尺変更が失敗する
- 画像の位置があっていない

## 根本原因
`ImageOverlay.tsx`の`handleResize`、`handleMove`、`handleMouseMove`関数で、`scaleX`や`scaleY`が0の場合にゼロ除算が発生し、`Infinity`が生成されていました。これにより、PDF生成時にエラーが発生していました。

## 修正内容
以下の3つの関数にゼロ除算チェックを追加しました：

1. `handleResize`: `scaleX`または`scaleY`が0の場合、早期リターン
2. `handleMove`: `scaleX`または`scaleY`が0の場合、早期リターン
3. `handleMouseMove`: `scaleX`または`scaleY`が0の場合、早期リターン

## 修正ファイル
- `src/components/ImageOverlay.tsx`

## 回帰防止
- ゼロ除算チェックにより、`scaleX`や`scaleY`が0の場合でもエラーが発生しなくなりました
- 警告ログを出力することで、問題の原因を特定しやすくなりました

