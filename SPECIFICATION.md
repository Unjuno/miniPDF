# miniPDF - 仕様書

## 1. 概要

この文書は、miniPDF の機能仕様、技術仕様、実装方法を定義するものです。

## 2. 機能仕様

### 2.1 ファイル操作

#### 2.1.1 PDFファイルの読み込み

**機能**: PDFファイルを開いて読み込む

**入力**:
- ファイルパス（ユーザーがファイル選択ダイアログで選択）

**処理**:
1. ファイルがPDF形式であることを検証
2. PDFファイルを読み込み
3. PDF構造を解析（ページ、画像、テキスト）
4. 解析結果をFrontendに送信

**出力**:
- PDF構造データ（後述のデータ構造仕様参照）
- エラー情報（読み込み失敗時）

**エラーハンドリング**:
- ファイルが存在しない → エラーメッセージ表示
- PDF形式が不正 → エラーメッセージ表示
- ファイルサイズが大きすぎる（100MB以上） → 警告表示
- メモリ不足 → エラーメッセージ表示

#### 2.1.2 PDFファイルの保存

**機能**: 編集後のPDFを保存する

**入力**:
- 編集内容（画像サイズ、改ページ位置、行間）
- 保存先パス（ユーザーがファイル保存ダイアログで選択）

**処理**:
1. 編集内容を統合
2. PDF構造を再構築
3. 新しいPDFファイルを生成
4. 指定されたパスに保存

**出力**:
- 保存成功/失敗の通知
- エラー情報（保存失敗時）

**エラーハンドリング**:
- 保存先ディレクトリが存在しない → エラーメッセージ表示
- 書き込み権限がない → エラーメッセージ表示
- ディスク容量不足 → エラーメッセージ表示

### 2.2 PDF表示

#### 2.2.1 PDFページの表示

**機能**: PDFページをWYSIWYG形式で表示する

**表示内容**:
- ページ全体のレンダリング
- 編集可能な要素（画像、改ページ位置）のハイライト
- ページ番号表示

**表示モード**:
- 単一ページ表示（デフォルト）
- 複数ページ表示（オプション、将来実装）

**ズーム機能**:
- ズームイン/アウト（50% - 200%）
- フィット表示（ページ全体を表示）
- 実際のサイズ表示（100%）

#### 2.2.2 編集可能要素の可視化

**機能**: 編集可能な要素を視覚的に識別できるようにする

**可視化方法**:
- 画像: 選択時に枠線とリサイズハンドルを表示
- 改ページ位置: ページ境界に線を表示
- テキストブロック: ホバー時に背景色を変更（オプション）

### 2.3 画像サイズ調整

#### 2.3.1 画像の選択

**機能**: PDF内の画像を選択する

**操作方法**:
- マウスクリックで画像を選択
- 選択された画像は枠線とリサイズハンドルを表示

**選択状態**:
- 1つの画像のみ選択可能
- 選択解除は背景をクリック

#### 2.3.2 画像のリサイズ

**機能**: 選択された画像のサイズを変更する

**操作方法**:
- リサイズハンドルをドラッグしてサイズ変更
- アスペクト比維持オプション（デフォルト: ON）

**制約**:
- 最小サイズ: 元のサイズの10%
- 最大サイズ: ページサイズの200%
- 画像がページ外に出ないように制限

**リアルタイムプレビュー**:
- ドラッグ中にリアルタイムでサイズ変更をプレビュー
- マウスを離した時点で変更を確定

#### 2.3.3 画像位置の調整（将来実装）

**機能**: 画像の位置を移動する（MVPでは未実装）

### 2.4 改ページ位置調整

#### 2.4.1 改ページ位置の検出

**機能**: PDF内の改ページ位置を自動検出する

**検出方法**:
- ページ境界を自動識別
- 各ページのY座標を記録

#### 2.4.2 改ページ位置の可視化

**機能**: 改ページ位置を視覚的に表示する

**表示方法**:
- ページ境界に点線を表示
- ページ番号を表示

#### 2.4.3 改ページ位置の調整

**機能**: 改ページ位置を前後に移動する

**操作方法**:
- 改ページ位置の線をドラッグして移動
- 移動範囲: 前後のページ境界内

**処理内容**:
1. 改ページ位置を移動
2. 前後のコンテンツを再計算
3. オーバーフローを検出
4. 必要に応じてページ分割・結合
5. 全体の再レイアウト

**制約**:
- ページ内のコンテンツがページ外に出ないように制限
- 最小ページ高さ: ページサイズの10%

### 2.5 行間調整

#### 2.5.1 テキストブロックの識別

**機能**: PDF内のテキストブロックを識別する

**識別内容**:
- テキストブロックの位置・サイズ
- 行単位での情報
- フォントサイズ・行間

#### 2.5.2 行間の調整

**機能**: テキストブロックの行間を変更する

**操作方法**:
- スライダーで行間を調整（0.5倍 - 2.0倍）
- 段落単位での調整
- 全体一括調整オプション

**処理内容**:
1. 行間値を変更
2. テキストブロック内の各行の位置を再計算
3. ブロック全体の高さを更新
4. ページ内収まり確認
5. 必要に応じて改ページ調整

**制約**:
- 行間が小さすぎて文字が重ならないように制限
- ページ内に収まるように自動調整

### 2.6 アンドゥ・リドゥ（将来実装）

**機能**: 操作を取り消す・やり直す（MVPでは未実装）

## 3. ユーザーインターフェース仕様

### 3.1 ウィンドウレイアウト

```
┌─────────────────────────────────────────┐
│  [ファイル] [編集] [表示] [ヘルプ]      │ ← メニューバー
├─────────────────────────────────────────┤
│  [開く] [保存] [元に戻す] [やり直し]     │ ← ツールバー
├─────────────────────────────────────────┤
│                                         │
│         PDF表示エリア                   │
│         (WYSIWYG)                       │
│                                         │
│                                         │
├─────────────────────────────────────────┤
│  [ページ: 1/10] [ズーム: 100%]         │ ← ステータスバー
└─────────────────────────────────────────┘
```

### 3.2 メニューバー

#### ファイルメニュー
- 開く (Ctrl+O)
- 保存 (Ctrl+S)
- 名前を付けて保存 (Ctrl+Shift+S)
- 終了 (Alt+F4)

#### 編集メニュー
- 元に戻す (Ctrl+Z) - 将来実装
- やり直し (Ctrl+Y) - 将来実装
- 設定 (Ctrl+,) - 将来実装

#### 表示メニュー
- ズームイン (Ctrl++)
- ズームアウト (Ctrl+-)
- 実際のサイズ (Ctrl+0)
- フィット表示 (Ctrl+F)

#### ヘルプメニュー
- 使い方
- バージョン情報

### 3.3 ツールバー

- **開くボタン**: PDFファイルを開く
- **保存ボタン**: 編集内容を保存
- **元に戻すボタン**: 操作を取り消す（将来実装）
- **やり直しボタン**: 操作をやり直す（将来実装）

### 3.4 右サイドバー（オプション）

編集モードに応じて表示:
- **画像編集モード**: 選択画像のサイズ情報、リサイズオプション
- **行間編集モード**: 行間調整スライダー、段落選択

### 3.5 キーボードショートカット

| 操作 | ショートカット |
|------|---------------|
| ファイルを開く | Ctrl+O |
| 保存 | Ctrl+S |
| 名前を付けて保存 | Ctrl+Shift+S |
| ズームイン | Ctrl++ |
| ズームアウト | Ctrl+- |
| 実際のサイズ | Ctrl+0 |
| フィット表示 | Ctrl+F |
| 終了 | Alt+F4 |

## 4. データ構造仕様

### 4.1 PDF構造データ（TypeScript）

```typescript
// PDF全体の構造
interface PdfStructure {
  pages: Page[];
  metadata: PdfMetadata;
  filePath: string;
}

// ページ情報
interface Page {
  pageNumber: number;        // 1始まり
  width: number;             // ポイント単位
  height: number;            // ポイント単位
  images: ImageElement[];    // 画像要素
  textBlocks: TextBlock[];    // テキストブロック
}

// 画像要素
interface ImageElement {
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
interface TextBlock {
  id: string;                // 一意のID
  x: number;                 // X座標
  y: number;                 // Y座標
  width: number;             // 幅
  height: number;            // 高さ
  text: string;              // テキスト内容
  fontSize: number;          // フォントサイズ
  lineHeight: number;        // 行間（倍率）
  fontFamily: string;        // フォントファミリー
}

// PDFメタデータ
interface PdfMetadata {
  title?: string;
  author?: string;
  subject?: string;
  creator?: string;
  producer?: string;
  creationDate?: Date;
  modificationDate?: Date;
}
```

### 4.2 編集状態データ（TypeScript）

```typescript
// 編集状態
interface EditState {
  pdfStructure: PdfStructure | null;
  selectedImageId: string | null;
  selectedTextBlockId: string | null;
  zoomLevel: number;         // ズーム倍率（0.5 - 2.0）
  viewMode: 'single' | 'continuous';
  changes: Change[];          // 変更履歴
}

// 変更内容
interface Change {
  id: string;
  type: 'image_resize' | 'page_break_adjust' | 'line_spacing_adjust';
  timestamp: Date;
  data: ChangeData;
}

// 変更データ
type ChangeData = 
  | { type: 'image_resize'; imageId: string; newWidth: number; newHeight: number }
  | { type: 'page_break_adjust'; pageNumber: number; newPosition: number }
  | { type: 'line_spacing_adjust'; textBlockId: string; newLineHeight: number };
```

### 4.3 Rust側データ構造

```rust
// PDF構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfStructure {
    pub pages: Vec<Page>,
    pub metadata: PdfMetadata,
    pub file_path: String,
}

// ページ情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub page_number: u32,
    pub width: f64,
    pub height: f64,
    pub images: Vec<ImageElement>,
    pub text_blocks: Vec<TextBlock>,
}

// 画像要素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageElement {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub original_width: f64,
    pub original_height: f64,
    pub data: Vec<u8>,  // 画像データ（バイナリ）
    pub format: ImageFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFormat {
    Png,
    Jpeg,
}

// テキストブロック
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlock {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub text: String,
    pub font_size: f64,
    pub line_height: f64,
    pub font_family: String,
}

// PDFメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
}
```

## 5. IPC通信仕様（Tauri）

### 5.1 コマンド定義

#### 5.1.1 load_pdf

**説明**: PDFファイルを読み込む

**リクエスト**:
```typescript
interface LoadPdfRequest {
  filePath: string;
}
```

**レスポンス**:
```typescript
interface LoadPdfResponse {
  success: boolean;
  data?: PdfStructure;
  error?: string;
}
```

**実装**:
```rust
#[tauri::command]
async fn load_pdf(file_path: String) -> Result<PdfStructure, String> {
    // PDF読み込み処理
}
```

#### 5.1.2 resize_image

**説明**: 画像のサイズを変更する

**リクエスト**:
```typescript
interface ResizeImageRequest {
  imageId: string;
  newWidth: number;
  newHeight: number;
}
```

**レスポンス**:
```typescript
interface ResizeImageResponse {
  success: boolean;
  data?: ImageElement;
  error?: string;
}
```

**実装**:
```rust
#[tauri::command]
async fn resize_image(
    image_id: String,
    new_width: f64,
    new_height: f64,
) -> Result<ImageElement, String> {
    // 画像リサイズ処理
}
```

#### 5.1.3 adjust_page_break

**説明**: 改ページ位置を調整する

**リクエスト**:
```typescript
interface AdjustPageBreakRequest {
  pageNumber: number;
  newPosition: number;
}
```

**レスポンス**:
```typescript
interface AdjustPageBreakResponse {
  success: boolean;
  data?: Page;
  error?: string;
}
```

**実装**:
```rust
#[tauri::command]
async fn adjust_page_break(
    page_number: u32,
    new_position: f64,
) -> Result<Page, String> {
    // 改ページ調整処理
}
```

#### 5.1.4 adjust_line_spacing

**説明**: 行間を調整する

**リクエスト**:
```typescript
interface AdjustLineSpacingRequest {
  textBlockId: string;
  newLineHeight: number;
}
```

**レスポンス**:
```typescript
interface AdjustLineSpacingResponse {
  success: boolean;
  data?: TextBlock;
  error?: string;
}
```

**実装**:
```rust
#[tauri::command]
async fn adjust_line_spacing(
    text_block_id: String,
    new_line_height: f64,
) -> Result<TextBlock, String> {
    // 行間調整処理
}
```

#### 5.1.5 generate_pdf

**説明**: 編集内容を反映したPDFを生成する

**リクエスト**:
```typescript
interface GeneratePdfRequest {
  pdfStructure: PdfStructure;
  changes: Change[];
}
```

**レスポンス**:
```typescript
interface GeneratePdfResponse {
  success: boolean;
  pdfData?: Uint8Array;  // PDFバイナリデータ
  error?: string;
}
```

**実装**:
```rust
#[tauri::command]
async fn generate_pdf(
    pdf_structure: PdfStructure,
    changes: Vec<Change>,
) -> Result<Vec<u8>, String> {
    // PDF生成処理
}
```

#### 5.1.6 save_pdf

**説明**: PDFファイルを保存する

**リクエスト**:
```typescript
interface SavePdfRequest {
  filePath: string;
  pdfData: Uint8Array;
}
```

**レスポンス**:
```typescript
interface SavePdfResponse {
  success: boolean;
  error?: string;
}
```

**実装**:
```rust
#[tauri::command]
async fn save_pdf(file_path: String, pdf_data: Vec<u8>) -> Result<(), String> {
    // PDF保存処理
}
```

### 5.2 イベント（将来実装）

進捗通知などのイベント送信（MVPでは未実装）

## 6. 実装方法の決定

### 6.1 技術選択の確定

#### 6.1.1 フロントエンド

**決定事項**:
- **フレームワーク**: React 18+ (TypeScript)
- **ビルドツール**: Vite
- **PDF表示**: **pdfjs-dist** (PDF.js) - より軽量で制御しやすいため
- **ドラッグ操作**: **カスタム実装** - 軽量性と柔軟性のため
- **状態管理**: **Zustand** - 軽量でシンプルなため
- **スタイリング**: **CSS Modules** - 軽量性重視

**理由**:
- PDF.jsはMozilla製で信頼性が高く、カスタマイズ性も高い
- react-draggableは不要な機能が多いため、カスタム実装で必要最小限に
- ZustandはReduxより軽量で、この規模のアプリに適している

#### 6.1.2 バックエンド（Rust）

**決定事項**:
- **PDF読み込み**: **lopdf** - PDF構造解析に適している
- **PDF生成**: **lopdf** - 読み込みと生成を統一
- **画像処理**: **image** crate
- **シリアライゼーション**: **serde** + **serde_json**
- **エラーハンドリング**: **anyhow**

**理由**:
- lopdfはRust製で軽量、PDF構造の読み書きに適している
- pdf crateは読み込みに特化しているが、生成機能が弱い
- pdfium-renderは重く、この用途には過剰

### 6.2 アーキテクチャパターン

#### 6.2.1 フロントエンドアーキテクチャ

**パターン**: Component-Based Architecture + State Management

```
src/
├── components/          # UIコンポーネント
│   ├── PDFViewer.tsx
│   ├── ImageResizer.tsx
│   ├── PageBreakEditor.tsx
│   ├── LineSpacingEditor.tsx
│   └── Toolbar.tsx
├── stores/             # 状態管理（Zustand）
│   └── pdfStore.ts
├── hooks/              # カスタムフック
│   ├── usePdfLoader.ts
│   └── useDragResize.ts
├── types/              # 型定義
│   └── pdf.ts
├── utils/              # ユーティリティ
│   └── pdfUtils.ts
└── App.tsx
```

#### 6.2.2 バックエンドアーキテクチャ

**パターン**: Modular Architecture

```
src-tauri/
├── src/
│   ├── main.rs
│   ├── commands/       # Tauriコマンド
│   │   ├── pdf_loader.rs
│   │   ├── image_resizer.rs
│   │   ├── page_break.rs
│   │   ├── line_spacing.rs
│   │   └── pdf_generator.rs
│   ├── models/         # データモデル
│   │   └── pdf_structure.rs
│   ├── services/       # ビジネスロジック
│   │   ├── pdf_reader.rs
│   │   ├── layout_adjuster.rs
│   │   └── image_processor.rs
│   └── utils/          # ユーティリティ
│       └── error.rs
└── Cargo.toml
```

### 6.3 実装方針

#### 6.3.1 PDF読み込み方針

1. **lopdfでPDFを読み込む**
2. **ページ単位で解析**
3. **画像を抽出**（XObject/Image）
4. **テキストを抽出**（Textオブジェクト）
5. **構造データを構築**
6. **FrontendにJSON形式で送信**

#### 6.3.2 PDF生成方針

1. **lopdfで新しいPDFドキュメントを作成**
2. **編集内容を反映**
3. **画像をリサイズして再配置**
4. **テキスト位置を再計算**
5. **ページを再構築**
6. **メタデータを保持**
7. **PDFバイナリを生成**

#### 6.3.3 画像リサイズ方針

1. **image crateで画像を読み込む**
2. **リサイズ処理を実行**
3. **画像データを更新**
4. **PDF構造を更新**
5. **プレビューを更新**

#### 6.3.4 改ページ調整方針

1. **改ページ位置を移動**
2. **前後のコンテンツを再計算**
3. **オーバーフローを検出**
4. **必要に応じてページ分割**
5. **全体の再レイアウト**

#### 6.3.5 行間調整方針

1. **テキストブロックの行を識別**
2. **行間を変更**
3. **各行のY座標を再計算**
4. **ブロック全体の高さを更新**
5. **ページ内収まり確認**
6. **必要に応じて改ページ調整**

## 7. モジュール設計

### 7.1 Frontendモジュール

#### 7.1.1 PDFViewer

**責務**: PDFページの表示

**主要機能**:
- PDF.jsを使用したPDFレンダリング
- ズーム機能
- ページナビゲーション

**インターフェース**:
```typescript
interface PDFViewerProps {
  pdfStructure: PdfStructure | null;
  zoomLevel: number;
  onPageChange?: (pageNumber: number) => void;
}
```

#### 7.1.2 ImageResizer

**責務**: 画像のサイズ調整UI

**主要機能**:
- 画像の選択
- リサイズハンドルの表示
- ドラッグによるサイズ変更

**インターフェース**:
```typescript
interface ImageResizerProps {
  image: ImageElement;
  onResize: (imageId: string, width: number, height: number) => void;
}
```

#### 7.1.3 PageBreakEditor

**責務**: 改ページ位置の調整UI

**主要機能**:
- 改ページ位置の可視化
- ドラッグによる位置調整

**インターフェース**:
```typescript
interface PageBreakEditorProps {
  page: Page;
  onAdjust: (pageNumber: number, newPosition: number) => void;
}
```

#### 7.1.4 LineSpacingEditor

**責務**: 行間調整UI

**主要機能**:
- 行間調整スライダー
- 段落選択
- 全体一括調整

**インターフェース**:
```typescript
interface LineSpacingEditorProps {
  textBlock: TextBlock;
  onAdjust: (textBlockId: string, lineHeight: number) => void;
}
```

#### 7.1.5 pdfStore (Zustand)

**責務**: PDF編集状態の管理

**状態**:
```typescript
interface PdfStore {
  pdfStructure: PdfStructure | null;
  selectedImageId: string | null;
  selectedTextBlockId: string | null;
  zoomLevel: number;
  changes: Change[];
  
  // Actions
  loadPdf: (filePath: string) => Promise<void>;
  selectImage: (imageId: string | null) => void;
  resizeImage: (imageId: string, width: number, height: number) => Promise<void>;
  adjustPageBreak: (pageNumber: number, position: number) => Promise<void>;
  adjustLineSpacing: (textBlockId: string, lineHeight: number) => Promise<void>;
  savePdf: (filePath: string) => Promise<void>;
}
```

### 7.2 Backendモジュール

#### 7.2.1 pdf_reader

**責務**: PDFファイルの読み込み・解析

**主要機能**:
- PDFファイルの読み込み
- ページ構造の解析
- 画像の抽出
- テキストの抽出

**インターフェース**:
```rust
pub struct PdfReader;

impl PdfReader {
    pub fn load_pdf(file_path: &str) -> Result<PdfStructure, PdfError>;
    pub fn extract_images(page: &Page) -> Vec<ImageElement>;
    pub fn extract_text_blocks(page: &Page) -> Vec<TextBlock>;
}
```

#### 7.2.2 layout_adjuster

**責務**: レイアウト調整ロジック

**主要機能**:
- 画像リサイズ処理
- 改ページ調整処理
- 行間調整処理
- オーバーフロー検出

**インターフェース**:
```rust
pub struct LayoutAdjuster;

impl LayoutAdjuster {
    pub fn resize_image(
        pdf: &mut PdfStructure,
        image_id: &str,
        new_width: f64,
        new_height: f64,
    ) -> Result<(), PdfError>;
    
    pub fn adjust_page_break(
        pdf: &mut PdfStructure,
        page_number: u32,
        new_position: f64,
    ) -> Result<(), PdfError>;
    
    pub fn adjust_line_spacing(
        pdf: &mut PdfStructure,
        text_block_id: &str,
        new_line_height: f64,
    ) -> Result<(), PdfError>;
}
```

#### 7.2.3 pdf_generator

**責務**: PDFファイルの生成

**主要機能**:
- PDF構造からPDFファイルを生成
- メタデータの保持
- 画像の埋め込み
- テキストの配置

**インターフェース**:
```rust
pub struct PdfGenerator;

impl PdfGenerator {
    pub fn generate(
        pdf_structure: &PdfStructure,
        changes: &[Change],
    ) -> Result<Vec<u8>, PdfError>;
}
```

#### 7.2.4 image_processor

**責務**: 画像処理

**主要機能**:
- 画像のリサイズ
- 画像形式の変換
- 画像データのエンコード

**インターフェース**:
```rust
pub struct ImageProcessor;

impl ImageProcessor {
    pub fn resize(
        image_data: &[u8],
        new_width: u32,
        new_height: u32,
    ) -> Result<Vec<u8>, ImageError>;
}
```

## 8. エラーハンドリング仕様

### 8.1 エラー分類

#### 8.1.1 ファイルエラー

- `FileNotFoundError`: ファイルが存在しない
- `FileReadError`: ファイル読み込み失敗
- `FileWriteError`: ファイル書き込み失敗
- `InvalidPdfError`: PDF形式が不正

#### 8.1.2 処理エラー

- `PdfParseError`: PDF解析失敗
- `ImageProcessError`: 画像処理失敗
- `LayoutError`: レイアウト調整失敗
- `PdfGenerationError`: PDF生成失敗

#### 8.1.3 バリデーションエラー

- `InvalidParameterError`: パラメータが不正
- `OutOfRangeError`: 値が範囲外
- `ConstraintViolationError`: 制約違反

### 8.2 エラー処理方針

1. **Backend**: RustのResult型でエラーを返す
2. **Frontend**: エラーメッセージをユーザーに表示
3. **ログ**: エラー詳細をログに記録（開発時）
4. **リカバリー**: 可能な限りリカバリー処理を実装

### 8.3 エラーメッセージ

- **ユーザー向け**: わかりやすい日本語メッセージ
- **開発者向け**: 詳細なエラー情報（ログ）

## 9. 非機能要件

### 9.1 パフォーマンス要件

- **起動時間**: 1秒以内
- **PDF読み込み**: 10MB以下で1秒以内
- **操作応答性**: ドラッグ操作は60fps
- **メモリ使用量**: 100MB以下（小規模PDF）
- **ファイルサイズ**: 実行ファイル50MB以下

### 9.2 セキュリティ要件

- **ファイル検証**: PDFファイルの形式検証
- **サンドボックス**: Tauriのサンドボックス環境を活用
- **メモリ安全**: Rustのメモリ安全性を活用
- **入力検証**: すべてのユーザー入力を検証

### 9.3 互換性要件

- **PDFバージョン**: PDF 1.4以上をサポート
- **OS**: Windows 10+, macOS 10.15+, Linux (主要ディストリビューション)
- **ファイルサイズ**: 最大100MBまでサポート

### 9.4 ユーザビリティ要件

- **直感的な操作**: ドラッグ&ドロップで操作可能
- **リアルタイムプレビュー**: 変更を即座に反映
- **エラーメッセージ**: わかりやすい日本語メッセージ
- **キーボードショートカット**: 主要操作にショートカットを提供

## 10. テスト仕様

### 10.1 単体テスト

#### 10.1.1 Frontend

- **コンポーネントテスト**: React Testing Library
- **フックテスト**: カスタムフックのテスト
- **ユーティリティテスト**: ユーティリティ関数のテスト

#### 10.1.2 Backend

- **モジュールテスト**: Rust標準テストフレームワーク
- **関数テスト**: 各関数のテスト
- **エラーハンドリングテスト**: エラーケースのテスト

### 10.2 統合テスト

- **IPC通信テスト**: Tauriコマンドのテスト
- **PDF読み込みテスト**: 様々なPDFファイルでのテスト
- **PDF生成テスト**: 生成されたPDFの検証

### 10.3 エンドツーエンドテスト

- **操作フローテスト**: ユーザー操作の一連の流れをテスト
- **UIテスト**: ユーザーインターフェースのテスト

## 11. 実装順序（MVP）

### フェーズ1: 基盤構築
1. プロジェクトセットアップ
2. 基本UI構築
3. IPC通信の基本実装

### フェーズ2: PDF読み込み・表示
1. PDF読み込み機能
2. PDF表示機能
3. データ構造定義

### フェーズ3: 画像サイズ調整
1. 画像検出・表示
2. ドラッグ操作UI
3. 画像リサイズ処理

### フェーズ4: PDF再生成・保存
1. PDF再生成機能
2. ファイル保存機能

### フェーズ5: 改ページ・行間調整（基本機能完成後）
1. 改ページ位置調整
2. 行間調整

## 12. 制約事項

### 12.1 技術的制約

- **PDFライブラリの制約**: lopdfの機能範囲内で実装
- **メモリ制約**: 大きなPDFファイルは段階的読み込み
- **パフォーマンス制約**: リアルタイム処理の最適化が必要

### 12.2 機能制約

- **MVPでは未実装**: アンドゥ・リドゥ、画像位置移動、複数ページ表示
- **対応範囲**: 基本的なPDF構造のみ（複雑な構造は未対応）

## 13. 将来の拡張

- アンドゥ・リドゥ機能
- 画像位置移動
- 複数ページ表示
- プリセット機能
- バッチ処理
- コマンドラインインターフェース

