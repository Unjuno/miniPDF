# miniPDF

> Markdown を書きながら PDF をライブプレビューし、必要なら PDF 保存する軽量デスクトップアプリ

[![GitHub](https://img.shields.io/github/license/Unjuno/miniPDF)](https://github.com/Unjuno/miniPDF)
[![GitHub](https://img.shields.io/github/stars/Unjuno/miniPDF)](https://github.com/Unjuno/miniPDF)

## 概要

miniPDF は、Markdown を編集しながら **PDF のライブプレビュー**を確認し、Markdown や生成 PDF を保存できる軽量デスクトップアプリケーションです。PDF 上でのレイアウト編集 UI は持ちません。

### ドキュメントの読む順（短縮）

1. この README（概要・セットアップ）
2. [docs/FEATURES.md](./docs/FEATURES.md)（操作）
3. [docs/SPECIFICATION.md](./docs/SPECIFICATION.md)（仕様・IPC）
4. [docs/CURRENT_ISSUES_SUMMARY.md](./docs/CURRENT_ISSUES_SUMMARY.md)（既知の問題の正本）
5. [docs/INDEX.md](./docs/INDEX.md)（全体索引）
6. [docs/RELEASING.md](./docs/RELEASING.md)（CI・インストーラー配布）

### 解決する問題

- Markdown + Mermaid を PDF にしたときの**見た目をアプリ内で素早く確認**したい
- ソース編集とプレビューを**同じ画面で**切り替えたくない

### 特徴

- **軽量**：起動が速い（1秒以内）を目安
- **プレビュー中心**：PDF 上のドラッグ編集は行わない
- **OSS**：完全オープンソース

## 機能

### ファイル操作
- Markdownファイルの読み込み・保存
- 生成済みPDFの保存
- （補助）PDFファイルの読み込み

### Markdownプレビュー
- Markdown入力のライブPDFプレビュー（デバウンス更新）
- Mermaidコードブロックのプレビュー反映（`mmdc` 利用時）
- Mermaid描画失敗時のコードブロックフォールバック
- 連続表示 / 1ページ表示、ズーム、ページ送り

### PDF の保存
- プレビューで表示中の PDF をファイルに保存

### 表示機能
- PDF.js によるページ描画（閲覧専用プレビュー）
- ズーム機能（50% - 200%）
- ページナビゲーション

## 技術スタック

- [Tauri](https://tauri.app/) - 軽量デスクトップアプリケーションフレームワーク
- React + TypeScript
- PDF.js - PDF表示
- PDF 生成・プレビュー（Rust）— [oxidize-pdf](https://crates.io/crates/oxidize-pdf) 等

## 開発状況

✅ **コア機能** — Markdown ライブプレビュー・PDF 保存・表示操作は利用可能です。

## GitHub Actions（CI / リリースビルド）

- **CI**（`.github/workflows/ci.yml`）: `main` / `master` への push と PR で `vitest` と `cargo test --lib` を実行します。
- **Release**（`.github/workflows/release.yml`）: 手動実行で Windows / macOS / Linux の **Tauri バンドル**をビルドし Artifact に保存。`v*` タグを push したときは **ドラフトの GitHub Release** にインストーラー類を添付します。

手順・バージョン合わせ・アイコン差し替えは [docs/RELEASING.md](./docs/RELEASING.md) を参照してください。

### 必要な環境

- Node.js 18+
- Rust (最新安定版)
- Tauri CLI
- Mermaid CLI（`mmdc`）※図を画像化する場合。`MINIPDF_MERMAID_CLI`、リポジトリの `node_modules/.bin`（`npm install` 後）、実行ファイル同階、または PATH のいずれかで解決可能（Win / Mac / Linux）

### インストール

```bash
# 依存関係のインストール
npm install

# 開発（Tauri + Vite。単独の npm run dev とポート 5173 が競合するため同時起動しない）
npm run tauri:dev

# ビルド
npm run tauri:build
```

フロントのみをブラウザで試す場合は `npm run dev` を使います。別ターミナルで `npm run tauri:dev` を動かすと Vite のポートが衝突し、`tauri dev` の `beforeDevCommand` が失敗することがあります。

### Mermaidプレビューに関する補足

Mermaid ブロックを画像化してプレビューするには **Mermaid CLI（`mmdc`）** が必要です。次のいずれかで解決します（**Windows / macOS / Linux 共通**）。

1. **環境変数 `MINIPDF_MERMAID_CLI`** — 配布パッケージに同梱した `mmdc` の**絶対パス**（PATH に入れなくてよい）
2. **このリポジトリで `npm install` 済み** — `@mermaid-js/mermaid-cli` が devDependency として入り、アプリはリポジトリ直下の `node_modules/.bin/mmdc` を自動で試します（初回は Chromium 取得などで時間がかかることがあります）
3. **実行ファイルと同じディレクトリ**に `mmdc`（Windows は `mmdc.exe`）または `bin/mmdc` を置く（インストーラが並置する想定）
4. 従来どおり **`mmdc` を PATH に追加**する

`mmdc` が起動できない場合、Mermaid ブロックはコードブロックとして表示されます。  
※ `@mermaid-js/mermaid-cli` は内部で Chromium 等を使うため、**完全オフライン同梱はサイズが大きくなりがち**です。リリースごとに OS 別バイナリをビルド成果物へコピーする運用が現実的です。

## 変更履歴

変更履歴は専用ファイルを用意していません。履歴は Git のコミット履歴を参照してください。

## テスト

```bash
# フロントエンドテスト
npm run test

# バックエンドテスト（Rust）
cd src-tauri
cargo test
```

詳細は以下を参照してください。
- [docs/IMPLEMENTATION_VERIFICATION.md](./docs/IMPLEMENTATION_VERIFICATION.md) - 実装検証レポート
- [docs/ACTUAL_IMPLEMENTATION_STATUS.md](./docs/ACTUAL_IMPLEMENTATION_STATUS.md) - コードベース検証メモ

## 機能テスト

機能確認の観点は以下を参照してください：
- [docs/FEATURES.md](./docs/FEATURES.md) - 機能の詳細と操作方法

手動テストの手順書と結果テンプレートは現時点で未整備です。

## ライセンス

AGPL-3.0 License - 詳細は [LICENSE](./LICENSE) を参照してください。

## コントリビューション

コントリビューションを歓迎します！IssueやPull Requestをお気軽に作成してください。

- [GitHub Issues](https://github.com/Unjuno/miniPDF/issues) - バグ報告・機能要望
- [GitHub Pull Requests](https://github.com/Unjuno/miniPDF/pulls) - コード貢献

## ドキュメント

プロジェクトの詳細情報は以下の文書を参照してください：

### 基本ドキュメント

- **[docs/FEATURES.md](./docs/FEATURES.md)** - **機能説明書（ユーザー向け）** ⭐ - 機能の詳細と操作方法
- **[docs/CONCEPT.md](./docs/CONCEPT.md)** - プロジェクトのコンセプト・設計思想
- **[docs/SPECIFICATION.md](./docs/SPECIFICATION.md)** - 機能仕様・実装方法（実装時の主要参照文書）
- **[docs/DO_NOT.md](./docs/DO_NOT.md)** - やらないこと制約リスト（プロジェクトのスコープと制約事項）

### ドキュメント索引

- **[docs/INDEX.md](./docs/INDEX.md)** - ドキュメント索引（ドキュメントの読み方）
- **[docs/FILE_LIST.md](./docs/FILE_LIST.md)** - プロジェクトファイルリスト（プロジェクト構造の全体像）
- **[docs/COMPLETE_FILE_LIST.md](./docs/COMPLETE_FILE_LIST.md)** - 完全ファイルリスト（全175ファイルの詳細リスト）
