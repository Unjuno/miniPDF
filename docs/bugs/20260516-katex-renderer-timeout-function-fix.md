# KaTeX renderer timeout function fix

## Symptom
- 数式を KaTeX で画像化する専用レンダラーが、テスト実行時に `None` を返していた
- その結果、インライン数式とブロック数式の回帰テストが失敗していた

## Root Cause
- Node ヘルパーで `page.waitForTimeout()` を使っていた
- この環境の Puppeteer ではその API が使えず、スクリプトが例外で終了していた

## Fix
- `page.waitForTimeout()` をやめ、`document.fonts.ready` と `setTimeout` ベースの待機に置き換えた
- ヘルパーの stdout から PNG を返す経路をそのまま維持した

## Regression Prevention
- `render_math_with_katex()` の unit test を維持する
- `cargo test` で KaTeX の inline / display 数式レンダリングを継続確認する
- Node ヘルパーの実行可否は、今後もテストで直接検証する
