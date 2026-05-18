# Bug Note: Mermaid PDF 埋め込みで soft mask が残る可能性を潰した

## 症状

Mermaid を含む PDF を Adobe Acrobat で開くと、`画像のデータに不足があります` の警告が出ることがあった。

## Root Cause

Mermaid CLI の PNG をそのまま PDF に入れると、透過や soft mask を伴う画像になりやすく、PDF リーダーによっては互換性問題を起こす可能性があった。

## Fix

- Mermaid PNG を白背景に合成して RGB 化したうえで、PDF へは JPEG として埋め込むようにした。
- これにより、`SMask` を持たない保守的な画像表現に寄せた。

## Regression Prevention

- Mermaid の PDF バイナリに `/SMask` が出ないことを unit test で確認する。
- Mermaid 埋め込み画像が JPEG になっていることを unit test で確認する。
- 既存の Mermaid 画像正規化テストと合わせて、透明由来の崩れを検出する。
