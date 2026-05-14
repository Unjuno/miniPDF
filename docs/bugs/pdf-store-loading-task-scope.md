# PDF読み込み時のloadingTaskスコープ問題修正

## 症状
- `loadingTask is not defined`エラーが発生する
- PDF読み込み失敗時にリソースが適切に解放されない

## 根本原因
`loadPdf`関数で、`loadingTask`と`pdf`変数が`invoke`の後に宣言されていたため、`invoke`が失敗した場合、外側の`catch`ブロックからアクセスできませんでした。

## 修正内容
`loadingTask`と`pdf`変数を`invoke`の前に宣言することで、外側の`catch`ブロックからもアクセスできるようにしました。

## 修正ファイル
- `src/stores/pdfStore.ts`

## 回帰防止
- 変数のスコープを適切に設定することで、エラー時に確実にリソースを解放できるようになりました
- テストで`mockPdf`に`destroy`メソッドを追加することで、警告を解消しました

