# PDFViewer.tsxの冗長なdevicePixelRatio計算の修正

## 症状
- 連続表示モードでのスクロール位置計算で、`devicePixelRatio`を使った冗長な計算が行われていた
- `Math.floor((page.height * zoomLevel * devicePixelRatio) / devicePixelRatio)`という計算が使用されていた

## 根本原因
`devicePixelRatio`を掛けてから割るという計算は、実質的に何もしていないため冗長です。`Math.floor((page.height * zoomLevel * devicePixelRatio) / devicePixelRatio)`は`Math.floor(page.height * zoomLevel)`と同じです。

## 修正内容

### 1. 連続表示モードのスクロール位置計算を簡略化
- `Math.floor((page.height * zoomLevel * devicePixelRatio) / devicePixelRatio)` → `Math.floor(page.height * zoomLevel)`
- 不要な`devicePixelRatio`変数の削除

### 2. 連続表示モードのオーバーレイスケール計算を簡略化
- `Math.floor((pageStructure.width * zoomLevel * devicePixelRatio) / devicePixelRatio)` → `Math.floor(pageStructure.width * zoomLevel)`
- `Math.floor((pageStructure.height * zoomLevel * devicePixelRatio) / devicePixelRatio)` → `Math.floor(pageStructure.height * zoomLevel)`
- 不要な`devicePixelRatio`変数の削除

### 3. 累積高さ計算の簡略化
- 同様の冗長な計算を修正

## 修正ファイル
- `src/components/PDFViewer.tsx`

## 回帰防止
- すべてのテストが通過
- ビルドが成功
- コードがより簡潔で読みやすくなった
- パフォーマンスがわずかに向上（不要な計算が削減）

## 注意
- キャンバスのレンダリング時には`devicePixelRatio`が必要ですが、スクロール位置やオーバーレイのスケール計算では不要です
- キャンバスのレンダリング部分（`renderPageToCanvas`など）では`devicePixelRatio`を正しく使用しています

