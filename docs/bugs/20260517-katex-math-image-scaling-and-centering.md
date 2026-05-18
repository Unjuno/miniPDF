# KaTeX math image scaling and centering

## Symptom
- KaTeX の数式は PDF に画像として入るようになったが、行内数式が大きすぎたり、ブロック数式の位置がやや不自然に見える可能性があった

## Root Cause
- 数式画像を自然サイズのまま埋め込んでいた
- 行内数式とブロック数式で、描画高さに対する調整が十分でなかった

## Fix
- 行内数式は行高に収まる範囲で縮小して描画するようにした
- ブロック数式は、画像サイズに合わせて枠サイズを調整しつつ、中央寄せで配置するようにした
- 画像サイズ計算を `scale_math_image()` に切り出した

## Regression Prevention
- `scale_math_image_never_expands_rendered_math` の unit test を追加した
- KaTeX の renderer test と PDF 生成 test を継続実行する
- 画像埋め込み後の PDF を目視確認する際は、数式の縦位置と枠内余白を見る
