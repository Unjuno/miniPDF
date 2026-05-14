# Bug note: Markdown プレビューの日本語・表・連続表示の見え方

## 日付

2026-05-15

## 症状

- Markdown プレビュー PDF で日本語が `?` になる（標準 14 フォントのみ使用していた）。
- `1.` 付きリストが段落に連結され、表は生の `|` 行のまま。
- 連続表示で背景と影が消え、ページ境界が分かりにくい。

## 根本原因

- `markdown_preview` が本文すべて `Font::Helvetica` で描画していた。
- `wrap_text` が `split_whitespace` のみで、日本語の単語区切りがなく改行されないケースがあった。
- パーサが順序付きリスト・表行を扱っていなかった。
- `PDFViewer.css` の continuous モードで背景白・影なし・ページ間マージン 0 にしていた。
- `pdf_generator` の `register_custom_fonts` がログのみで `Document::add_font` を呼んでいなかった。

## 修正

- `font_manager::register_fonts_on_document` で `oxidize` の `add_font` を実行。プレビュー・本生成の両方で利用。
- **Windows**: リポジトリに Noto が無い場合、`%WINDIR%\Fonts\NotoSansJP-VF.ttf` 等の **単一 `.ttf`** を `FontLoader` で検証してから `NotoSansJP` 名でレジストリ登録（日本語 `?` を防ぐ）。
- 開発時は `CARGO_MANIFEST_DIR/fonts` に置いた TTF も解決（`src-tauri/fonts/`）。
- プレビュー本文は `NotoSansJP` が埋め込めたとき `Font::custom("NotoSansJP")`。
- 順序付きリスト・表ブロック（等幅で行描画）、表示幅ベースの `wrap_text`。
- continuous 表示: グレー背景・ページ間マージン・キャンバス影と枠線を復帰。
- 非 ASCII の構造 PDF テキストは、フォント埋め込み成功時に `Font::custom("NotoSansJP")`（未埋め込み時は Helvetica にフォールバックし警告）。

## 再発防止

- `commands::markdown_preview::tests` で順序リスト・表パース・CJK 折り返しを検証。
- 日本語表示には `src-tauri/fonts/NotoSansJP-Regular.ttf`（またはリリース隣接の `fonts/`）が必須であることを README に記載済み。
