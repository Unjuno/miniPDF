# Bug Note: 連続表示でプレビューが真っ白

## 日付

2026-05-15

## 種別

Bug

## ステータス

修正済み

## 対象

`src/components/PDFViewer.tsx`

## 影響範囲

連続表示（既定）で Markdown プレビューを利用するすべてのユーザー

## 症状

右ペインにページ操作バーだけ表示され、PDF 本文が一切描画されない（白いまま）。

## 根因

`useEffect` が `if (!pdfStructure || !canvasRef.current) return` で PDF 読み込みを開始する前に抜けていた。連続表示では各ページの `<canvas>` は `canvasRefsMap` にだけマウントされ、単一ページ用の `canvasRef` は **常に null** のため、PDF.js の `getDocument` が一度も走らず `pdfDoc` が null のままだった。

## 修正

- PDF 読み込み effect の前提条件から `canvasRef.current` を外す
- ズーム用 effect は `pdfDoc` のみ必須とし、1 ページ表示の分岐の中だけ `canvasRef` を検証する

## 検証

`npm test -- --run`

## 再発防止

連続表示と単一ページで ref の付き方が違うため、「読み込み」と「描画」で必要な DOM を取り違えないこと。
