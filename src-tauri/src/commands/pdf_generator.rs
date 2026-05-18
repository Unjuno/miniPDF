use crate::models::pdf_structure::{ImageFormat, Page as PdfPage, PdfMetadata, PdfStructure};
use crate::utils::font_manager;
use base64::{engine::general_purpose, Engine as _};
use oxidize_pdf::{Document, Font, Image, Page as OxidizePage};
use tauri::command;

#[command]
pub async fn generate_pdf(pdf_structure: PdfStructure) -> Result<Vec<u8>, String> {
    log::info!("generate_pdf started: {} pages", pdf_structure.pages.len());

    // why: PDF生成前にページ番号を検証して不整合を防ぐ
    // alt: 検証なしでPDF生成（ページ番号の不整合がPDFに反映される）
    // evidence: 検証により、ページ番号の不整合を早期に発見できる
    let mut pdf = pdf_structure;
    pdf.renumber_pages(); // 念のため再割り当て

    // why: 空のページリストの場合は検証をスキップ（空のPDFも生成可能）
    // alt: 空のページリストでも検証を実行（エラーになる）
    // evidence: 空のPDFも有効なPDFとして生成できる
    if !pdf.pages.is_empty() {
        pdf.validate_page_numbers()
            .map_err(|e| format!("ページ番号の検証に失敗しました: {e}"))?;
    }

    // 注意: 現時点では、`generate_pdf_with_preservation`と`generate_pdf_new`の実装は同じ
    // 将来的に元のPDFのリソース（フォント、画像など）を保持する機能を実装する予定
    // そのため、元のPDFが存在する場合は`generate_pdf_with_preservation`を呼び出す
    if std::path::Path::new(&pdf.file_path).exists() {
        generate_pdf_with_preservation(pdf).await
    } else {
        generate_pdf_new(pdf).await
    }
}

async fn generate_pdf_with_preservation(pdf: PdfStructure) -> Result<Vec<u8>, String> {
    // 注意: 将来的に元のPDFのリソース（フォント、画像など）を保持する機能を実装する予定
    // 現時点では、元のPDFを読み込むだけで、リソースの再利用は行っていない
    build_pdf_document(&pdf).await
}

async fn generate_pdf_new(pdf: PdfStructure) -> Result<Vec<u8>, String> {
    build_pdf_document(&pdf).await
}

/// PDFドキュメントを構築する共通関数
/// why: コードの重複を減らすため、共通のロジックを抽出
/// alt: 各関数で個別に実装（コードの重複が発生する）
/// evidence: 共通のロジックを抽出することで、メンテナンスが容易になる
async fn build_pdf_document(pdf: &PdfStructure) -> Result<Vec<u8>, String> {
    let mut doc = Document::new();

    // why: カスタムフォントをPDFドキュメントに登録（日本語などのマルチバイト文字をサポート）
    // alt: フォントを登録しない（日本語が文字化けする）
    // evidence: カスタムフォントを登録することで、日本語テキストを正しく表示できる
    // assumption: フォントファイルはsrc-tauri/fonts/ディレクトリに配置される
    register_custom_fonts(&mut doc)?;
    let jp_embedded = doc.has_custom_font("NotoSansJP");

    // メタデータを設定
    set_metadata_oxidize(&mut doc, &pdf.metadata);

    // PdfStructureからPageを作成
    for page_data in &pdf.pages {
        let page = create_page_from_structure(page_data, jp_embedded)
            .map_err(|e| format!("ページの作成に失敗しました: {e}"))?;
        doc.add_page(page);
    }

    // メモリバッファに保存
    let buffer = doc.to_bytes().map_err(|e| format!("PDF保存エラー: {e}"))?;

    log::info!("generate_pdf completed: {} bytes generated", buffer.len());
    Ok(buffer)
}

/// カスタムフォントをPDFドキュメントに登録
/// why: 日本語などのマルチバイト文字をサポートするため、カスタムフォントを登録
/// alt: 標準フォントのみを使用（日本語が文字化けする）
/// evidence: カスタムフォントを登録することで、日本語テキストを正しく表示できる
fn register_custom_fonts(doc: &mut Document) -> Result<(), String> {
    font_manager::register_fonts_on_document(doc)
}

fn create_page_from_structure(
    page_data: &PdfPage,
    jp_font_embedded: bool,
) -> Result<OxidizePage, String> {
    // ページサイズの検証
    if page_data.width.is_nan()
        || page_data.width.is_infinite()
        || page_data.height.is_nan()
        || page_data.height.is_infinite()
    {
        return Err(format!(
            "無効なページサイズ（NaNまたはInfinity）: {:.2}x{:.2}",
            page_data.width, page_data.height
        ));
    }
    if page_data.width <= 0.0 || page_data.height <= 0.0 {
        return Err(format!(
            "無効なページサイズ: {:.2}x{:.2}",
            page_data.width, page_data.height
        ));
    }

    let mut page = OxidizePage::new(page_data.width, page_data.height);

    // テキストブロックを追加
    for text_block in &page_data.text_blocks {
        if text_block.text.is_empty() {
            continue;
        }

        // フォントサイズの検証
        if text_block.font_size.is_nan()
            || text_block.font_size.is_infinite()
            || text_block.font_size <= 0.0
        {
            log::warn!(
                "無効なフォントサイズ: {}, スキップします",
                text_block.font_size
            );
            continue;
        }

        // why: フォントを選択（カスタムフォントが利用可能な場合は使用、そうでない場合は標準フォント）
        // alt: 常に標準フォントを使用（日本語が文字化けする）
        // evidence: カスタムフォントを使用することで、日本語テキストを正しく表示できる
        let contains_non_ascii = text_block.text.chars().any(|c| c as u32 > 127);
        let font = if contains_non_ascii {
            if jp_font_embedded {
                Font::custom("NotoSansJP")
            } else {
                log::warn!(
                    "日本語フォントが PDF に埋め込まれていません（src-tauri/fonts に NotoSansJP-Regular.ttf を配置してください）。一部文字が欠落する可能性があります: {}",
                    text_block.text.chars().take(50).collect::<String>()
                );
                Font::Helvetica
            }
        } else {
            // ASCII文字のみの場合は、標準フォントを使用
            match text_block.font_family.as_str() {
                "Times" | "Times-Roman" => Font::TimesRoman,
                "Courier" => Font::Courier,
                _ => Font::Helvetica,
            }
        };

        // why: PDF座標系は左下が原点のため、y座標を変換する必要がある
        // alt: y座標を変換しない（テキストが上下逆になる）
        // evidence: PDF座標系ではy=0がページの下端、y=heightが上端
        let y_position = page_data.height - text_block.y - text_block.height;

        // 座標の検証（警告のみ、エラーにはしない）
        if text_block.x.is_nan()
            || text_block.x.is_infinite()
            || y_position.is_nan()
            || y_position.is_infinite()
        {
            log::warn!(
                "テキストブロックの座標が無効です（NaNまたはInfinity）: x={}, y={}",
                text_block.x,
                y_position
            );
            continue;
        }
        if text_block.x < 0.0 || y_position < 0.0 {
            log::warn!(
                "テキストブロックの座標が範囲外です: x={}, y={}",
                text_block.x,
                y_position
            );
        }

        page.text()
            .set_font(font, text_block.font_size)
            .at(text_block.x, y_position)
            .write(&text_block.text)
            .map_err(|e| format!("テキスト追加エラー: {e}"))?;
    }

    // 画像を追加
    for (index, image) in page_data.images.iter().enumerate() {
        if image.data.is_empty() {
            continue; // 空の画像データはスキップ
        }

        // 画像サイズの検証
        if image.width.is_nan()
            || image.width.is_infinite()
            || image.height.is_nan()
            || image.height.is_infinite()
        {
            log::warn!(
                "無効な画像サイズ（NaNまたはInfinity）: {:.2}x{:.2}, スキップします",
                image.width,
                image.height
            );
            continue;
        }
        if image.width <= 0.0 || image.height <= 0.0 {
            log::warn!(
                "無効な画像サイズ: {:.2}x{:.2}, スキップします",
                image.width,
                image.height
            );
            continue;
        }

        let image_data = general_purpose::STANDARD
            .decode(&image.data)
            .map_err(|e| format!("画像データのデコードエラー: {e}"))?;

        let image_obj = match image.format {
            ImageFormat::Jpeg => Image::from_jpeg_data(image_data)
                .map_err(|e| format!("JPEG画像の読み込みエラー: {e}"))?,
            ImageFormat::Png => Image::from_png_data(image_data)
                .map_err(|e| format!("PNG画像の読み込みエラー: {e}"))?,
        };

        let image_name = format!("Im{index}");
        page.add_image(&image_name, image_obj);

        // PDF座標系は左下が原点のため、y座標を変換
        let y_position = page_data.height - image.y - image.height;

        // 座標の検証（警告のみ、エラーにはしない）
        if image.x.is_nan()
            || image.x.is_infinite()
            || y_position.is_nan()
            || y_position.is_infinite()
        {
            log::warn!(
                "画像の座標が無効です（NaNまたはInfinity）: x={}, y={}",
                image.x,
                y_position
            );
            continue;
        }
        if image.x < 0.0 || y_position < 0.0 {
            log::warn!("画像の座標が範囲外です: x={}, y={}", image.x, y_position);
        }

        page.graphics()
            .draw_image(&image_name, image.x, y_position, image.width, image.height);
    }

    Ok(page)
}

fn set_metadata_oxidize(doc: &mut Document, metadata: &PdfMetadata) {
    if let Some(ref title) = metadata.title {
        doc.set_title(title);
    }
    if let Some(ref author) = metadata.author {
        doc.set_author(author);
    }
    if let Some(ref subject) = metadata.subject {
        doc.set_subject(subject);
    }
    if let Some(ref creator) = metadata.creator {
        doc.set_creator(creator);
    }
    if let Some(ref producer) = metadata.producer {
        doc.set_producer(producer);
    }
    // 注意: oxidize-pdfのメタデータAPIでは、日付の設定方法が異なる可能性がある
    // 現時点では、基本的なメタデータのみを設定
}
