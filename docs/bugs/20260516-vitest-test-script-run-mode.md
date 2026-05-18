# `npm test` が watch モードのまま終了しない問題

## 症状

`npm test` を実行すると、Vitest が一回実行で終了せず、対話的な watch 挙動になって終了しないことがあった。

## 根本原因

`package.json` の `test` スクリプトが `vitest` だけになっており、実行モードが固定されていなかった。

## 修正

`package.json` の `test` スクリプトを `vitest --run` に変更し、常に一回実行で終了するようにした。

## 再発防止

`src/packageScripts.test.ts` で `scripts.test` が `vitest --run` であることを検証する。
