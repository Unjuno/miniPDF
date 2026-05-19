# Bug Note: 混在 emoji span で日本語の斜体が落ちる

- 症状: `*日本語 😀*` のように日本語と絵文字が同じ span に入ると、日本語部分の斜体が効かず、絵文字の扱いも崩れて見えることがあった。
- 根本原因: `draw_text_segment()` は絵文字を含む span を `draw_mixed_emoji_text()` に分岐していたが、その経路では日本語グラフェムに対するフェイク斜体の適用が抜けていた。
- 修正: `draw_mixed_emoji_text()` で日本語グラフェムに対してもフェイク斜体を適用し、絵文字は従来どおり画像として描画するようにした。
- 再発防止: 混在 emoji span のグラフェム単位で `should_fake_italic_grapheme()` を検証するテストを追加した。
