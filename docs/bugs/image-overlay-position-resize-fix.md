# 画像の位置と縮尺変更の問題修正

## 症状
- 画像の縮尺変更が失敗する
- 画像の位置があっていない

## 根本原因
1. **画像の位置の問題**：
   - `handleMove`関数の座標変換が不正確だった
   - `screenY`は画面座標系のY座標（上から下）で、PDF座標系（下から上）に変換する際の計算が間違っていた

2. **画像の縮尺変更の問題**：
   - `ImageResizer`内で無効な値（0以下、NaN、Infinity）が`handleResize`に渡される可能性があった
   - 最小サイズの計算は正しかったが、無効な値のチェックが不足していた

## 修正内容
1. **`src/components/ImageOverlay.tsx`**：
   - `handleMove`関数の座標変換ロジックを修正し、コメントを追加
   - `screenY`は画面座標系のY座標（上から下）で、PDF座標系（下から上）に変換する際は`pageHeight - (screenY / scaleY) - image.height`が必要であることを明確化

2. **`src/components/ImageResizer.tsx`**：
   - 無効な値（0以下、NaN、Infinity）のチェックを追加
   - 最小サイズの計算に関するコメントを追加

## 回帰防止
- 画像の位置とサイズの座標変換に関するテストを追加することを推奨
- `ImageResizer`と`ImageOverlay`の座標変換ロジックを統一し、テストで検証することを推奨

