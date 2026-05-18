# Markdown editor Enter inserts hard break

## Symptom
- Markdown editor で Enter を押しても、プレビューに改行が反映されにくいケースがあった

## Root Cause
- Markdown の hard break は行末スペース 2 つ + 改行が必要だが、手入力に依存すると入力漏れが起きやすかった
- その結果、編集時に期待した改行表現が Markdown ソースに入っていないことがあった

## Fix
- Markdown editor で Enter を押したとき、`"  \n"` を挿入するようにした
- これにより、通常のタイピングでも hard break が Markdown ソースに入りやすくなった

## Regression Prevention
- `insertMarkdownHardBreak` の unit test を追加した
- hard break の PDF レンダリングテストと合わせて、入力と出力の両方を監視する
