# Bug Note: PDF.js で emoji が表示されない

- 症状: CLI で生成した PDF を preview に表示すると、emoji が空白に見えたり、黒っぽい崩れた字形で出たりした。
- 根本原因: emoji の描画を `rusttype` ベースのモノクロラスタライズに落としていたため、Segoe UI Emoji の色グリフを失っていた。加えて、PDF.js 側では emoji フォント埋め込みが不安定だった。
- 修正: emoji フォントの PDF 埋め込みを止め、`swash` で color glyph を直接レンダリングしてから白背景にフラット化した JPEG として埋め込むようにした。
- 追加調整: 位置合わせは PDF の `pt` 単位で持ち、`24 CSS px` 相当の上方向オフセットを入れるようにした。
- 再発防止: `markdown_preview_cli` で生成した PDF を PDF.js で開くハーネスを使い、emoji が色付きで実際に見えることと、基準線からの縦位置が期待どおりに動くことを確認した。
