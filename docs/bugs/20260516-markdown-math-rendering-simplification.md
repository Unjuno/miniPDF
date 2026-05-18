# Markdown Math Rendering Simplification

## Symptom
- `$$ ... $$` のブロック数式が、TeX 記法のまま PDF に表示されていた
- `\sum`、`\frac`、`\sqrt`、行列環境などがそのまま文字列として見えていた
- visual fixture の数式確認で、PDF 上の見た目と期待値がずれていた

## Root Cause
- Comrak の math AST を使わず、Markdown の本文文字列をそのまま描画していた
- display math の判定がテキスト抽出ベースだったため、math ノードを正しく拾えていなかった
- 数式の表示は専用レンダラーではなく、単純なテキスト描画に依存していた

## Fix
- Comrak の `math_dollars` / `math_code` を有効化した
- `NodeValue::Math` を直接処理して、inline / display math を AST ベースで分岐するようにした
- display math は `\sum`、`\frac`、`\sqrt`、行列環境、cases 環境を最低限整形して描画するようにした

## Regression Prevention
- 数式の簡易整形関数に対する unit test を追加した
- `$$ ... $$` が display math として処理される回帰テストを維持する
- visual fixture で数式の見た目が極端に崩れていないことを継続確認する
