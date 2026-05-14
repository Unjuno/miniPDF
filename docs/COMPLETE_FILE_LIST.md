# miniPDF - 完全ファイルリスト

この文書は、miniPDFプロジェクトのすべてのファイルを網羅的にリストアップしたものです。

**最終更新**: 2026年（`PageEditor` 撤去・プレビュー専用 UI 反映）
**総ファイル数**: 175ファイル（node_modules、target、dist、.gitを除く）

## ファイル統計

- **Markdownファイル**: 24ファイル
- **TypeScript/TSXファイル**: 36ファイル
- **Rustファイル**: 21ファイル
- **CSSファイル**: 13ファイル
- **JSONファイル**: 12ファイル
- **TOMLファイル**: 3ファイル
- **その他**: 66ファイル（画像、設定ファイル等）

## 1. ルートディレクトリファイル

### ドキュメント
- `README.md` - プロジェクトの基本情報・エントリーポイント
- `AGENTS.md` - エージェント開発ルール
- `LICENSE` - AGPL-3.0ライセンス

### 設定ファイル
- `.gitignore` - Git除外設定
- `package.json` - Node.js依存関係とスクリプト
- `package-lock.json` - npm依存関係ロックファイル（Git管理対象外）
- `tsconfig.json` - TypeScript設定（フロントエンド用）
- `tsconfig.node.json` - TypeScript設定（Node.js用）
- `vite.config.ts` - Vite設定
- `vitest.config.ts` - Vitest設定

### その他
- `index.html` - HTMLエントリーポイント
- `image.png` - プロジェクト画像
- `mian.pdf` - テスト用PDFファイル（typoあり、実際のテストで使用）

## 2. docs/ ディレクトリ

### 基本ドキュメント（5ファイル）
- `INDEX.md` - ドキュメント索引
- `CONCEPT.md` - プロジェクトのコンセプト・設計思想
- `SPECIFICATION.md` - 機能仕様・実装方法
- `FEATURES.md` - 機能説明書（ユーザー向け）⭐
- `DO_NOT.md` - やらないこと制約リスト

### 実装・検証ドキュメント（3ファイル）
- `IMPLEMENTATION_PLAN.md` - 実装計画（AI開発支援用）
- `IMPLEMENTATION_VERIFICATION.md` - 実装検証レポート
- `ACTUAL_IMPLEMENTATION_STATUS.md` - コードベース検証メモ

### 技術ドキュメント（4ファイル）
- `PDF_LIBRARY_RESEARCH.md` - PDFライブラリ調査
- `OXIDIZE_PDF_MIGRATION_PLAN.md` - PDFライブラリ移行計画
- `CARGO_PROFILE_ANALYSIS.md` - Cargoプロファイル分析
- `FONT_MANAGEMENT.md` - フォント管理機能の説明

### ファイルリスト（2ファイル）
- `FILE_LIST.md` - プロジェクトファイルリスト（構造説明）
- `COMPLETE_FILE_LIST.md` - 完全ファイルリスト（このファイル）

### バグノート（docs/bugs/ - 6ファイル）
- `page-delete-render-mismatch.md` - ページ削除時のレンダリング不一致
- `page-reorder-dnd.md` - ページ並び替えのドラッグアンドドロップ
- `preview-blank-page-render.md` - プレビューの空白ページレンダリング
- `preview-reorder-not-updating.md` - プレビューの並び替え更新
- `readme-missing-links.md` - READMEのリンク不足
- `zoom-minus-button.md` - ズームマイナスボタン

## 3. src/ ディレクトリ（フロントエンド）

### コンポーネント（src/components/ - 26ファイル）
- **TSX/TSファイル（16ファイル）**:
  - `PDFViewer.tsx` - PDF表示コンポーネント
  - `PDFViewer.clear-state.test.tsx` - PDFViewerのクリア状態テスト
  - `ImageResizer.tsx` - 画像リサイズコンポーネント
  - `ImageElement.tsx` - 画像要素コンポーネント
  - `ImageOverlay.tsx` - 画像オーバーレイコンポーネント
  - `ImageInserter.tsx` - 画像挿入コンポーネント
  - `TextEditor.tsx` - テキストエディタコンポーネント
  - `InlineTextEditor.tsx` - インラインテキストエディタコンポーネント
  - `TextBlockOverlay.tsx` - テキストブロックオーバーレイコンポーネント（`PDFViewer` の `previewOnly={false}` 時のみ使用）
  - `PageBreakEditor.tsx` - 改ページエディタコンポーネント
  - `ErrorDisplay.tsx` - エラー表示コンポーネント
  - `ErrorDisplay.test.tsx` - ErrorDisplayのテスト
  - `Toast.tsx` - トースト通知コンポーネント
  - `KeyboardShortcutsHelp.tsx` - キーボードショートカットヘルプコンポーネント
- **CSSファイル（10ファイル）**:
  - `ErrorDisplay.css`, `ImageElement.css`, `ImageInserter.css`, `ImageResizer.css`
  - `InlineTextEditor.css`, `PageBreakEditor.css`
  - `PDFViewer.css`, `TextEditor.css`, `Toast.css`, `KeyboardShortcutsHelp.css`
- **注**: `App.tsx`と`App.css`は`src/`ディレクトリにあります

### フック（src/hooks/ - 6ファイル）
- `useDebounce.ts` - デバウンスフック
- `useDragMove.ts` - ドラッグ移動フック
- `useDragResize.ts` - ドラッグリサイズフック
- `useKeyboardShortcuts.ts` - キーボードショートカットフック
- `useRenderCache.ts` - レンダリングキャッシュフック
- `useToast.ts` - トースト通知フック

### ストア（src/stores/ - 2ファイル）
- `pdfStore.ts` - PDF編集状態管理ストア（Zustand）
- `pdfStore.test.ts` - pdfStoreのテスト

### 型定義（src/types/ - 1ファイル）
- `pdf.ts` - PDF関連のTypeScript型定義

### ユーティリティ（src/utils/ - 7ファイル）
- `logger.ts` - ロガーユーティリティ
- `pageMapping.ts` - ページマッピングユーティリティ
- `pageMapping.test.ts` - pageMappingのテスト
- `renderBlankPage.ts` - 空白ページレンダリングユーティリティ
- `renderBlankPage.test.ts` - renderBlankPageのテスト
- `pdfImageExtractor.ts` - PDF画像抽出ユーティリティ
- `pdfTextExtractor.ts` - PDFテキスト抽出ユーティリティ

### その他（src/ - 4ファイル）
- `main.tsx` - エントリーポイント
- `App.css` - アプリケーションスタイル
- `index.css` - グローバルスタイル
- `vite-env.d.ts` - Vite環境型定義

### 国際化（src/locales/ - 2ファイル）
- `ja.json` - 日本語リソース
- `en.json` - 英語リソース

## 4. src-tauri/ ディレクトリ（バックエンド）

### コマンド（src-tauri/src/commands/ - 12ファイル）
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

### モデル（src-tauri/src/models/ - 2ファイル）
- `mod.rs` - モデルモジュール
- `pdf_structure.rs` - PDF構造データモデル

### ユーティリティ（src-tauri/src/utils/ - 3ファイル）
- `mod.rs` - ユーティリティモジュール
- `logger.rs` - ロガーユーティリティ
- `font_manager.rs` - フォント管理ユーティリティ

### サービス層（src-tauri/src/services/）
- （現在は空のディレクトリ、将来の拡張用）

### その他（src-tauri/src/ - 3ファイル）
- `main.rs` - メインエントリーポイント
- `lib.rs` - ライブラリエントリーポイント
- `build.rs` - ビルドスクリプト

### フォント（src-tauri/fonts/ - 1ファイル）
- `README.md` - フォントディレクトリの使用方法

### テスト（src-tauri/tests/integration/ - 2ファイル）
- `mod.rs` - 統合テストモジュール
- `ipc_test.rs` - IPC通信統合テスト

### 設定（src-tauri/ - 6ファイル）
- `Cargo.toml` - Rust依存関係設定
- `Cargo.lock` - 依存関係ロックファイル（Git管理対象外）
- `tauri.conf.json` - Tauri設定ファイル
- `permissions/custom-commands.toml` - カスタムコマンド権限
- `permissions/fs-default.toml` - ファイルシステム権限
- `capabilities/main-capability.json` - メイン機能設定

### 生成ファイル（src-tauri/gen/schemas/ - 4ファイル）
- `acl-manifests.json` - ACLマニフェスト（自動生成）
- `capabilities.json` - 機能設定（自動生成）
- `desktop-schema.json` - デスクトップスキーマ（自動生成）
- `windows-schema.json` - Windowsスキーマ（自動生成）

### アイコン（src-tauri/icons/ - 52ファイル）
- 各種サイズのアイコンファイル（PNG、ICO、ICNS等）
- Android用アイコン（mipmap-*）
- iOS用アイコン（AppIcon-*）
- ストア用ロゴ（Square*、StoreLogo）

## 5. その他のディレクトリ

### .playwright-mcp/（開発用 - 8ファイル）
- UIテスト用スナップショット画像ファイル（7ファイル）
- `ui-test-snapshot.md` - UIテストスナップショットドキュメント

### test_data/（テストデータ）
- （現在は空のディレクトリ）

### tests/（テスト）
- （現在は空のディレクトリ、将来の拡張用）

### src/test/（フロントエンドテスト）
- （現在は空のディレクトリ、将来の拡張用）

## 6. ビルド出力（Git管理対象外）

### dist/
- ビルド済みのフロントエンドファイル

### src-tauri/target/
- Rustビルド出力（debug/、release/）

## ファイルカテゴリ別統計

### ソースコード
- **TypeScript/TSX**: 36ファイル
  - コンポーネント: 16ファイル（.tsx, .ts）- `src/components/`内（テストファイル含む）
  - フック: 6ファイル（.ts）
  - ストア: 2ファイル（.ts）
  - ユーティリティ: 7ファイル（.ts）
  - 型定義: 1ファイル（.ts）
  - その他: 4ファイル（.tsx, .ts）- `App.tsx`, `main.tsx`, `vite-env.d.ts`（合計36ファイル）
- **Rust**: 22ファイル
  - コマンド: 12ファイル（.rs）
  - モデル: 2ファイル（.rs）
  - ユーティリティ: 3ファイル（.rs）
  - テスト: 2ファイル（.rs）
  - その他: 3ファイル（.rs）
- **CSS**: 13ファイル

### ドキュメント
- **Markdown**: 22ファイル
  - ルート: 2ファイル（README.md, AGENTS.md）
  - docs/: 19ファイル（INDEX.md, CONCEPT.md, SPECIFICATION.md, FEATURES.md, DO_NOT.md, IMPLEMENTATION_PLAN.md, IMPLEMENTATION_VERIFICATION.md, ACTUAL_IMPLEMENTATION_STATUS.md, PDF_LIBRARY_RESEARCH.md, OXIDIZE_PDF_MIGRATION_PLAN.md, CARGO_PROFILE_ANALYSIS.md, FILE_LIST.md, COMPLETE_FILE_LIST.md, バグノート6ファイル）
  - .playwright-mcp/: 1ファイル（ui-test-snapshot.md）

### 設定ファイル
- **JSON**: 12ファイル
- **TOML**: 3ファイル
- **TypeScript設定**: 2ファイル

### その他
- **画像**: 52ファイル（アイコン等）
- **その他**: 約14ファイル（設定ファイル、HTML、PDF等）

## 参照

- [FILE_LIST.md](./FILE_LIST.md) - プロジェクトファイルリスト（構造説明）
- [INDEX.md](./INDEX.md) - ドキュメント索引
- [README.md](../README.md) - プロジェクト基本情報

