# miniPDF - 技術リファレンス（一次情報）

この文書は、miniPDF の実装時に参照すべき技術の一次情報（公式ドキュメント）のURLをまとめたものです。

## コアフレームワーク

### Tauri

- **公式サイト**: https://tauri.app/
- **公式ドキュメント**: https://tauri.app/v1/guides/
- **API リファレンス**: https://tauri.app/v1/api/
- **GitHub**: https://github.com/tauri-apps/tauri
- **Tauri 2.x ドキュメント**（最新版）: https://v2.tauri.app/
- **セットアップガイド**: https://tauri.app/v1/guides/getting-started/prerequisites
- **IPC通信**: https://tauri.app/v1/guides/features/command
- **ファイルシステムAPI**: https://tauri.app/v1/api/js/fs/

### React

- **公式サイト**: https://react.dev/
- **公式ドキュメント**: https://react.dev/learn
- **API リファレンス**: https://react.dev/reference/react
- **GitHub**: https://github.com/facebook/react
- **Hooks API**: https://react.dev/reference/react

### TypeScript

- **公式サイト**: https://www.typescriptlang.org/
- **公式ドキュメント**: https://www.typescriptlang.org/docs/
- **ハンドブック**: https://www.typescriptlang.org/docs/handbook/intro.html
- **GitHub**: https://github.com/microsoft/TypeScript
- **型定義**: https://www.typescriptlang.org/docs/handbook/declaration-files/introduction.html

### Vite

- **公式サイト**: https://vitejs.dev/
- **公式ドキュメント**: https://vitejs.dev/guide/
- **API リファレンス**: https://vitejs.dev/config/
- **GitHub**: https://github.com/vitejs/vite
- **プラグイン**: https://vitejs.dev/plugins/

## Rust関連

### Rust言語

- **公式サイト**: https://www.rust-lang.org/
- **公式ドキュメント**: https://doc.rust-lang.org/
- **The Rust Book**: https://doc.rust-lang.org/book/
- **標準ライブラリ**: https://doc.rust-lang.org/std/
- **Cargo**: https://doc.rust-lang.org/cargo/

### Rust開発ツール

- **rustfmt**: https://github.com/rust-lang/rustfmt
- **clippy**: https://github.com/rust-lang/rust-clippy
- **rust-analyzer**: https://rust-analyzer.github.io/

## PDF処理ライブラリ

### Rust PDFライブラリ

#### pdf crate

- **crates.io**: https://crates.io/crates/pdf
- **ドキュメント**: https://docs.rs/pdf/
- **GitHub**: https://github.com/pdf-rs/pdf

#### lopdf

- **crates.io**: https://crates.io/crates/lopdf
- **ドキュメント**: https://docs.rs/lopdf/
- **GitHub**: https://github.com/J-F-Liu/lopdf

#### pdfium-render

- **crates.io**: https://crates.io/crates/pdfium-render
- **ドキュメント**: https://docs.rs/pdfium-render/
- **GitHub**: https://github.com/ajrcarey/pdfium-render

### JavaScript PDFライブラリ

#### PDF.js (Mozilla)

- **公式サイト**: https://mozilla.github.io/pdf.js/
- **ドキュメント**: https://mozilla.github.io/pdf.js/getting_started/
- **API リファレンス**: https://mozilla.github.io/pdf.js/api/
- **GitHub**: https://github.com/mozilla/pdf.js
- **ビューアー例**: https://mozilla.github.io/pdf.js/web/viewer.html

#### react-pdf

- **npm**: https://www.npmjs.com/package/react-pdf
- **ドキュメント**: https://react-pdf.org/
- **GitHub**: https://github.com/diegomura/react-pdf

## 画像処理

### Rust

#### image crate

- **crates.io**: https://crates.io/crates/image
- **ドキュメント**: https://docs.rs/image/
- **GitHub**: https://github.com/image-rs/image

### JavaScript

#### Canvas API

- **MDN**: https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API
- **Canvas 2D Context**: https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D

## UIライブラリ・コンポーネント

### ドラッグ&ドロップ

#### react-draggable

- **npm**: https://www.npmjs.com/package/react-draggable
- **GitHub**: https://github.com/react-grid-layout/react-draggable

#### @dnd-kit/core

- **公式サイト**: https://dndkit.com/
- **ドキュメント**: https://docs.dndkit.com/
- **GitHub**: https://github.com/clauderic/dnd-kit

### 状態管理

#### Zustand

- **公式サイト**: https://zustand-demo.pmnd.rs/
- **GitHub**: https://github.com/pmndrs/zustand
- **ドキュメント**: https://github.com/pmndrs/zustand#documentation

#### Jotai

- **公式サイト**: https://jotai.org/
- **ドキュメント**: https://jotai.org/docs/basics/getting-started
- **GitHub**: https://github.com/pmndrs/jotai

### スタイリング

#### Tailwind CSS

- **公式サイト**: https://tailwindcss.com/
- **ドキュメント**: https://tailwindcss.com/docs
- **GitHub**: https://github.com/tailwindlabs/tailwindcss

#### CSS Modules

- **GitHub**: https://github.com/css-modules/css-modules
- **Viteでの使用**: https://vitejs.dev/guide/features.html#css-modules

## ユーティリティ

### シリアライゼーション

#### serde (Rust)

- **公式サイト**: https://serde.rs/
- **ドキュメント**: https://docs.rs/serde/
- **crates.io**: https://crates.io/crates/serde
- **GitHub**: https://github.com/serde-rs/serde

### エラーハンドリング

#### anyhow (Rust)

- **crates.io**: https://crates.io/crates/anyhow
- **ドキュメント**: https://docs.rs/anyhow/
- **GitHub**: https://github.com/dtolnay/anyhow

### キーボードショートカット

#### react-hotkeys-hook

- **npm**: https://www.npmjs.com/package/react-hotkeys-hook
- **GitHub**: https://github.com/JohannesKlauss/react-hotkeys-hook

## テスト

### Frontend

#### React Testing Library

- **公式サイト**: https://testing-library.com/react
- **ドキュメント**: https://testing-library.com/docs/react-testing-library/intro/
- **GitHub**: https://github.com/testing-library/react-testing-library

#### Vitest

- **公式サイト**: https://vitest.dev/
- **ドキュメント**: https://vitest.dev/guide/
- **GitHub**: https://github.com/vitest-dev/vitest

### Backend (Rust)

#### Rust標準テスト

- **ドキュメント**: https://doc.rust-lang.org/book/ch11-00-testing.html
- **テストガイド**: https://doc.rust-lang.org/rust-by-example/testing.html

## 開発ツール

### リンター・フォーマッター

#### ESLint

- **公式サイト**: https://eslint.org/
- **ドキュメント**: https://eslint.org/docs/latest/
- **GitHub**: https://github.com/eslint/eslint

#### Prettier

- **公式サイト**: https://prettier.io/
- **ドキュメント**: https://prettier.io/docs/en/
- **GitHub**: https://github.com/prettier/prettier

#### rustfmt

- **ドキュメント**: https://github.com/rust-lang/rustfmt#readme
- **設定**: https://rust-lang.github.io/rustfmt/

#### clippy

- **ドキュメント**: https://github.com/rust-lang/rust-clippy#readme
- **ルール一覧**: https://rust-lang.github.io/rust-clippy/master/index.html

## パッケージマネージャー

### npm

- **公式サイト**: https://www.npmjs.com/
- **ドキュメント**: https://docs.npmjs.com/
- **CLI リファレンス**: https://docs.npmjs.com/cli/v9

### Cargo

- **ドキュメント**: https://doc.rust-lang.org/cargo/
- **リファレンス**: https://doc.rust-lang.org/cargo/reference/

## その他

### Web標準

#### File API

- **MDN**: https://developer.mozilla.org/en-US/docs/Web/API/File
- **FileReader API**: https://developer.mozilla.org/en-US/docs/Web/API/FileReader

#### Drag and Drop API

- **MDN**: https://developer.mozilla.org/en-US/docs/Web/API/HTML_Drag_and_Drop_API

### バージョン管理

#### Git

- **公式サイト**: https://git-scm.com/
- **ドキュメント**: https://git-scm.com/doc
- **GitHub Docs**: https://docs.github.com/

## 参考: 技術選定の判断基準

実装時は以下の優先順位で情報を参照してください：

1. **公式ドキュメント**（この文書に記載のURL）
2. **GitHubリポジトリ**のREADME・Issues
3. **Stack Overflow**等のコミュニティ情報（公式情報で解決できない場合のみ）

特に重要な注意点：

- **バージョンに注意**: Tauri 2.x と 1.x ではAPIが異なる可能性があります
- **PDFライブラリの選定**: 実装前に複数のライブラリを検証し、最適なものを選択
- **セキュリティ**: ファイル操作・PDF処理時のセキュリティ考慮事項を公式ドキュメントで確認

## 更新履歴

- 2024-12-19: 初版作成

