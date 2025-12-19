# miniPDF - 実装ガイド（Step by Step）

この文書は、miniPDFを実装する際の詳細なステップバイステップガイドです。

## 目次

1. [フェーズ1: プロジェクトセットアップ](#フェーズ1-プロジェクトセットアップ)
2. [フェーズ2: PDF読み込み・表示](#フェーズ2-pdf読み込み表示)
3. [フェーズ3: 画像サイズ調整](#フェーズ3-画像サイズ調整)
4. [フェーズ4: PDF再生成・保存](#フェーズ4-pdf再生成保存)
5. [フェーズ5: 改ページ位置調整](#フェーズ5-改ページ位置調整)
6. [フェーズ6: 行間調整](#フェーズ6-行間調整)
7. [フェーズ7: 最適化・UX改善](#フェーズ7-最適化ux改善)

---

## フェーズ1: プロジェクトセットアップ

### Step 1.1: Tauriプロジェクトの初期化

**目的**: Tauri + React + TypeScriptプロジェクトの基本構造を作成

**手順**:

1. **Tauri CLIのインストール確認**
   ```bash
   npm install -g @tauri-apps/cli
   ```

2. **プロジェクトの初期化**
   ```bash
   npm create tauri-app@latest
   ```
   - Project name: `minipdf`
   - Template: `react-ts`
   - Package manager: `npm`

3. **プロジェクトディレクトリに移動**
   ```bash
   cd minipdf
   ```

4. **依存関係のインストール**
   ```bash
   npm install
   ```

5. **動作確認**
   ```bash
   npm run tauri dev
   ```
   - ウィンドウが開いて「Hello Tauri!」が表示されればOK

**確認事項**:
- [ ] プロジェクトが正常にビルドできる
- [ ] 開発サーバーが起動する
- [ ] 基本的なUIが表示される

**参考**: [Tauri公式ドキュメント](https://tauri.app/v1/guides/getting-started/prerequisites)

---

### Step 1.2: プロジェクト構造の整理

**目的**: 実装しやすいディレクトリ構造を作成

**手順**:

1. **フロントエンドディレクトリ構造の作成**
   ```
   src/
   ├── components/       # UIコンポーネント
   ├── stores/          # Zustandストア
   ├── hooks/           # カスタムフック
   ├── types/           # TypeScript型定義
   ├── utils/           # ユーティリティ関数
   └── App.tsx
   ```

2. **ディレクトリの作成**
   ```bash
   mkdir -p src/components src/stores src/hooks src/types src/utils
   ```

3. **バックエンドディレクトリ構造の作成**
   ```
   src-tauri/src/
   ├── commands/        # Tauriコマンド
   ├── models/          # データモデル
   ├── services/        # ビジネスロジック
   └── utils/           # ユーティリティ
   ```

4. **Rustディレクトリの作成**
   ```bash
   mkdir -p src-tauri/src/commands src-tauri/src/models src-tauri/src/services src-tauri/src/utils
   ```

**確認事項**:
- [ ] ディレクトリ構造が作成された
- [ ] 既存のファイルが壊れていない

---

### Step 1.3: 必要なライブラリのインストール

**目的**: プロジェクトに必要な依存関係を追加

**手順**:

1. **フロントエンド依存関係のインストール**
   ```bash
   npm install pdfjs-dist zustand
   npm install -D @types/node
   ```

2. **Rust依存関係の追加** (`src-tauri/Cargo.toml`)
   ```toml
   [dependencies]
   tauri = { version = "2.0", features = ["fs-all", "dialog-all"] }
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   lopdf = "0.32"
   image = "0.24"
   anyhow = "1.0"
   ```

3. **依存関係のインストール確認**
   ```bash
   cd src-tauri
   cargo check
   cd ..
   ```

**確認事項**:
- [ ] npmパッケージがインストールされた
- [ ] Rustの依存関係が解決された
- [ ] ビルドエラーがない

---

### Step 1.4: 基本UIの構築

**目的**: メインウィンドウのレイアウトを作成

**手順**:

1. **App.tsxの基本構造を作成**
   ```typescript
   // src/App.tsx
   import './App.css';

   function App() {
     return (
       <div className="app">
         <header className="app-header">
           <h1>miniPDF</h1>
         </header>
         <main className="app-main">
           <p>PDFファイルを開いてください</p>
         </main>
       </div>
     );
   }

   export default App;
   ```

2. **基本的なCSSを作成** (`src/App.css`)
   ```css
   .app {
     display: flex;
     flex-direction: column;
     height: 100vh;
   }

   .app-header {
     padding: 1rem;
     border-bottom: 1px solid #ccc;
   }

   .app-main {
     flex: 1;
     padding: 1rem;
   }
   ```

3. **動作確認**
   ```bash
   npm run tauri dev
   ```

**確認事項**:
- [ ] 基本的なUIが表示される
- [ ] レイアウトが正しく表示される

---

### Step 1.5: ファイル選択ダイアログの実装

**目的**: PDFファイルを選択する機能を実装

**手順**:

1. **Tauriコマンドの実装** (`src-tauri/src/commands/mod.rs`)
   ```rust
   use tauri::command;

   #[command]
   pub async fn open_file_dialog() -> Result<Option<String>, String> {
       use tauri::api::dialog::FileDialogBuilder;
       
       let file_path = FileDialogBuilder::new()
           .add_filter("PDF", &["pdf"])
           .pick_file()
           .await;
       
       Ok(file_path.map(|p| p.to_string_lossy().to_string()))
   }
   ```

2. **コマンドの登録** (`src-tauri/src/main.rs`)
   ```rust
   mod commands;
   use commands::open_file_dialog;

   fn main() {
       tauri::Builder::default()
           .invoke_handler(tauri::generate_handler![open_file_dialog])
           .run(tauri::generate_context!())
           .expect("error while running tauri application");
   }
   ```

3. **フロントエンドでの使用** (`src/App.tsx`)
   ```typescript
   import { invoke } from '@tauri-apps/api/tauri';

   function App() {
     const handleOpenFile = async () => {
       try {
         const filePath = await invoke<string | null>('open_file_dialog');
         if (filePath) {
           console.log('Selected file:', filePath);
         }
       } catch (error) {
         console.error('Error opening file:', error);
       }
     };

     return (
       <div className="app">
         <header className="app-header">
           <h1>miniPDF</h1>
           <button onClick={handleOpenFile}>ファイルを開く</button>
         </header>
         {/* ... */}
       </div>
     );
   }
   ```

4. **動作確認**
   - ボタンをクリックしてファイル選択ダイアログが開くことを確認

**確認事項**:
- [ ] ファイル選択ダイアログが開く
- [ ] 選択したファイルパスが取得できる

---

## フェーズ2: PDF読み込み・表示

### Step 2.1: PDF構造データ型の定義

**目的**: TypeScriptとRustでPDF構造を表現する型を定義

**手順**:

1. **TypeScript型定義** (`src/types/pdf.ts`)
   ```typescript
   export interface PdfStructure {
     pages: Page[];
     metadata: PdfMetadata;
     filePath: string;
   }

   export interface Page {
     pageNumber: number;
     width: number;
     height: number;
     images: ImageElement[];
     textBlocks: TextBlock[];
   }

   export interface ImageElement {
     id: string;
     x: number;
     y: number;
     width: number;
     height: number;
     originalWidth: number;
     originalHeight: number;
     data: string; // Base64
     format: 'png' | 'jpeg';
   }

   export interface TextBlock {
     id: string;
     x: number;
     y: number;
     width: number;
     height: number;
     text: string;
     fontSize: number;
     lineHeight: number;
     fontFamily: string;
   }

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

2. **Rust型定義** (`src-tauri/src/models/pdf_structure.rs`)
   ```rust
   use serde::{Deserialize, Serialize};

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct PdfStructure {
       pub pages: Vec<Page>,
       pub metadata: PdfMetadata,
       pub file_path: String,
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct Page {
       pub page_number: u32,
       pub width: f64,
       pub height: f64,
       pub images: Vec<ImageElement>,
       pub text_blocks: Vec<TextBlock>,
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct ImageElement {
       pub id: String,
       pub x: f64,
       pub y: f64,
       pub width: f64,
       pub height: f64,
       pub original_width: f64,
       pub original_height: f64,
       pub data: String, // Base64
       pub format: ImageFormat,
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub enum ImageFormat {
       Png,
       Jpeg,
   }

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

3. **モジュールの登録** (`src-tauri/src/models/mod.rs`)
   ```rust
   pub mod pdf_structure;
   pub use pdf_structure::*;
   ```

**確認事項**:
- [ ] 型定義が正しくコンパイルされる
- [ ] TypeScriptとRustの型が一致している

---

### Step 2.2: PDF読み込み機能の実装（Backend）

**目的**: PDFファイルを読み込んで構造を解析する

**手順**:

1. **PDFリーダーサービスの作成** (`src-tauri/src/services/pdf_reader.rs`)
   ```rust
   use lopdf::Document;
   use crate::models::*;
   use anyhow::Result;
   use std::fs;

   pub struct PdfReader;

   impl PdfReader {
       pub fn load_pdf(file_path: &str) -> Result<PdfStructure> {
           // PDFファイルを読み込む
           let doc = Document::load(file_path)?;
           
           // メタデータを取得
           let metadata = Self::extract_metadata(&doc);
           
           // ページを解析
           let pages = Self::extract_pages(&doc)?;
           
           Ok(PdfStructure {
               pages,
               metadata,
               file_path: file_path.to_string(),
           })
       }

       fn extract_metadata(doc: &Document) -> PdfMetadata {
           PdfMetadata {
               title: doc.trailer.get("Title")
                   .and_then(|v| v.as_string().ok())
                   .map(|s| s.to_string()),
               author: doc.trailer.get("Author")
                   .and_then(|v| v.as_string().ok())
                   .map(|s| s.to_string()),
               // ... 他のメタデータ
               ..Default::default()
           }
       }

       fn extract_pages(doc: &Document) -> Result<Vec<Page>> {
           let mut pages = Vec::new();
           
           for (page_num, page_id) in doc.get_pages() {
               let page = Self::extract_page(doc, *page_num, *page_id)?;
               pages.push(page);
           }
           
           Ok(pages)
       }

       fn extract_page(doc: &Document, page_num: u32, page_id: u32) -> Result<Page> {
           // ページサイズを取得
           let (width, height) = Self::get_page_size(doc, page_id)?;
           
           // 画像を抽出
           let images = Self::extract_images(doc, page_id, page_num)?;
           
           // テキストを抽出
           let text_blocks = Self::extract_text_blocks(doc, page_id, page_num)?;
           
           Ok(Page {
               page_number: page_num,
               width,
               height,
               images,
               text_blocks,
           })
       }

       fn get_page_size(doc: &Document, page_id: u32) -> Result<(f64, f64)> {
           // MediaBoxからページサイズを取得
           // 実装詳細は省略
           Ok((595.0, 842.0)) // A4サイズ（デフォルト）
       }

       fn extract_images(doc: &Document, page_id: u32, page_num: u32) -> Result<Vec<ImageElement>> {
           // 画像オブジェクトを抽出
           // 実装詳細は省略
           Ok(Vec::new())
       }

       fn extract_text_blocks(doc: &Document, page_id: u32, page_num: u32) -> Result<Vec<TextBlock>> {
           // テキストオブジェクトを抽出
           // 実装詳細は省略
           Ok(Vec::new())
       }
   }
   ```

2. **Tauriコマンドの実装** (`src-tauri/src/commands/pdf_loader.rs`)
   ```rust
   use tauri::command;
   use crate::services::pdf_reader::PdfReader;
   use crate::models::PdfStructure;

   #[command]
   pub async fn load_pdf(file_path: String) -> Result<PdfStructure, String> {
       PdfReader::load_pdf(&file_path)
           .map_err(|e| e.to_string())
   }
   ```

3. **コマンドの登録** (`src-tauri/src/main.rs`)
   ```rust
   mod commands;
   mod models;
   mod services;

   use commands::pdf_loader::load_pdf;

   fn main() {
       tauri::Builder::default()
           .invoke_handler(tauri::generate_handler![load_pdf])
           .run(tauri::generate_context!())
           .expect("error while running tauri application");
   }
   ```

**確認事項**:
- [ ] PDFファイルが読み込める
- [ ] エラーハンドリングが正しく動作する

---

### Step 2.3: PDF表示機能の実装（Frontend）

**目的**: PDF.jsを使用してPDFを表示する

**手順**:

1. **PDF.jsの設定** (`src/utils/pdfUtils.ts`)
   ```typescript
   import * as pdfjsLib from 'pdfjs-dist';

   // PDF.jsのワーカーを設定
   pdfjsLib.GlobalWorkerOptions.workerSrc = `//cdnjs.cloudflare.com/ajax/libs/pdf.js/${pdfjsLib.version}/pdf.worker.min.js`;

   export const loadPdf = async (filePath: string) => {
     const loadingTask = pdfjsLib.getDocument(filePath);
     const pdf = await loadingTask.promise;
     return pdf;
   };
   ```

2. **PDFViewerコンポーネントの作成** (`src/components/PDFViewer.tsx`)
   ```typescript
   import React, { useEffect, useRef, useState } from 'react';
   import * as pdfjsLib from 'pdfjs-dist';
   import { PdfStructure } from '../types/pdf';

   interface PDFViewerProps {
     pdfStructure: PdfStructure | null;
     zoomLevel: number;
   }

   export const PDFViewer: React.FC<PDFViewerProps> = ({ pdfStructure, zoomLevel }) => {
     const canvasRef = useRef<HTMLCanvasElement>(null);
     const [currentPage, setCurrentPage] = useState(1);

     useEffect(() => {
       if (!pdfStructure || !canvasRef.current) return;

       const renderPage = async () => {
         // PDF.jsでページをレンダリング
         // 実装詳細は省略
       };

       renderPage();
     }, [pdfStructure, currentPage, zoomLevel]);

     if (!pdfStructure) {
       return <div>PDFファイルを開いてください</div>;
     }

     return (
       <div className="pdf-viewer">
         <canvas ref={canvasRef} />
       </div>
     );
   };
   ```

3. **App.tsxでの使用**
   ```typescript
   import { PDFViewer } from './components/PDFViewer';
   import { useState } from 'react';
   import { invoke } from '@tauri-apps/api/tauri';
   import { PdfStructure } from './types/pdf';

   function App() {
     const [pdfStructure, setPdfStructure] = useState<PdfStructure | null>(null);
     const [zoomLevel, setZoomLevel] = useState(1.0);

     const handleOpenFile = async () => {
       try {
         const filePath = await invoke<string | null>('open_file_dialog');
         if (filePath) {
           const structure = await invoke<PdfStructure>('load_pdf', { filePath });
           setPdfStructure(structure);
         }
       } catch (error) {
         console.error('Error loading PDF:', error);
       }
     };

     return (
       <div className="app">
         <header className="app-header">
           <h1>miniPDF</h1>
           <button onClick={handleOpenFile}>ファイルを開く</button>
         </header>
         <main className="app-main">
           <PDFViewer pdfStructure={pdfStructure} zoomLevel={zoomLevel} />
         </main>
       </div>
     );
   }
   ```

**確認事項**:
- [ ] PDFファイルが表示される
- [ ] ズーム機能が動作する

---

## フェーズ3: 画像サイズ調整

### Step 3.1: 画像検出・表示機能の実装

**目的**: PDF内の画像を識別して表示する

**手順**:

1. **画像抽出機能の実装** (`src-tauri/src/services/pdf_reader.rs`)
   ```rust
   impl PdfReader {
       fn extract_images(doc: &Document, page_id: u32, page_num: u32) -> Result<Vec<ImageElement>> {
           let mut images = Vec::new();
           let mut image_index = 0;

           // XObjectから画像を抽出
           // 実装詳細は省略

           Ok(images)
       }
   }
   ```

2. **画像表示コンポーネント** (`src/components/ImageElement.tsx`)
   ```typescript
   import React from 'react';
   import { ImageElement as ImageElementType } from '../types/pdf';

   interface ImageElementProps {
     image: ImageElementType;
     isSelected: boolean;
     onSelect: () => void;
   }

   export const ImageElement: React.FC<ImageElementProps> = ({
     image,
     isSelected,
     onSelect,
   }) => {
     return (
       <div
         className={`image-element ${isSelected ? 'selected' : ''}`}
         style={{
           position: 'absolute',
           left: `${image.x}px`,
           top: `${image.y}px`,
           width: `${image.width}px`,
           height: `${image.height}px`,
         }}
         onClick={onSelect}
       >
         <img
           src={`data:image/${image.format};base64,${image.data}`}
           alt="PDF image"
           style={{ width: '100%', height: '100%' }}
         />
         {isSelected && (
           <div className="resize-handles">
             {/* リサイズハンドル */}
           </div>
         )}
       </div>
     );
   };
   ```

**確認事項**:
- [ ] PDF内の画像が表示される
- [ ] 画像をクリックして選択できる

---

### Step 3.2: ドラッグ操作UIの実装

**目的**: 画像をドラッグしてリサイズする機能

**手順**:

1. **カスタムフックの作成** (`src/hooks/useDragResize.ts`)
   ```typescript
   import { useState, useCallback } from 'react';

   export const useDragResize = (
     initialWidth: number,
     initialHeight: number,
     onResize: (width: number, height: number) => void
   ) => {
     const [isDragging, setIsDragging] = useState(false);
     const [startPos, setStartPos] = useState({ x: 0, y: 0 });

     const handleMouseDown = useCallback((e: React.MouseEvent) => {
       setIsDragging(true);
       setStartPos({ x: e.clientX, y: e.clientY });
     }, []);

     const handleMouseMove = useCallback((e: MouseEvent) => {
       if (!isDragging) return;

       const deltaX = e.clientX - startPos.x;
       const deltaY = e.clientY - startPos.y;

       const newWidth = initialWidth + deltaX;
       const newHeight = initialHeight + deltaY;

       onResize(newWidth, newHeight);
     }, [isDragging, startPos, initialWidth, initialHeight, onResize]);

     const handleMouseUp = useCallback(() => {
       setIsDragging(false);
     }, []);

     return {
       handleMouseDown,
       handleMouseMove,
       handleMouseUp,
       isDragging,
     };
   };
   ```

2. **リサイズハンドルの実装** (`src/components/ImageResizer.tsx`)
   ```typescript
   import React from 'react';
   import { ImageElement } from '../types/pdf';
   import { useDragResize } from '../hooks/useDragResize';

   interface ImageResizerProps {
     image: ImageElement;
     onResize: (imageId: string, width: number, height: number) => void;
   }

   export const ImageResizer: React.FC<ImageResizerProps> = ({
     image,
     onResize,
   }) => {
     const { handleMouseDown } = useDragResize(
       image.width,
       image.height,
       (width, height) => onResize(image.id, width, height)
     );

     return (
       <div className="resize-handle" onMouseDown={handleMouseDown}>
         {/* リサイズハンドルのUI */}
       </div>
     );
   };
   ```

**確認事項**:
- [ ] リサイズハンドルが表示される
- [ ] ドラッグで画像サイズが変更される

---

### Step 3.3: 画像リサイズ処理の実装（Backend）

**目的**: 画像をリサイズしてPDF構造を更新する

**手順**:

1. **画像リサイズサービスの実装** (`src-tauri/src/services/image_processor.rs`)
   ```rust
   use image::{DynamicImage, ImageBuffer};
   use anyhow::Result;

   pub struct ImageProcessor;

   impl ImageProcessor {
       pub fn resize(image_data: &[u8], new_width: u32, new_height: u32) -> Result<Vec<u8>> {
           let img = image::load_from_memory(image_data)?;
           let resized = img.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3);
           
           let mut buffer = Vec::new();
           resized.write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageOutputFormat::Png)?;
           
           Ok(buffer)
       }
   }
   ```

2. **Tauriコマンドの実装** (`src-tauri/src/commands/image_resizer.rs`)
   ```rust
   use tauri::command;
   use crate::models::ImageElement;

   #[command]
   pub async fn resize_image(
       image_id: String,
       new_width: f64,
       new_height: f64,
   ) -> Result<ImageElement, String> {
       // 画像リサイズ処理
       // 実装詳細は省略
       Err("Not implemented".to_string())
   }
   ```

**確認事項**:
- [ ] 画像が正しくリサイズされる
- [ ] PDF構造が更新される

---

## フェーズ4: PDF再生成・保存

### Step 4.1: PDF再生成機能の実装

**目的**: 編集内容を反映したPDFを生成する

**手順**:

1. **PDF生成サービスの実装** (`src-tauri/src/services/pdf_generator.rs`)
   ```rust
   use lopdf::Document;
   use crate::models::*;
   use anyhow::Result;

   pub struct PdfGenerator;

   impl PdfGenerator {
       pub fn generate(pdf_structure: &PdfStructure, changes: &[Change]) -> Result<Vec<u8>> {
           // 新しいPDFドキュメントを作成
           let mut doc = Document::with_version("1.4");

           // ページを再構築
           for page in &pdf_structure.pages {
               Self::add_page(&mut doc, page)?;
           }

           // メタデータを設定
           Self::set_metadata(&mut doc, &pdf_structure.metadata);

           // PDFバイナリを生成
           let mut buffer = Vec::new();
           doc.save_to(&mut buffer)?;

           Ok(buffer)
       }

       fn add_page(doc: &mut Document, page: &Page) -> Result<()> {
           // ページを追加
           // 実装詳細は省略
           Ok(())
       }

       fn set_metadata(doc: &mut Document, metadata: &PdfMetadata) {
           // メタデータを設定
           // 実装詳細は省略
       }
   }
   ```

2. **Tauriコマンドの実装** (`src-tauri/src/commands/pdf_generator.rs`)
   ```rust
   use tauri::command;
   use crate::models::{PdfStructure, Change};
   use crate::services::pdf_generator::PdfGenerator;

   #[command]
   pub async fn generate_pdf(
       pdf_structure: PdfStructure,
       changes: Vec<Change>,
   ) -> Result<Vec<u8>, String> {
       PdfGenerator::generate(&pdf_structure, &changes)
           .map_err(|e| e.to_string())
   }
   ```

**確認事項**:
- [ ] PDFが正しく生成される
- [ ] 編集内容が反映される

---

### Step 4.2: ファイル保存機能の実装

**目的**: 生成したPDFをファイルに保存する

**手順**:

1. **保存ダイアログの実装** (`src-tauri/src/commands/file_saver.rs`)
   ```rust
   use tauri::command;
   use std::fs;

   #[command]
   pub async fn save_file_dialog() -> Result<Option<String>, String> {
       use tauri::api::dialog::FileDialogBuilder;
       
       let file_path = FileDialogBuilder::new()
           .add_filter("PDF", &["pdf"])
           .set_file_name("output.pdf")
           .save_file()
           .await;
       
       Ok(file_path.map(|p| p.to_string_lossy().to_string()))
   }

   #[command]
   pub async fn save_pdf(file_path: String, pdf_data: Vec<u8>) -> Result<(), String> {
       fs::write(&file_path, pdf_data)
           .map_err(|e| e.to_string())
   }
   ```

2. **フロントエンドでの使用**
   ```typescript
   const handleSave = async () => {
     if (!pdfStructure) return;

     try {
       const filePath = await invoke<string | null>('save_file_dialog');
       if (filePath) {
         const pdfData = await invoke<number[]>('generate_pdf', {
           pdfStructure,
           changes: [],
         });
         
         await invoke('save_pdf', {
           filePath,
           pdfData: new Uint8Array(pdfData),
         });
         
         alert('保存しました');
       }
     } catch (error) {
       console.error('Error saving PDF:', error);
       alert('保存に失敗しました');
     }
   };
   ```

**確認事項**:
- [ ] 保存ダイアログが開く
- [ ] PDFファイルが保存される

---

## フェーズ5: 改ページ位置調整

### Step 5.1: 改ページ検出機能の実装

**目的**: PDF内の改ページ位置を識別する

**手順**:

1. **改ページ検出の実装** (`src-tauri/src/services/layout_adjuster.rs`)
   ```rust
   impl LayoutAdjuster {
       pub fn detect_page_breaks(pdf: &PdfStructure) -> Vec<PageBreak> {
           let mut page_breaks = Vec::new();
           
           for (i, page) in pdf.pages.iter().enumerate() {
               if i > 0 {
                   page_breaks.push(PageBreak {
                       id: format!("break_{}", i),
                       position: 0.0, // 前ページの下端
                       page_number: i as u32,
                   });
               }
           }
           
           page_breaks
       }
   }
   ```

2. **改ページ表示コンポーネント** (`src/components/PageBreakEditor.tsx`)
   ```typescript
   import React from 'react';

   interface PageBreakEditorProps {
     pageBreaks: PageBreak[];
     onAdjust: (pageNumber: number, newPosition: number) => void;
   }

   export const PageBreakEditor: React.FC<PageBreakEditorProps> = ({
     pageBreaks,
     onAdjust,
   }) => {
     return (
       <div className="page-break-editor">
         {pageBreaks.map((break_) => (
           <div
             key={break_.id}
             className="page-break-line"
             style={{ top: `${break_.position}px` }}
             // ドラッグ処理
           />
         ))}
       </div>
     );
   };
   ```

**確認事項**:
- [ ] 改ページ位置が検出される
- [ ] 改ページ位置が表示される

---

### Step 5.2: 改ページ調整処理の実装

**目的**: 改ページ位置を移動してコンテンツを再配置する

**手順**:

1. **改ページ調整ロジック** (`src-tauri/src/services/layout_adjuster.rs`)
   ```rust
   impl LayoutAdjuster {
       pub fn adjust_page_break(
           pdf: &mut PdfStructure,
           page_number: u32,
           new_position: f64,
       ) -> Result<()> {
           // 改ページ位置を移動
           // 前後のコンテンツを再計算
           // オーバーフローを検出
           // 必要に応じてページ分割
           
           Ok(())
       }
   }
   ```

2. **Tauriコマンドの実装**
   ```rust
   #[command]
   pub async fn adjust_page_break(
       page_number: u32,
       new_position: f64,
   ) -> Result<Page, String> {
       // 実装
       Err("Not implemented".to_string())
   }
   ```

**確認事項**:
- [ ] 改ページ位置が移動できる
- [ ] コンテンツが正しく再配置される

---

## フェーズ6: 行間調整

### Step 6.1: テキストブロック識別機能の実装

**目的**: PDF内のテキストブロックを識別する

**手順**:

1. **テキスト抽出の実装** (`src-tauri/src/services/pdf_reader.rs`)
   ```rust
   impl PdfReader {
       fn extract_text_blocks(doc: &Document, page_id: u32, page_num: u32) -> Result<Vec<TextBlock>> {
           // PDFからテキストを抽出
           // 行単位で情報を取得
           // 実装詳細は省略
           Ok(Vec::new())
       }
   }
   ```

**確認事項**:
- [ ] テキストブロックが識別される
- [ ] 行情報が取得できる

---

### Step 6.2: 行間調整UIの実装

**目的**: 行間を調整するUIを作成

**手順**:

1. **行間調整コンポーネント** (`src/components/LineSpacingEditor.tsx`)
   ```typescript
   import React, { useState } from 'react';
   import { TextBlock } from '../types/pdf';

   interface LineSpacingEditorProps {
     textBlock: TextBlock;
     onAdjust: (textBlockId: string, lineHeight: number) => void;
   }

   export const LineSpacingEditor: React.FC<LineSpacingEditorProps> = ({
     textBlock,
     onAdjust,
   }) => {
     const [lineHeight, setLineHeight] = useState(textBlock.lineHeight);

     const handleChange = (value: number) => {
       setLineHeight(value);
       onAdjust(textBlock.id, value);
     };

     return (
       <div className="line-spacing-editor">
         <label>行間: {lineHeight.toFixed(2)}</label>
         <input
           type="range"
           min="0.5"
           max="2.0"
           step="0.1"
           value={lineHeight}
           onChange={(e) => handleChange(parseFloat(e.target.value))}
         />
       </div>
     );
   };
   ```

**確認事項**:
- [ ] スライダーが表示される
- [ ] 行間が調整される

---

### Step 6.3: 行間調整処理の実装

**目的**: 行間を変更してテキスト位置を再計算する

**手順**:

1. **行間調整ロジック** (`src-tauri/src/services/layout_adjuster.rs`)
   ```rust
   impl LayoutAdjuster {
       pub fn adjust_line_spacing(
           pdf: &mut PdfStructure,
           text_block_id: &str,
           new_line_height: f64,
       ) -> Result<()> {
           // テキストブロックを検索
           // 行間を変更
           // 各行のY座標を再計算
           // ブロック全体の高さを更新
           // ページ内収まり確認
           
           Ok(())
       }
   }
   ```

**確認事項**:
- [ ] 行間が正しく変更される
- [ ] テキスト位置が再計算される

---

## フェーズ7: 最適化・UX改善

### Step 7.1: 状態管理の実装（Zustand）

**目的**: アプリケーションの状態を一元管理する

**手順**:

1. **Zustandストアの作成** (`src/stores/pdfStore.ts`)
   ```typescript
   import { create } from 'zustand';
   import { PdfStructure, ImageElement, TextBlock } from '../types/pdf';

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

   export const usePdfStore = create<PdfStore>((set, get) => ({
     pdfStructure: null,
     selectedImageId: null,
     selectedTextBlockId: null,
     zoomLevel: 1.0,
     changes: [],

     loadPdf: async (filePath: string) => {
       const structure = await invoke<PdfStructure>('load_pdf', { filePath });
       set({ pdfStructure: structure });
     },

     selectImage: (imageId: string | null) => {
       set({ selectedImageId: imageId });
     },

     resizeImage: async (imageId: string, width: number, height: number) => {
       // 実装
     },

     // ... 他のアクション
   }));
   ```

**確認事項**:
- [ ] 状態管理が正しく動作する
- [ ] コンポーネント間で状態が共有される

---

### Step 7.2: エラーハンドリングの強化

**目的**: エラーを適切に処理してユーザーに表示する

**手順**:

1. **エラー表示コンポーネント** (`src/components/ErrorDisplay.tsx`)
   ```typescript
   import React from 'react';

   interface ErrorDisplayProps {
     error: string | null;
     onDismiss: () => void;
   }

   export const ErrorDisplay: React.FC<ErrorDisplayProps> = ({
     error,
     onDismiss,
   }) => {
     if (!error) return null;

     return (
       <div className="error-display">
         <p>{error}</p>
         <button onClick={onDismiss}>閉じる</button>
       </div>
     );
   };
   ```

2. **エラーハンドリングの実装**
   - すべての非同期処理でtry-catchを使用
   - エラーメッセージをユーザーに表示

**確認事項**:
- [ ] エラーが適切に表示される
- [ ] エラーから回復できる

---

### Step 7.3: キーボードショートカットの実装

**目的**: 主要操作にキーボードショートカットを追加

**手順**:

1. **ショートカットフックの作成** (`src/hooks/useKeyboardShortcuts.ts`)
   ```typescript
   import { useEffect } from 'react';

   export const useKeyboardShortcuts = (
     shortcuts: Record<string, () => void>
   ) => {
     useEffect(() => {
       const handleKeyDown = (e: KeyboardEvent) => {
         const key = `${e.ctrlKey ? 'Ctrl+' : ''}${e.key}`;
         if (shortcuts[key]) {
           e.preventDefault();
           shortcuts[key]();
         }
       };

       window.addEventListener('keydown', handleKeyDown);
       return () => window.removeEventListener('keydown', handleKeyDown);
     }, [shortcuts]);
   };
   ```

2. **ショートカットの登録**
   ```typescript
   useKeyboardShortcuts({
     'Ctrl+O': handleOpenFile,
     'Ctrl+S': handleSave,
     'Ctrl++': () => setZoomLevel(z => Math.min(z + 0.1, 2.0)),
     'Ctrl+-': () => setZoomLevel(z => Math.max(z - 0.1, 0.5)),
   });
   ```

**確認事項**:
- [ ] ショートカットが動作する
- [ ] デフォルトのブラウザ動作が無効化される

---

## 実装チェックリスト

### フェーズ1: プロジェクトセットアップ
- [ ] Tauriプロジェクトの初期化
- [ ] プロジェクト構造の整理
- [ ] 必要なライブラリのインストール
- [ ] 基本UIの構築
- [ ] ファイル選択ダイアログの実装

### フェーズ2: PDF読み込み・表示
- [ ] PDF構造データ型の定義
- [ ] PDF読み込み機能の実装（Backend）
- [ ] PDF表示機能の実装（Frontend）

### フェーズ3: 画像サイズ調整
- [ ] 画像検出・表示機能の実装
- [ ] ドラッグ操作UIの実装
- [ ] 画像リサイズ処理の実装（Backend）

### フェーズ4: PDF再生成・保存
- [ ] PDF再生成機能の実装
- [ ] ファイル保存機能の実装

### フェーズ5: 改ページ位置調整
- [ ] 改ページ検出機能の実装
- [ ] 改ページ調整処理の実装

### フェーズ6: 行間調整
- [ ] テキストブロック識別機能の実装
- [ ] 行間調整UIの実装
- [ ] 行間調整処理の実装

### フェーズ7: 最適化・UX改善
- [ ] 状態管理の実装（Zustand）
- [ ] エラーハンドリングの強化
- [ ] キーボードショートカットの実装

---

## トラブルシューティング

### よくある問題と解決方法

1. **PDF.jsのワーカーエラー**
   - 解決: ワーカーのパスを正しく設定する

2. **Rustのコンパイルエラー**
   - 解決: `cargo check`でエラーを確認し、依存関係を更新

3. **IPC通信のエラー**
   - 解決: コマンドの登録と型定義を確認

4. **画像が表示されない**
   - 解決: Base64エンコードとデータ形式を確認

---

## 次のステップ

実装を進める際は、このガイドに従って各ステップを順番に実装してください。各ステップの確認事項をチェックしながら進めることで、確実に実装できます。

不明な点がある場合は、[SPECIFICATION.md](./SPECIFICATION.md)や[TECHNICAL_REFERENCES.md](./TECHNICAL_REFERENCES.md)を参照してください。

