# リポジトリ整理: シム一覧（削除順チェックリスト）

`export * from` による再エクスポートのみのファイル。正本へ統合後に削除する。

## コンポーネントシム

- [x] `src/components/*.tsx` のシムは撤去済み。実体はすべて `src/components/` に集約。

## フックシム

- [x] `src/utils/hooks/*.ts` を削除。参照は `src/hooks/` へ統一。

## Store / 型シム

- [x] `src/utils/stores/pdfStore.ts` 削除
- [x] `src/utils/types/pdf.ts` 削除

## ユーティリティ二重パス

- [x] `src/utils/utils/*` 削除。コンポーネントからは `../utils/logger` 等で `src/utils/*.ts` を直接参照。

## 参照の最終形

- `src/components/*` は `../stores/`、`../hooks/`、`../types/`、`../utils/*` を直接 import。
- `src/utils/` はロガー・マッピング・PDF 抽出・`tauriRuntime` など非 UI のみ。
