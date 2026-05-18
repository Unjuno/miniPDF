# Markdown nested blockquote line breaks

## Symptom
- ネストした引用が同じ行に結合されて、`Level 1> Level 2>> Level 3` のように読みにくく見えていた

## Root Cause
- `BlockQuote` の再帰処理で、親引用と子引用の境界に改行を入れていなかった
- その結果、ネストした引用が 1 行の連結文字列として描画されていた

## Fix
- 再帰的に子 `BlockQuote` を処理する前に改行を挿入した
- ネストした引用を行単位で分けて、内容が潰れないようにした

## Regression Prevention
- `nested_blockquote_levels_insert_line_breaks` の unit test を追加した
- 引用の visual check fixture でネスト引用の見え方を継続確認する
