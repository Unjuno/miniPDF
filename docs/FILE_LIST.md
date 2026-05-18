# miniPDF - プロジェクトファイルリスト

この文書は、miniPDFプロジェクトの全ファイル構成を整理したものです。

## ディレクトリ構造

```
miniPDF/
├── docs/                    # ドキュメント
│   ├── bugs/               # バグノート
│   ├── *.md                # 各種ドキュメント
├── src/                     # フロントエンド（React + TypeScript）
│   ├── components/         # Reactコンポーネント
│   ├── hooks/              # カスタムフック
│   ├── stores/             # Zustandストア
│   ├── types/              # TypeScript型定義
│   ├── utils/              # ユーティリティ関数
│   └── locales/           # 国際化リソース
├── src-tauri/              # バックエンド（Rust + Tauri）
│   ├── src/
│   │   ├── commands/       # Tauriコマンド
│   │   ├── models/         # データモデル
│   │   ├── utils/         # ユーティリティ
│   │   └── services/      # サービス層（現在は空）
│   ├── tests/              # 統合テスト
│   └── permissions/        # Tauri権限設定
├── dist/                    # ビルド出力
├── test_data/              # テストデータ（現在は空）
├── tests/                   # テストディレクトリ（現在は空）
├── .playwright-mcp/         # Playwright MCP関連ファイル（開発用）
└── [設定ファイル]          # 各種設定ファイル
```

## ドキュメントファイル

### ルートドキュメント

- `README.md` - プロジェクトの基本情報・エントリーポイント
- `AGENTS.md` - エージェント開発ルール
- `LICENSE` - MITライセンス

### docs/ ディレクトリ

#### 基本ドキュメント

- `INDEX.md` - ドキュメント索引（ドキュメントの読み方）
- `CONCEPT.md` - プロジェクトのコンセプト・設計思想
- `SPECIFICATION.md` - 機能仕様・実装方法（実装時の主要参照文書）
- `FEATURES.md` - 機能説明書（ユーザー向け）⭐
- `markdown-preview-cli.md` - Markdown を PDF に出力する CLI
- `DO_NOT.md` - やらないこと制約リスト

#### 実装・検証ドキュメント

- `IMPLEMENTATION_PLAN.md` - 実装計画（AI開発支援用）
- `IMPLEMENTATION_VERIFICATION.md` - 実装検証レポート
- `ACTUAL_IMPLEMENTATION_STATUS.md` - コードベース検証メモ

#### 技術ドキュメント

- `PDF_LIBRARY_RESEARCH.md` - PDFライブラリ調査
- `OXIDIZE_PDF_MIGRATION_PLAN.md` - PDFライブラリ移行計画
- `CARGO_PROFILE_ANALYSIS.md` - Cargoプロファイル分析

#### バグノート（docs/bugs/）

- `page-delete-render-mismatch.md` - ページ削除時のレンダリング不一致
- `page-reorder-dnd.md` - ページ並び替えのドラッグアンドドロップ
- `preview-blank-page-render.md` - プレビューの空白ページレンダリング
- `preview-reorder-not-updating.md` - プレビューの並び替え更新
- `20260519-thematic-break-style-variants.md` - 水平線の記法ごとの差分レンダリング
- `readme-missing-links.md` - READMEのリンク不足
- `zoom-minus-button.md` - ズームマイナスボタン

## ソースコードファイル

### フロントエンド（src/）

#### コンポーネント（src/components/）

- `App.tsx` - メインアプリケーションコンポーネント
- `PDFViewer.tsx` - PDF表示コンポーネント
- `PDFViewer.clear-state.test.tsx` - PDFViewerのクリア状態テスト
- `ImageResizer.tsx` - 画像リサイズコンポーネント
- `ImageElement.tsx` - 画像要素コンポーネント
- `ImageOverlay.tsx` - 画像オーバーレイコンポーネント
- `ImageInserter.tsx` - 画像挿入コンポーネント
- `TextEditor.tsx` - テキストエディタコンポーネント
- `InlineTextEditor.tsx` - インラインテキストエディタコンポーネント
- `TextBlockOverlay.tsx` - テキストブロックオーバーレイ（編集モード用、`previewOnly` 時は非表示）
- `PageBreakEditor.tsx` - 改ページエディタコンポーネント
- `ErrorDisplay.tsx` - エラー表示コンポーネント
- `ErrorDisplay.test.tsx` - ErrorDisplayのテスト
- `Toast.tsx` - トースト通知コンポーネント
- `KeyboardShortcutsHelp.tsx` - キーボードショートカットヘルプコンポーネント
- `*.css` - 各コンポーネントのスタイルシート

#### フック（src/hooks/）

- `useDebounce.ts` - デバウンスフック
- `useDragMove.ts` - ドラッグ移動フック
- `useDragResize.ts` - ドラッグリサイズフック
- `useKeyboardShortcuts.ts` - キーボードショートカットフック
- `useRenderCache.ts` - レンダリングキャッシュフック
- `useToast.ts` - トースト通知フック

#### ストア（src/stores/）

- `pdfStore.ts` - PDF編集状態管理ストア（Zustand）
- `pdfStore.test.ts` - pdfStoreのテスト

#### 型定義（src/types/）

- `pdf.ts` - PDF関連のTypeScript型定義

#### ユーティリティ（src/utils/）

- `logger.ts` - ロガーユーティリティ
- `pageMapping.ts` - ページマッピングユーティリティ
- `pageMapping.test.ts` - pageMappingのテスト
- `renderBlankPage.ts` - 空白ページレンダリングユーティリティ
- `renderBlankPage.test.ts` - renderBlankPageのテスト
- `pdfImageExtractor.ts` - PDF画像抽出ユーティリティ
- `pdfTextExtractor.ts` - PDFテキスト抽出ユーティリティ

#### その他（src/）

- `main.tsx` - エントリーポイント
- `App.css` - アプリケーションスタイル
- `index.css` - グローバルスタイル
- `vite-env.d.ts` - Vite環境型定義
- `locales/ja.json` - 日本語リソース
- `locales/en.json` - 英語リソース

### バックエンド（src-tauri/）

#### コマンド（src-tauri/src/commands/）

- `mod.rs` - コマンドモジュール
- `file_dialog.rs` - ファイルダイアログコマンド
- `pdf_loader.rs` - PDF読み込みコマンド
- `file_saver.rs` - ファイル保存コマンド
- `pdf_generator.rs` - PDF生成コマンド
- `image_resizer.rs` - 画像リサイズコマンド
- `image_inserter.rs` - 画像挿入コマンド
- `image_mover.rs` - 画像移動コマンド
- `text_editor.rs` - テキスト編集コマンド
- `text_mover.rs` - テキスト移動コマンド
- `page_editor.rs` - ページ編集コマンド
- `page_break.rs` - 改ページ調整コマンド

#### モデル（src-tauri/src/models/）

- `mod.rs` - モデルモジュール
- `pdf_structure.rs` - PDF構造データモデル

#### ユーティリティ（src-tauri/src/utils/）

- `mod.rs` - ユーティリティモジュール
- `logger.rs` - ロガーユーティリティ

#### その他（src-tauri/src/）

- `main.rs` - メインエントリーポイント
- `lib.rs` - ライブラリエントリーポイント
- `build.rs` - ビルドスクリプト

#### サービス層（src-tauri/src/services/）

- （現在は空のディレクトリ、将来の拡張用）

#### テスト（src-tauri/tests/）

- `integration/mod.rs` - 統合テストモジュール
- `integration/ipc_test.rs` - IPC通信統合テスト

#### 設定（src-tauri/）

- `Cargo.toml` - Rust依存関係設定
- `Cargo.lock` - 依存関係ロックファイル
- `tauri.conf.json` - Tauri設定ファイル
- `permissions/custom-commands.toml` - カスタムコマンド権限
- `permissions/fs-default.toml` - ファイルシステム権限
- `capabilities/main-capability.json` - メイン機能設定

## 設定ファイル

### ルート設定ファイル

- `package.json` - Node.js依存関係とスクリプト
- `package-lock.json` - npm依存関係ロックファイル（Git管理対象外）
- `tsconfig.json` - TypeScript設定（フロントエンド用）
- `tsconfig.node.json` - TypeScript設定（Node.js用）
- `vite.config.ts` - Vite設定
- `vitest.config.ts` - Vitest設定
- `.gitignore` - Git除外設定

### その他のファイル

- `index.html` - HTMLエントリーポイント
- `mian.pdf` - テスト用PDFファイル（typoあり、実際のテストで使用されている）

### 開発用ファイル

- `.playwright-mcp/` - Playwright MCP関連ファイル（UIテスト用スナップショット等）

## ビルド出力

### dist/ ディレクトリ

- `index.html` - ビルド済みHTML
- `assets/` - ビルド済みアセット（JS、CSS等）

### src-tauri/target/ ディレクトリ

- `debug/` - デバッグビルド出力
- `release/` - リリースビルド出力

## テストデータ・ディレクトリ

### test_data/ ディレクトリ

テスト用のPDFファイル等を配置するディレクトリ（現在は空）

### tests/ ディレクトリ

統合テスト用ディレクトリ（現在は空、将来の拡張用）

### src/test/ ディレクトリ

フロントエンドテスト用ディレクトリ（現在は空、将来の拡張用）

## ファイル統計

### ドキュメント

- Markdownファイル: 25ファイル
  - ルート: 2ファイル（README.md, AGENTS.md）
  - docs/: 23ファイル（INDEX.md, CONCEPT.md, SPECIFICATION.md, FEATURES.md, DO_NOT.md, IMPLEMENTATION_PLAN.md, IMPLEMENTATION_VERIFICATION.md, ACTUAL_IMPLEMENTATION_STATUS.md, PDF_LIBRARY_RESEARCH.md, OXIDIZE_PDF_MIGRATION_PLAN.md, CARGO_PROFILE_ANALYSIS.md, FONT_MANAGEMENT.md, markdown-preview-cli.md, FILE_LIST.md, COMPLETE_FILE_LIST.md, RELEASING.md, バグノート6ファイル）
  - .playwright-mcp/: 1ファイル（ui-test-snapshot.md）
  - src-tauri/fonts/: 1ファイル（README.md）
- バグノート: 7ファイル（docs/bugs/）

### ソースコード

- TypeScript/TSXファイル: 36ファイル
  - コンポーネント: 16ファイル（.tsx, .ts）- `src/components/`内（テストファイル含む）
  - フック: 6ファイル（.ts）
  - ストア: 2ファイル（.ts）
  - ユーティリティ: 7ファイル（.ts）
  - 型定義: 1ファイル（.ts）
  - その他: 4ファイル（.tsx, .ts）- `App.tsx`, `main.tsx`, `vite-env.d.ts`
- Rustファイル: 21ファイル
  - コマンド: 12ファイル（.rs）
  - モデル: 2ファイル（.rs）
  - ユーティリティ: 2ファイル（.rs）
  - テスト: 2ファイル（.rs）
  - その他: 3ファイル（.rs）
- CSSファイル: 13ファイル（各コンポーネント用 + App.css, index.css）

### 設定ファイル

- JSONファイル: 12ファイル
- TOMLファイル: 3ファイル
- TypeScript設定: 2ファイル
- その他: `.gitignore`, `LICENSE`, `index.html`等

## 参照

- [COMPLETE_FILE_LIST.md](./COMPLETE_FILE_LIST.md) - 完全ファイルリスト（全ファイルの詳細リスト）
- [INDEX.md](./INDEX.md) - ドキュメント索引
- [README.md](../README.md) - プロジェクト基本情報
