# Bug Note: emoji のプレビュー位置が縦にずれる

- 症状: `- 😀` や `日本語 English 123 🚀` のような Markdown をプレビューすると、emoji が本文の基準線から上下にずれて見えることがあった。
- 根本原因: emoji を画像化して描画する際に、`swash` が返す glyph placement ではなく、画像高さの固定比率 `0.80` で縦位置を決めていた。そのため、glyph ごとの実際のベースライン差を反映できていなかった。
- 修正: rasterized glyph の `placement.top` と `placement.height` から、画像の下端からベースラインまでの距離を計算して描画位置に使うようにした。固定比率の縦オフセットは廃止した。
- 再発防止: `emoji_baseline_offset_scales_with_raster_placement` と `emoji_draw_y_uses_the_rasterized_baseline_offset` のテストを追加し、固定比率へ戻した場合に検出できるようにした。
