# PDF保存時のバリデーション強化

## 症状
- 保存するとちゃんと表示されません
- 無効なファイルパスで保存を試みた場合のエラーハンドリングが不十分

## 根本原因
`savePdf`関数で、`filePath`のバリデーションと`pdfData`の空チェックが不足していました。また、配列操作で`page.images`や`page.textBlocks`がundefinedの場合のチェックが不足していました。

## 修正内容
1. `savePdf`関数:
   - `filePath`のバリデーションを追加（空文字列、null、undefinedのチェック）
   - `pdfData`が空の場合のチェックを追加

2. 配列操作の安全性向上:
   - `page.images.map`で`(page.images || [])`を使用
   - `page.textBlocks`で`(page.textBlocks || [])`を使用

## 修正ファイル
- `src/stores/pdfStore.ts`

## 回帰防止
- バリデーションにより、無効な値のリクエストを即座に検出できるようになりました
- 安全な配列アクセスにより、undefinedエラーを防ぎました
- エラーメッセージを統一することで、ユーザーが問題を理解しやすくなりました

