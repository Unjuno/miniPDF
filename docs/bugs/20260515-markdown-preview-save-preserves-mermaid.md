# Bug note: Markdown プレビュー保存時に Mermaid 図が落ちる

- **日付**: 2026-05-15
- **種別**: Bug
- **ステータス**: 修正済み

## 症状

Markdown から生成したプレビュー PDF を保存すると、Mermaid 図が保存先 PDF から消えることがあった。

## 原因

保存時に `pdfStructure` から PDF を再生成しており、Markdown プレビュー由来の Mermaid 図は `ImageElement.data` が空のままだったため、再生成時に画像として復元できなかった。

## 修正

- Markdown プレビュー由来の一時 PDF は、そのまま保存するようにした。
- これにより、Mermaid を含む描画結果を再生成経由で失わないようにした。

## 再発防止

- `previewPdfPath` と現在の `pdfStructure.filePath` が一致するとき、保存処理が再生成ではなく元 PDF のバイト列を使うことを `pdfStore.test.ts` で検証する。
- `savePdf` の回帰テストを実行する。
