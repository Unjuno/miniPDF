# Markdown math inline baseline drift

## Symptom
- 数式を含むリスト項目や引用ブロックで、数式が周囲のテキストより下に寄って見えることがあった。
- その結果、`数式:` の直後や引用終端付近で、数式行が少し浮いて見えたり、前後の行と詰まって見えることがあった。

## Root Cause
- 行中の数式画像を通常テキストと同じ行高で描画していたため、見た目の縦方向の余白が足りなかった。
- `draw_rich_block()` のページ見積もりが数式入りの行を特別扱いしておらず、引用やリストの中での余白が保守的ではなかった。

## Fix
- 数式を含む行だけ、必要な縦方向の余白を少し増やすようにした。
- 行中数式の描画位置をわずかに上げて、本文のベースラインに近づけた。
- `markdown-renderer-visual-check.md` に、引用内の数式を含むケースを追加した。

## Regression Prevention
- `math_lines_request_more_vertical_room_than_plain_text_lines` の unit test を追加した。
- `cargo test --lib`, `npm test`, `npm run build` で回帰確認する。
