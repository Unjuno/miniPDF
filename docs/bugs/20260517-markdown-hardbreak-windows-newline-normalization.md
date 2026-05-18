# Markdown hard break newline normalization

## Symptom
- Windows 由来の Markdown 入力で、行末スペース 2 つ + Enter による改行がプレビューに反映されないように見えることがあった

## Root Cause
- Markdown をそのまま comrak に渡しており、入力の改行コード差分を事前に正規化していなかった
- 描画側も `\n` を中心に扱っていたため、`CRLF` や `\r` を含む入力で改行が見えにくくなる余地があった

## Fix
- Markdown 入力を `\r\n` / `\r` から `\n` に正規化してから解析するようにした
- レンダラー側でも `\r` を改行トークンとして扱うようにした

## Regression Prevention
- Windows 改行の hard break を検証する unit test を追加した
- 既存の paragraph soft break テストと合わせて、改行の扱いを継続監視する
