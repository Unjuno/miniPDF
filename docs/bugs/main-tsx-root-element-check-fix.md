# main.tsxのroot要素チェック追加

## 症状
- `document.getElementById('root')!`を使用しており、root要素が存在しない場合にエラーが発生する可能性がある
- エラーメッセージが不明確で、デバッグが困難

## 根本原因
`src/main.tsx`で、`document.getElementById('root')!`を使用していました。これにより、root要素が存在しない場合、非nullアサーション演算子により実行時エラーが発生する可能性がありました。

## 修正内容
1. `document.getElementById('root')`の結果を変数に保存
2. root要素が存在しない場合、明確なエラーメッセージをスロー
3. root要素が存在する場合のみ、`ReactDOM.createRoot`を呼び出し

## 修正ファイル
- `src/main.tsx`

## 回帰防止
- root要素の存在確認により、より明確なエラーメッセージを提供できるようになりました
- 実行時エラーを防ぎ、デバッグを容易にしました

