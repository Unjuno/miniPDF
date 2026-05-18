# Bug Note: Mermaid PNG をそのまま埋めると Acrobat で警告が出ることがあった

## 症状

Mermaid を含む Markdown から出力した PDF を Adobe Acrobat で開くと、`画像のデータに不足があります` のような警告が出ることがあった。

## Root Cause

Mermaid CLI が出力する PNG は透過を含むことがあり、PDF 埋め込み時にそのまま使うと Acrobat 側で扱いが不安定になるケースがあった。

## Fix

- Mermaid PNG をいったんデコードし、白背景に合成したうえで RGB PNG として再エンコードしてから PDF に埋め込むようにした。
- これにより、透過由来の soft mask を避け、PDF の互換性を上げた。

## Regression Prevention

- 透明 PNG を正規化すると alpha を失うことを unit test で確認する。
- Mermaid を含む PDF 生成後に Acrobat 互換の確認を手動で行う。
