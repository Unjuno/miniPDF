# Bug note: Markdown プレビュー PDF のレンダリング不足

- **日付**: 2026-05-15
- **種別**: Bug
- **ステータス**: 修正済み

## 症状

ライブプレビュー用 PDF で、太字・斜体・インラインコード・コードブロック・引用・表・（外部 CLI なし時の）Mermaid が期待どおりに見えない。

## 原因

独自の行ベース Markdown パーサが段落をプレーンテキストとして扱い、GFM のインライン／表構造を解釈していなかった。コードブロックは `[lang code block]` ラベル付きでソースが二重に見えやすかった。

## 修正

- `comrak`（`default-features = false`）で GFM をパースし、`Strong` / `Emph` / `Code` 等から `InlineSpan` を生成。
- 本文は `NotoSansJP`、太字は `NotoSansJP-Bold`（**同梱があるとき**）。無いときは欧文太字のみ Helvetica Bold、和文太字は本文＋やや濃い色で代用。
- フェンス付きコードは薄い背景矩形の上にソースのみ表示（Mermaid 失敗時は短い案内＋ソース）。
- 引用は左縦バー＋本文。表は格子罫線＋セル内テキスト（1 行目のみ折り込み）。

## 再発防止

- `cargo test --lib markdown_preview` にインライン太字＋コードのパース検証を追加。
- Mermaid は引き続き `mmdc`（`MINIPDF_MERMAID_CLI`、リポジトリの `node_modules/.bin`、`npm install` 後、または PATH）が必要。

## 検証

- `cargo test --lib`
- `npx vitest run`（リグレッション確認）

## 追記（2026-05-15）

- コードブロック背景の `fill` のグレーが PDF の描画状態に残り、続く文字が同じグレーで描かれていた → 各 `text()` で `set_fill_color`（ほぼ黒）を指定。
- Courier / Oblique で日本語が `?` になる → 非 ASCII のコード行・Mermaid 案内は `NotoSansJP`、インラインコードは ASCII のみ Courier、それ以外は本文フォント。
- インラインコードに薄い背景ボックス、引用に背景＋太めの左バー、表に罫線＋黒字を追加。

## 追記（2026-05-15 ②）

- **太字が `??`**: `NotoSansJP-Bold` が無いときに Helvetica Bold で日本語を描いていた → 非 ASCII の太字は本文フォントに戻し、色をわずかに濃くして区別。本物の太字は `src-tauri/fonts/NotoSansJP-Bold.ttf` 同梱で解決。
- **表の罫線がずれる**: 行ごとに縦線を重ね描きしていた → 表全体の外周に対し横線 `行数+1`・縦線 `列数+1` のみ描画するよう変更。
