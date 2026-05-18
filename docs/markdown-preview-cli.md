# Markdown Preview CLI

`src-tauri` に、Markdown を PDF プレビューとして出力する CLI を追加しました。

## 使い方

```bash
cargo run --manifest-path src-tauri/Cargo.toml --bin markdown_preview_cli -- input.md output.pdf
```

repo ルートからは次でも実行できます。

```bash
npm run markdown:preview -- input.md output.pdf
```

または出力先を明示する形でも使えます。

```bash
cargo run --manifest-path src-tauri/Cargo.toml --bin markdown_preview_cli -- input.md --output output.pdf
```

入力ファイルを省略せず、UTF-8 の Markdown ファイルを渡してください。

## 挙動

- Tauri の `render_markdown_to_pdf_preview` と同じレンダリング経路を使います。
- Mermaid は `mmdc` が使える環境でのみ画像化されます。
- `mmdc` が見つからない場合、Mermaid ブロックはフォールバック表示になります。
- 出力先は既存ファイルを上書きします。

## 制約

- 入力は Markdown テキストに限定します。
- 生成結果は PDF ファイルです。
- 画像・Mermaid の描画結果は環境依存です。`MINIPDF_MERMAID_CLI`、`node_modules/.bin`、実行ファイル同階、または PATH で `mmdc` を解決してください。
