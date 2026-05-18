# KaTeX PDF black box normalization

## Symptom
- KaTeX でレンダリングした数式は PNG として生成できていた
- しかし PDF に埋め込むと、数式が黒い矩形のように見えた

## Root Cause
- PDF 生成経路で KaTeX の PNG をそのまま画像 XObject として埋め込んでいた
- この環境の PDF レンダラでは、PNG 埋め込みが見た目上の黒ベタ化を起こしていた

## Fix
- KaTeX の PNG を白背景に合成して JPEG に再エンコードしてから埋め込むようにした
- PDF には `DCTDecode` の JPEG 画像として入るようにした

## Regression Prevention
- `render_math_with_katex()` の unit test を残す
- `display_math_blocks_render_without_dollars` で `/DCTDecode` と `/SMask` 不在を検証する
- 数式が画像として出るだけでなく、PDF 上で見た目が黒ベタ化しないことを手順書で確認する
