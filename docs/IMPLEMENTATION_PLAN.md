# miniPDF - 実装計画書（コンテキストエンジニアリング）

> **目的**: AI開発支援のための包括的な実装情報を体系的に整理

## 目次

1. [プロジェクト概要](#1-プロジェクト概要)
2. [技術スタック](#2-技術スタック)
3. [アーキテクチャ概要](#3-アーキテクチャ概要)
4. [実装済み機能](#4-実装済み機能)
5. [IPC通信仕様](#5-ipc通信仕様)
6. [データ構造](#6-データ構造)
7. [コードベース構造](#7-コードベース構造)
8. [開発フロー](#8-開発フロー)
9. [テスト戦略](#9-テスト戦略)
10. [制約事項と設計思想](#10-制約事項と設計思想)
11. [将来の拡張](#11-将来の拡張)

---

## 1. プロジェクト概要

### 1.1 目的

**miniPDF** は、Markdown を編集しながら PDF をライブプレビューし、必要なら保存する軽量デスクトップアプリです。メイン UI はプレビュー専用であり、PDF キャンバス上のレイアウト編集は提供しません。

### 1.2 解決する問題

- Mermaid図が大きすぎる
- 意図しない改ページ・改行
- 行が詰まらず、Wordみたいに整わない

### 1.3 設計思想

**核心原則:**
- **編集対象を極限まで絞る** - 3つの核心機能のみ
- **PDFを「最終成果物」として扱う** - 内部構造の完全編集はしない
- **超軽量・高速起動** - 1秒以内の起動を目指す

**やること:**
1. 画像（Mermaid図）のサイズ調整
2. 改行・改ページ位置の微調整
3. テキスト編集と改行調整

**やらないこと:**
- 文字内容の編集（本文編集は不要）
- PDFの完全な再構造化
- 署名・フォーム・高度注釈
- アンドゥ・リドゥ（将来実装）
- 画像位置移動（将来実装）

### 1.4 プロジェクト状態

- ✅ **全14機能実装完了**
- ✅ **ドキュメント整備済み**
- ⚠️ **将来拡張機能は未実装**

---

## 2. 技術スタック

### 2.1 フロントエンド

| 技術 | バージョン | 用途 |
|------|-----------|------|
| React | 18.2.0 | UIフレームワーク |
| TypeScript | 5.2.2 | 型安全性 |
| Vite | 5.0.8 | ビルドツール |
| Zustand | 4.5.7 | 状態管理 |
| PDF.js | 4.10.38 | PDF表示 |
| @tauri-apps/api | 2.0.0 | Tauri IPC通信 |
| @tauri-apps/plugin-fs | 2.4.4 | ファイルシステム操作 |

### 2.2 バックエンド（Rust）

| 技術 | バージョン | 用途 |
|------|-----------|------|
| Tauri | 2.0 | デスクトップアプリフレームワーク |
| lopdf | 0.32 | PDF読み込み・生成 |
| image | 0.24 | 画像処理 |
| serde | 1.0 | シリアライゼーション |
| anyhow | 1.0 | エラーハンドリング |
| base64 | 0.22 | Base64エンコード/デコード |
| uuid | 1.0 | 一意ID生成 |

### 2.3 開発ツール

| ツール | バージョン | 用途 |
|------|-----------|------|
| Vitest | 1.0.4 | フロントエンドテスト |
| Rust標準テスト | - | バックエンドテスト |
| Node.js | 18+ | 必須 |

### 2.4 ビルド設定

**Rustリリースビルド最適化:**
```toml
[profile.release]
opt-level = "z"  # サイズ最適化（起動時間重視）
lto = true       # リンク時最適化
codegen-units = 1
strip = true     # シンボル情報削除
```

**Viteビルド最適化:**
- コード分割（PDF.js、React、Tauriを個別チャンク）
- Tree shaking強化
- コンポーネント単位の遅延読み込み

---

## 3. アーキテクチャ概要

### 3.1 全体構成

```
┌─────────────────────────────────────────┐
│         Frontend (React + TS)           │
│  ┌──────────┐  ┌──────────┐           │
│  │ Zustand  │  │ PDF.js   │           │
│  │  Store   │  │ Viewer   │           │
│  └────┬─────┘  └────┬─────┘           │
│       │            │                  │
│       └────────────┼──────────────────┘
│                    │ IPC (Tauri)       │
└────────────────────┼──────────────────┘
                     │
┌────────────────────┼──────────────────┐
│                    │                  │
│    Backend (Rust)  │                  │
│  ┌──────────────┐ │                  │
│  │  Commands    │ │                  │
│  │  (IPC)       │ │                  │
│  └──────┬───────┘ │                  │
│         │         │                  │
│  ┌──────▼──────┐  │                  │
│  │ lopdf       │  │                  │
│  │ image crate │  │                  │
│  └─────────────┘  │                  │
└───────────────────────────────────────┘
```

### 3.2 データフロー

1. **PDF読み込み:**
   ```
   User → Frontend → IPC → Rust (lopdf) → PDF構造解析 → Frontend (Zustand)
   ```

2. **編集操作:**
   ```
   User → Frontend (UI) → Zustand → IPC → Rust (処理) → Frontend (更新)
   ```

3. **PDF保存:**
   ```
   Zustand → IPC → Rust (PDF生成) → ファイル保存 → Frontend (通知)
   ```

### 3.3 モジュール設計

**Frontend:**
- **Component-Based Architecture** - Reactコンポーネント単位
- **State Management** - Zustandで一元管理
- **Lazy Loading** - 重いコンポーネントは遅延読み込み

**Backend:**
- **Modular Architecture** - コマンド単位で分離
- **Error Handling** - anyhowで統一
- **Data Models** - serdeでシリアライゼーション

---

## 4. 実装済み機能

### 4.1 機能一覧（全14機能）

| カテゴリ | 機能 | 実装状態 | コマンド |
|---------|------|---------|---------|
| ファイル操作 | PDF読み込み | ✅ | `load_pdf` |
| ファイル操作 | PDF保存 | ✅ | `save_pdf` |
| PDF表示 | WYSIWYG表示 | ✅ | - |
| PDF表示 | ズーム（50-200%） | ✅ | - |
| ページ操作 | ページ追加 | ✅ | `add_page` |
| ページ操作 | ページ削除 | ✅ | `delete_page` |
| ページ操作 | ページ並び替え | ✅ | `reorder_pages` |
| 画像操作 | 画像選択 | ✅ | - |
| 画像操作 | 画像サイズ調整 | ✅ | `resize_image` |
| 画像操作 | 画像挿入 | ✅ | `insert_image` |
| テキスト操作 | テキスト編集 | ✅ | `edit_text_block` |
| テキスト操作 | テキスト追加 | ✅ | `add_text_block` |
| レイアウト調整 | 改ページ調整 | ✅ | `adjust_page_break` |

### 4.2 未実装機能（将来拡張）

- アンドゥ・リドゥ
- 画像位置移動
- 複数ページ表示
- プリセット機能
- バッチ処理
- コマンドラインインターフェース

---

## 5. IPC通信仕様

### 5.1 コマンド一覧

すべてのTauriコマンドは `src-tauri/src/main.rs` で登録されています。

#### 5.1.1 ファイル操作

**`open_file_dialog`**
```rust
pub async fn open_file_dialog(app: tauri::AppHandle) -> Result<Option<String>, String>
```
- **用途**: ファイル選択ダイアログを開く
- **戻り値**: 選択されたファイルパス（キャンセル時はNone）

**`save_file_dialog`**
```rust
pub async fn save_file_dialog(app: tauri::AppHandle) -> Result<Option<String>, String>
```
- **用途**: ファイル保存ダイアログを開く
- **戻り値**: 保存先ファイルパス（キャンセル時はNone）

#### 5.1.2 PDF操作

**`load_pdf`**
```rust
pub async fn load_pdf(file_path: String) -> Result<PdfStructure, String>
```
- **用途**: PDFファイルを読み込んで構造を解析
- **入力**: `file_path: String`
- **戻り値**: `PdfStructure`（ページ、画像、テキストブロックを含む）

**`generate_pdf`**
```rust
pub async fn generate_pdf(pdf_structure: PdfStructure) -> Result<Vec<u8>, String>
```
- **用途**: PDF構造からPDFバイナリを生成
- **入力**: `pdf_structure: PdfStructure`
- **戻り値**: PDFバイナリデータ（`Vec<u8>`）

**`save_pdf`**
```rust
pub async fn save_pdf(file_path: String, pdf_data: Vec<u8>) -> Result<(), String>
```
- **用途**: PDFバイナリをファイルに保存
- **入力**: `file_path: String`, `pdf_data: Vec<u8>`
- **戻り値**: 成功時は`()`、失敗時はエラーメッセージ

#### 5.1.3 画像操作

**`resize_image`**
```rust
pub async fn resize_image(
    image_id: String,
    new_width: f64,
    new_height: f64,
    image_data: String,      // Base64
    format: String,          // "png" or "jpeg"
    x: f64,
    y: f64,
    original_width: f64,
    original_height: f64,
) -> Result<ImageElement, String>
```
- **用途**: 画像をリサイズして新しいImageElementを返す
- **処理**: Base64デコード → image crateでリサイズ → Base64エンコード

**`insert_image`**
```rust
pub async fn insert_image(
    pdf_structure: PdfStructure,
    page_number: u32,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    image_data: String,      // Base64
    format: String,
) -> Result<PdfStructure, String>
```
- **用途**: PDFに新しい画像を挿入
- **戻り値**: 更新された`PdfStructure`

#### 5.1.4 テキスト操作

**`edit_text_block`**
```rust
pub async fn edit_text_block(
    pdf_structure: PdfStructure,
    text_block_id: String,
    new_text: String,
) -> Result<PdfStructure, String>
```
- **用途**: テキストブロックの内容を編集
- **処理**: テキスト更新 → 高さ自動調整

**`add_text_block`**
```rust
pub async fn add_text_block(
    pdf_structure: PdfStructure,
    page_number: u32,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    text: String,
    font_size: f64,
    line_height: f64,
    font_family: String,
) -> Result<PdfStructure, String>
```
- **用途**: 新しいテキストブロックを追加

#### 5.1.5 ページ操作

**`add_page`**
```rust
pub async fn add_page(
    pdf_structure: PdfStructure,
    page_number: u32,
    width: f64,
    height: f64,
) -> Result<PdfStructure, String>
```
- **用途**: 新しいページを追加
- **制約**: `page_number`は1始まり、既存ページ数+1まで

**`delete_page`**
```rust
pub async fn delete_page(
    pdf_structure: PdfStructure,
    page_number: u32,
) -> Result<PdfStructure, String>
```
- **用途**: ページを削除
- **制約**: 最後の1ページは削除不可

**`reorder_pages`**
```rust
pub async fn reorder_pages(
    pdf_structure: PdfStructure,
    from_index: u32,
    to_index: u32,
) -> Result<PdfStructure, String>
```
- **用途**: ページの順序を変更
- **入力**: インデックス（0始まり）

#### 5.1.6 レイアウト調整

**`adjust_page_break`**
```rust
pub async fn adjust_page_break(
    pdf_structure: PdfStructure,
    page_number: u32,
    new_position: f64,
) -> Result<PdfStructure, String>
```
- **用途**: 改ページ位置を調整
- **入力**: `new_position`はページ上端からの距離（pt）

### 5.2 エラーハンドリング

- **Rust側**: `Result<T, String>`でエラーを返す
- **Frontend側**: Zustandの`error`状態で管理
- **エラー表示**: `ErrorDisplay`コンポーネントで表示

---

## 6. データ構造

### 6.1 TypeScript型定義

**ファイル**: `src/types/pdf.ts`

```typescript
// PDF全体の構造
export interface PdfStructure {
  pages: Page[];
  metadata: PdfMetadata;
  filePath: string;
}

// ページ情報
export interface Page {
  pageNumber: number;        // 1始まり
  width: number;             // ポイント単位
  height: number;            // ポイント単位
  images: ImageElement[];    // 画像要素
  textBlocks: TextBlock[];    // テキストブロック
}

// 画像要素
export interface ImageElement {
  id: string;                // 一意のID
  x: number;                 // X座標（ポイント単位）
  y: number;                 // Y座標（ポイント単位）
  width: number;             // 幅（ポイント単位）
  height: number;            // 高さ（ポイント単位）
  originalWidth: number;     // 元の幅
  originalHeight: number;    // 元の高さ
  data: string;              // Base64エンコードされた画像データ
  format: 'png' | 'jpeg';    // 画像形式
}

// テキストブロック
export interface TextBlock {
  id: string;                // 一意のID
  x: number;                 // X座標
  y: number;                 // Y座標
  width: number;             // 幅
  height: number;            // 高さ
  text: string;              // テキスト内容
  fontSize: number;          // フォントサイズ
  lineHeight: number;         // 行間（倍率）
  fontFamily: string;         // フォントファミリー
}

// PDFメタデータ
export interface PdfMetadata {
  title?: string;
  author?: string;
  subject?: string;
  creator?: string;
  producer?: string;
  creationDate?: string;
  modificationDate?: string;
}
```

### 6.2 Rustデータ構造

**ファイル**: `src-tauri/src/models/pdf_structure.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfStructure {
    pub pages: Vec<Page>,
    pub metadata: PdfMetadata,
    #[serde(rename = "filePath")]
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    #[serde(rename = "pageNumber")]
    pub page_number: u32,
    pub width: f64,
    pub height: f64,
    pub images: Vec<ImageElement>,
    #[serde(rename = "textBlocks")]
    pub text_blocks: Vec<TextBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageElement {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(rename = "originalWidth")]
    pub original_width: f64,
    #[serde(rename = "originalHeight")]
    pub original_height: f64,
    pub data: String,  // Base64 encoded
    pub format: ImageFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Png,
    Jpeg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextBlock {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub text: String,
    #[serde(rename = "fontSize")]
    pub font_size: f64,
    #[serde(rename = "lineHeight")]
    pub line_height: f64,
    #[serde(rename = "fontFamily")]
    pub font_family: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    #[serde(rename = "creationDate")]
    pub creation_date: Option<String>,
    #[serde(rename = "modificationDate")]
    pub modification_date: Option<String>,
}
```

### 6.3 シリアライゼーション

- **Rust → TypeScript**: `serde`でJSONシリアライゼーション
- **命名規則**: Rustは`snake_case`、TypeScriptは`camelCase`（`serde(rename_all = "camelCase")`で変換）
- **画像データ**: Base64文字列として転送

---

## 7. コードベース構造

### 7.1 フロントエンド構造

```
src/
├── App.tsx                 # メインアプリコンポーネント
├── App.css                 # アプリスタイル
├── main.tsx                # エントリーポイント
├── index.css               # グローバルスタイル
│
├── components/             # UIコンポーネント
│   ├── PDFViewer.tsx       # PDF 表示（既定 previewOnly）
│   ├── ImageResizer.tsx    # 画像リサイズUI
│   ├── ImageElement.tsx    # 画像要素表示
│   ├── ImageInserter.tsx   # 画像挿入UI
│   ├── TextEditor.tsx      # テキスト編集UI
│   ├── InlineTextEditor.tsx # インラインテキストエディタ
│   ├── PageBreakEditor.tsx # 改ページ編集UI
│   ├── ErrorDisplay.tsx    # エラー表示
│   ├── ImageOverlay.tsx    # 画像オーバーレイ
│   └── TextBlockOverlay.tsx # テキストブロックオーバーレイ
│
├── stores/                 # 状態管理（Zustand）
│   └── pdfStore.ts         # PDF編集状態管理
│
├── hooks/                  # カスタムフック
│   ├── useDebounce.ts      # デバウンス処理
│   ├── useDragResize.ts    # ドラッグリサイズ
│   ├── useKeyboardShortcuts.ts # キーボードショートカット
│   └── useRenderCache.ts   # レンダリングキャッシュ
│
├── types/                  # 型定義
│   └── pdf.ts              # PDF関連型定義
│
└── utils/                  # ユーティリティ
    ├── pdfTextExtractor.ts # PDF.jsテキスト抽出
    └── pdfImageExtractor.ts # PDF.js画像抽出
```

### 7.2 バックエンド構造

```
src-tauri/
├── src/
│   ├── main.rs             # エントリーポイント（Tauri初期化）
│   ├── lib.rs              # ライブラリエントリ（未使用）
│   │
│   ├── commands/            # Tauriコマンド（IPC）
│   │   ├── mod.rs          # モジュール定義
│   │   ├── file_dialog.rs  # ファイル選択ダイアログ
│   │   ├── file_saver.rs   # ファイル保存
│   │   ├── pdf_loader.rs   # PDF読み込み
│   │   ├── pdf_generator.rs # PDF生成
│   │   ├── image_resizer.rs # 画像リサイズ
│   │   ├── image_inserter.rs # 画像挿入
│   │   ├── text_editor.rs  # テキスト編集
│   │   ├── page_editor.rs  # ページ操作
│   │   └── page_break.rs  # 改ページ調整
│   │
│   └── models/             # データモデル
│       ├── mod.rs
│       └── pdf_structure.rs # PDF構造定義
│
├── Cargo.toml              # Rust依存関係
├── tauri.conf.json         # Tauri設定
└── tests/                  # 統合テスト
    └── integration/
        ├── mod.rs
        └── ipc_test.rs     # IPC通信テスト
```

### 7.3 主要ファイルの責務

**Frontend:**
- `App.tsx`: アプリ全体のレイアウト、キーボードショートカット
- `pdfStore.ts`: 全PDF編集状態の一元管理、IPC呼び出し
- `PDFViewer.tsx`: PDF.jsを使用したPDF表示
- `PDFViewer.tsx`: PDF プレビュー（`previewOnly` 既定で閲覧専用）

**Backend:**
- `main.rs`: Tauriアプリ初期化、コマンド登録
- `pdf_loader.rs`: PDF解析、構造抽出
- `pdf_generator.rs`: PDF構造からPDFバイナリ生成
- `image_resizer.rs`: 画像リサイズ処理

---

## 8. 開発フロー

### 8.1 セットアップ

```bash
# 依存関係インストール
npm install

# Rust依存関係（自動）
# Tauriが自動的にビルド時に解決
```

### 8.2 開発コマンド

```bash
# 開発サーバー起動（フロントエンド + Tauri）
npm run tauri:dev

# フロントエンドのみ（開発時）
npm run dev

# ビルド
npm run tauri:build

# フロントエンドビルドのみ
npm run build
```

### 8.3 テストコマンド

```bash
# フロントエンドテスト
npm run test
npm run test:ui        # UI付きテストランナー
npm run test:coverage  # カバレッジ

# バックエンドテスト
cd src-tauri
cargo test
```

### 8.4 開発時の注意事項

1. **PDF.js Worker設定**: `pdfStore.ts`でWorkerパスを設定
2. **IPC呼び出し**: `invoke()`を使用（`@tauri-apps/api/core`）
3. **エラーハンドリング**: Zustandの`error`状態で管理
4. **型安全性**: TypeScriptとRustの型定義を同期

### 8.5 ビルド最適化

**Vite設定** (`vite.config.ts`):
- コード分割（PDF.js、React、Tauriを個別チャンク）
- Tree shaking強化
- コンポーネント単位の遅延読み込み

**Rust設定** (`Cargo.toml`):
- リリースビルドで最適化（`opt-level = "z"`）
- LTO有効化
- シンボル削除

---

## 9. テスト戦略

### 9.1 フロントエンドテスト

**フレームワーク**: Vitest + React Testing Library

**テスト対象:**
- コンポーネントのレンダリング
- ユーザーインタラクション
- カスタムフック
- ユーティリティ関数

**テストファイル配置:**
- `src/**/*.test.ts`
- `src/**/*.test.tsx`

### 9.2 バックエンドテスト

**フレームワーク**: Rust標準テスト

**テスト対象:**
- データモデルのシリアライゼーション
- PDF読み込み・生成
- 画像処理
- エラーハンドリング

**統合テスト:**
- `src-tauri/tests/integration/ipc_test.rs`
- IPC通信のエンドツーエンドテスト

### 9.3 テスト実行

```bash
# 全テスト実行
npm run test && cd src-tauri && cargo test

# ウォッチモード（フロントエンド）
npm run test -- --watch
```

---

## 10. 制約事項と設計思想

### 10.1 技術的制約

1. **PDFライブラリの制約**
   - `lopdf`の機能範囲内で実装
   - 複雑なPDF構造（フォーム、署名）は完全対応不可

2. **メモリ制約**
   - 大きなPDFファイル（100MB以上）は警告
   - 段階的読み込みを検討（未実装）

3. **パフォーマンス制約**
   - リアルタイム処理の最適化が必要
   - ドラッグ操作は60fpsを目指す

### 10.2 機能制約

1. **未実装機能**
   - アンドゥ・リドゥ
   - 画像位置移動
   - 複数ページ表示
   - プリセット機能

2. **対応範囲**
   - 基本的なPDF構造のみ
   - Markdown を書きながら PDF をプレビューし、保存まで行う軽量フローに特化

### 10.3 設計原則

1. **最小限の機能**
   - 核心機能3つに集中
   - 余計な機能は排除

2. **軽量・高速**
   - 起動1秒以内
   - コード分割で初期バンドル削減

3. **直感的操作**
   - ドラッグ&ドロップ
   - WYSIWYG表示

4. **検索可能性維持**
   - テキスト検索機能を維持
   - PDF構造を破壊しない

---

## 11. 将来の拡張

### 11.1 優先度: 高

- **アンドゥ・リドゥ機能**
  - 操作履歴の管理
  - 状態のスナップショット

- **画像位置移動**
  - ドラッグで画像を移動
  - 座標更新

### 11.2 優先度: 中

- **複数ページ表示**
  - ページ一覧表示
  - サムネイル表示

- **プリセット機能**
  - よく使うサイズの保存
  - 一括適用

### 11.3 優先度: 低

- **バッチ処理**
  - 複数PDFの一括処理

- **コマンドラインインターフェース**
  - CLI版の提供

---

## 12. 参考ドキュメント

### 12.1 プロジェクトドキュメント

- **[README.md](../README.md)** - プロジェクト基本情報
- **[docs/FEATURES.md](./FEATURES.md)** - 機能説明書（ユーザー向け）
- **[docs/SPECIFICATION.md](./SPECIFICATION.md)** - 詳細仕様書
- **[docs/CONCEPT.md](./CONCEPT.md)** - 設計思想・コンセプト
- **[docs/INDEX.md](./INDEX.md)** - ドキュメント索引

### 12.2 外部リソース

- [Tauri Documentation](https://tauri.app/)
- [PDF.js Documentation](https://mozilla.github.io/pdf.js/)
- [lopdf Documentation](https://docs.rs/lopdf/)
- [Zustand Documentation](https://zustand-demo.pmnd.rs/)

---

## 13. 開発時のチェックリスト

### 13.1 新機能実装時

- [ ] IPCコマンドの型定義を確認
- [ ] Rust側のエラーハンドリングを実装
- [ ] Frontend側のエラー表示を実装
- [ ] Zustandストアにアクションを追加
- [ ] 型定義（TypeScript/Rust）を同期
- [ ] テストを追加

### 13.2 バグ修正時

- [ ] エラーメッセージを確認
- [ ] IPC通信のログを確認
- [ ] 型の整合性を確認
- [ ] テストを実行

### 13.3 リリース前

- [ ] 全テストを実行
- [ ] ビルドが成功することを確認
- [ ] ドキュメントを更新
- [ ] 変更履歴を更新

---

**最終更新**: 2024年
**バージョン**: 1.0.0

