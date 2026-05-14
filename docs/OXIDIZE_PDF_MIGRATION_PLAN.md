# oxidize-pdf移行計画

## 概要

現在の`lopdf`ベースの実装を`oxidize-pdf`に段階的に移行し、PDF編集機能の信頼性とパフォーマンスを向上させます。特に、元のPDFのリソース（フォント、画像、その他のオブジェクト）を保持する機能を実現します。

## 現状の問題点

1. **リソースの損失**: ページを削除・編集した後に保存すると、元のPDFのリソース（フォント、画像など）が失われる
2. **メタデータの保持**: メタデータは保持されているが、リソースは保持されていない
3. **編集機能の制限**: `lopdf`では既存のPDFを編集するのが困難
4. **バグの可能性**: リソースの保持が不完全なため、予期しない動作が発生する可能性がある

## 移行の目的

1. **リソースの完全な保持**: 元のPDFのリソース（フォント、画像、その他のオブジェクト）を完全に保持
2. **パフォーマンスの向上**: 3,000-4,000ページ/秒の生成速度を実現
3. **バグの削減**: より堅牢なPDF編集機能により、バグを削減
4. **機能の拡張**: インクリメンタル更新機能により、より柔軟な編集が可能

## ライセンス対応

### オプション1: AGPL-3.0に準拠（推奨）
- プロジェクトのライセンスをAGPL-3.0に変更
- ソースコードを公開
- 商用利用時もソースコードを公開する必要がある

### オプション2: 商用ライセンスを取得
- `oxidize-pdf`の商用ライセンスを取得
- プロジェクトのライセンスをMITのまま維持可能

### オプション3: ハイブリッドアプローチ
- 読み込み・編集機能のみ`oxidize-pdf`を使用
- 生成機能は`lopdf`を使用（ただし、リソース保持の問題は残る）

**推奨**: オプション1（AGPL-3.0に準拠）を推奨します。プロジェクトの性質上、オープンソースとして公開することは問題ないと考えられます。

## 段階的移行計画

### フェーズ1: 準備と検証（1-2日）

#### 1.1 依存関係の追加
- `Cargo.toml`に`oxidize-pdf`を追加
- ライセンス要件を確認・対応

#### 1.2 既存機能のテスト
- 現在の`lopdf`ベースの実装のテストを実行
- 既存のバグを記録
- パフォーマンスベースラインを測定

#### 1.3 移行戦略の決定
- 完全移行 vs 段階的移行
- フォールバック戦略の決定

### フェーズ2: PDF読み込み機能の移行（2-3日）

#### 2.1 `load_pdf`の移行
- `lopdf::Document::load` → `oxidize_pdf::PdfReader::open`
- メタデータ抽出の移行
- ページ抽出の移行
- 画像抽出の移行
- テキスト抽出の移行

#### 2.2 テスト
- 既存のPDFファイルでテスト
- メタデータが正しく抽出されることを確認
- ページ、画像、テキストが正しく抽出されることを確認

#### 2.3 フォールバック
- `lopdf`ベースの実装をフォールバックとして保持
- エラー時に`lopdf`にフォールバック

### フェーズ3: PDF生成機能の移行（3-4日）

#### 3.1 `generate_pdf`の移行
- `lopdf::Document` → `oxidize_pdf::Document`
- ページ生成の移行
- リソース保持の実装
- メタデータ設定の移行

#### 3.2 インクリメンタル更新機能の実装
- `write_incremental_with_page_replacement`を使用
- 元のPDFのリソースを保持
- ページの置き換えを実装

#### 3.3 テスト
- 既存のPDFファイルでテスト
- リソースが正しく保持されることを確認
- メタデータが正しく保持されることを確認

### フェーズ4: 編集機能の移行（2-3日）

#### 4.1 ページ操作の移行
- `reorder_pages`: `PdfStructure`の操作のみ（変更不要）
- `delete_page`: `PdfStructure`の操作のみ（変更不要）
- `add_page`: `PdfStructure`の操作のみ（変更不要）

#### 4.2 テキスト編集の移行
- `edit_text_block`: `PdfStructure`の操作のみ（変更不要）
- `add_text_block`: `PdfStructure`の操作のみ（変更不要）

#### 4.3 画像操作の移行
- `resize_image`: `PdfStructure`の操作のみ（変更不要）
- `move_image`: `PdfStructure`の操作のみ（変更不要）
- `insert_image`: `PdfStructure`の操作のみ（変更不要）

### フェーズ5: 統合と最適化（2-3日）

#### 5.1 統合テスト
- すべての機能を統合してテスト
- エッジケースのテスト
- パフォーマンステスト

#### 5.2 最適化
- 不要なコードの削除
- パフォーマンスの最適化
- エラーハンドリングの改善

#### 5.3 ドキュメント更新
- APIドキュメントの更新
- ユーザードキュメントの更新
- 移行ガイドの作成

## 実装の詳細

### 1. PDF読み込み機能の移行

#### 現在の実装（lopdf）
```rust
use lopdf::Document;

pub async fn load_pdf(file_path: String) -> Result<PdfStructure, String> {
    let doc = Document::load(&file_path)?;
    let metadata = extract_metadata(&doc);
    let pages = extract_pages(&doc)?;
    Ok(PdfStructure { pages, metadata, file_path })
}
```

#### 移行後の実装（oxidize-pdf）
```rust
use oxidize_pdf::{PdfReader, PdfDocument};

pub async fn load_pdf(file_path: String) -> Result<PdfStructure, String> {
    let reader = PdfReader::open(&file_path)?;
    let pdf_doc = PdfDocument::new(reader);
    
    // メタデータ抽出
    let metadata = extract_metadata_oxidize(&pdf_doc)?;
    
    // ページ抽出
    let pages = extract_pages_oxidize(&pdf_doc)?;
    
    Ok(PdfStructure { pages, metadata, file_path })
}
```

### 2. PDF生成機能の移行

#### 現在の実装（lopdf）
```rust
use lopdf::Document;

pub async fn generate_pdf(pdf_structure: PdfStructure) -> Result<Vec<u8>, String> {
    let mut doc = Document::with_version("1.4");
    // ページを追加
    // メタデータを設定
    let mut buffer = Vec::new();
    doc.save_to(&mut buffer)?;
    Ok(buffer)
}
```

#### 移行後の実装（oxidize-pdf）
```rust
use oxidize_pdf::{Document, Page, Font, Color, Image, ImageFormat};
use oxidize_pdf::parser::{PdfReader, PdfDocument};
use oxidize_pdf::operations::merge_pdfs;
use std::io::Cursor;

pub async fn generate_pdf(pdf_structure: PdfStructure) -> Result<Vec<u8>, String> {
    // 元のPDFが存在する場合は、元のPDFを読み込んでリソースを保持
    if std::path::Path::new(&pdf_structure.file_path).exists() {
        generate_pdf_with_preservation(pdf_structure).await
    } else {
        generate_pdf_new(pdf_structure).await
    }
}

async fn generate_pdf_with_preservation(pdf_structure: PdfStructure) -> Result<Vec<u8>, String> {
    // 元のPDFを読み込む
    let reader = PdfReader::open(&pdf_structure.file_path)
        .map_err(|e| format!("元のPDFファイルを読み込めませんでした: {}", e))?;
    let original_doc = PdfDocument::new(reader);
    
    // 新しいPDFドキュメントを作成
    let mut new_doc = Document::new();
    
    // メタデータを設定
    if let Some(title) = &pdf_structure.metadata.title {
        new_doc.set_title(title);
    }
    if let Some(author) = &pdf_structure.metadata.author {
        new_doc.set_author(author);
    }
    // ... その他のメタデータ
    
    // PdfStructureからPageを作成
    for page_data in &pdf_structure.pages {
        let mut page = Page::new(page_data.width, page_data.height);
        
        // テキストブロックを追加
        for text_block in &page_data.text_blocks {
            let font = match text_block.font_family.as_str() {
                "Times" | "Times-Roman" => Font::TimesRoman,
                "Courier" => Font::Courier,
                _ => Font::Helvetica,
            };
            
            page.text()
                .set_font(font, text_block.font_size)
                .at(text_block.x, page_data.height - text_block.y - text_block.height)
                .write(&text_block.text)
                .map_err(|e| format!("テキスト追加エラー: {}", e))?;
        }
        
        // 画像を追加
        for (index, image) in page_data.images.iter().enumerate() {
            let image_data = general_purpose::STANDARD
                .decode(&image.data)
                .map_err(|e| format!("画像データのデコードエラー: {}", e))?;
            
            let image_obj = match image.format {
                ImageFormat::Jpeg => Image::from_jpeg_data(image_data),
                ImageFormat::Png => Image::from_png_data(image_data)
                    .map_err(|e| format!("PNG画像の読み込みエラー: {}", e))?,
            };
            
            page.graphics()
                .image(&image_obj)
                .at(image.x, page_data.height - image.y - image.height)
                .size(image.width, image.height);
        }
        
        new_doc.add_page(page);
    }
    
    // メモリバッファに保存
    let mut buffer = Vec::new();
    new_doc.save_to(&mut buffer)
        .map_err(|e| format!("PDF保存エラー: {}", e))?;
    
    // 注意: 元のPDFのリソースを完全に保持するには、より高度な実装が必要
    // 現時点では、新しいPDFを生成し、必要に応じて元のPDFのリソースをコピーする
    // 将来的には、oxidize-pdfのリソース保持機能を使用する
    
    Ok(buffer)
}

async fn generate_pdf_new(pdf_structure: PdfStructure) -> Result<Vec<u8>, String> {
    let mut doc = Document::new();
    
    // メタデータを設定
    if let Some(title) = &pdf_structure.metadata.title {
        doc.set_title(title);
    }
    if let Some(author) = &pdf_structure.metadata.author {
        doc.set_author(author);
    }
    // ... その他のメタデータ
    
    // PdfStructureからPageを作成
    for page_data in &pdf_structure.pages {
        let mut page = Page::new(page_data.width, page_data.height);
        
        // テキストブロックを追加
        for text_block in &page_data.text_blocks {
            let font = match text_block.font_family.as_str() {
                "Times" | "Times-Roman" => Font::TimesRoman,
                "Courier" => Font::Courier,
                _ => Font::Helvetica,
            };
            
            page.text()
                .set_font(font, text_block.font_size)
                .at(text_block.x, page_data.height - text_block.y - text_block.height)
                .write(&text_block.text)
                .map_err(|e| format!("テキスト追加エラー: {}", e))?;
        }
        
        // 画像を追加
        for (index, image) in page_data.images.iter().enumerate() {
            let image_data = general_purpose::STANDARD
                .decode(&image.data)
                .map_err(|e| format!("画像データのデコードエラー: {}", e))?;
            
            let image_obj = match image.format {
                ImageFormat::Jpeg => Image::from_jpeg_data(image_data),
                ImageFormat::Png => Image::from_png_data(image_data)
                    .map_err(|e| format!("PNG画像の読み込みエラー: {}", e))?,
            };
            
            page.graphics()
                .image(&image_obj)
                .at(image.x, page_data.height - image.y - image.height)
                .size(image.width, image.height);
        }
        
        doc.add_page(page);
    }
    
    // メモリバッファに保存
    let mut buffer = Vec::new();
    doc.save_to(&mut buffer)
        .map_err(|e| format!("PDF保存エラー: {}", e))?;
    
    Ok(buffer)
}
```

### 3. リソース保持の実装

`oxidize-pdf`では、元のPDFのリソースを保持するために、以下のアプローチが考えられます：

#### アプローチ1: 元のPDFのリソースを抽出して再利用
```rust
use oxidize_pdf::parser::{PdfReader, PdfDocument};

async fn extract_resources_from_original(file_path: &str) -> Result<Resources, String> {
    let reader = PdfReader::open(file_path)?;
    let doc = PdfDocument::new(reader);
    
    // 元のPDFからリソース（フォント、画像など）を抽出
    // 各ページのリソースを収集
    let mut resources = Resources::new();
    
    for i in 0..doc.page_count()? {
        let page = doc.get_page(i)?;
        if let Some(page_resources) = page.get_resources() {
            // フォントを抽出
            if let Some(fonts) = page_resources.get("Font") {
                // フォント情報を保存
            }
            // 画像を抽出
            if let Some(xobjects) = page_resources.get("XObject") {
                // 画像情報を保存
            }
        }
    }
    
    Ok(resources)
}
```

#### アプローチ2: 元のPDFのページを保持し、必要な部分だけを置き換え
```rust
use oxidize_pdf::operations::merge_pdfs;

async fn generate_pdf_with_original_pages(
    pdf_structure: PdfStructure,
    original_file: &str,
) -> Result<Vec<u8>, String> {
    // 元のPDFから必要なページを抽出
    // 新しいページを生成
    // マージして新しいPDFを作成
    
    // 注意: このアプローチは、ページの内容を完全に置き換える場合には適さない
    // リソースを保持するには、アプローチ1を使用する
}
```

**推奨**: アプローチ1を使用して、元のPDFのリソースを抽出し、新しいPDFに再利用します。これにより、フォント、画像、その他のリソースが完全に保持されます。

## リスクと対策

### リスク1: ライセンスの互換性
- **対策**: AGPL-3.0に準拠するか、商用ライセンスを取得

### リスク2: APIの違いによる実装の複雑化
- **対策**: 段階的な移行により、各フェーズでテストを実施

### リスク3: パフォーマンスの低下
- **対策**: ベンチマークテストを実施し、必要に応じて最適化

### リスク4: 既存機能の破壊
- **対策**: フォールバック機能を実装し、エラー時に`lopdf`にフォールバック

## テスト計画

### 単体テスト
- 各機能の単体テストを実装
- 既存のテストを`oxidize-pdf`用に更新

### 統合テスト
- エンドツーエンドのテストを実施
- 既存のPDFファイルでテスト

### パフォーマンステスト
- ベンチマークテストを実施
- 既存の実装と比較

### 回帰テスト
- 既存の機能が正しく動作することを確認
- バグが発生しないことを確認

## 移行スケジュール

| フェーズ | 期間 | タスク |
|---------|------|--------|
| フェーズ1 | 1-2日 | 準備と検証 |
| フェーズ2 | 2-3日 | PDF読み込み機能の移行 |
| フェーズ3 | 3-4日 | PDF生成機能の移行 |
| フェーズ4 | 2-3日 | 編集機能の移行 |
| フェーズ5 | 2-3日 | 統合と最適化 |
| **合計** | **10-15日** | |

## 成功基準

1. **機能**: すべての既存機能が正しく動作する
2. **パフォーマンス**: 既存の実装と同等以上のパフォーマンス
3. **リソース保持**: 元のPDFのリソースが完全に保持される
4. **バグ**: 既存のバグが修正され、新しいバグが発生しない
5. **テスト**: すべてのテストがパスする

## 次のステップ

1. ライセンス要件を確認・決定
2. `oxidize-pdf`を`Cargo.toml`に追加
3. フェーズ1を開始（準備と検証）

