use tauri::command;
use oxidize_pdf::parser::{PdfReader, PdfDocument};
use crate::models::pdf_structure::*;
use anyhow::{Result, Context};
use std::io::{Read, Seek};

#[command]
pub async fn load_pdf(file_path: String) -> Result<PdfStructure, String> {
    log::info!("load_pdf started: {file_path}");
    
    let reader = PdfReader::open(&file_path)
        .map_err(|e| {
            log::error!("load_pdf failed to open file: {e}");
            format!("PDFファイルを開けませんでした: {e}")
        })?;
    
    let doc = PdfDocument::new(reader);
    
    let metadata = extract_metadata(&doc)
        .map_err(|e| {
            log::error!("extract_metadata failed: {e}");
            format!("メタデータの抽出に失敗しました: {e}")
        })?;
    
    let pages = extract_pages(&doc, &file_path)
        .with_context(|| "PDFページの解析に失敗しました")
        .map_err(|e| {
            log::error!("extract_pages failed: {e}");
            format!("{e}")
        })?;
    
    log::info!("load_pdf completed: {} pages loaded", pages.len());
    
    Ok(PdfStructure {
        pages,
        metadata,
        file_path,
    })
}

fn extract_metadata<R: Read + Seek>(doc: &PdfDocument<R>) -> Result<PdfMetadata, String> {
    let mut metadata = PdfMetadata::default();
    
    // why: oxidize-pdfのメタデータAPIを使用してメタデータを抽出
    // alt: 低レベルAPIを使用（複雑でエラーが発生しやすい）
    // evidence: oxidize-pdfのメタデータAPIは高レベルで使いやすい
    if let Ok(parsed_metadata) = doc.metadata() {
        metadata.title = parsed_metadata.title.clone();
        metadata.author = parsed_metadata.author.clone();
        metadata.subject = parsed_metadata.subject.clone();
        metadata.creator = parsed_metadata.creator.clone();
        metadata.producer = parsed_metadata.producer.clone();
        metadata.creation_date = parsed_metadata.creation_date.clone();
        metadata.modification_date = parsed_metadata.modification_date.clone();
    }
    
    Ok(metadata)
}

fn extract_pages<R: Read + Seek>(doc: &PdfDocument<R>, file_path: &str) -> Result<Vec<Page>> {
    let mut pages = Vec::new();
    
    let page_count = doc.page_count()
        .with_context(|| "ページ数の取得に失敗しました")?;
    
    for i in 0..page_count {
        let page_num = i + 1;
        let page = extract_page(doc, i as usize, page_num, file_path)?;
        pages.push(page);
    }
    
    Ok(pages)
}

fn extract_page<R: Read + Seek>(
    doc: &PdfDocument<R>,
    page_index: usize,
    page_num: u32,
    _file_path: &str,
) -> Result<Page> {
    let parsed_page = doc.get_page(page_index as u32)
        .with_context(|| format!("ページ{page_num}の取得に失敗しました"))?;
    
    let width = parsed_page.width();
    let height = parsed_page.height();
    
    let images = extract_images(&parsed_page, page_num)
        .with_context(|| format!("ページ{page_num}の画像抽出に失敗しました"))?;
    
    let text_blocks = extract_text_blocks(doc, &parsed_page, page_num, width, height)
        .with_context(|| format!("ページ{page_num}のテキスト抽出に失敗しました"))?;
    
    Ok(Page {
        page_number: page_num,
        source_page_number: Some(page_num),
        width,
        height,
        images,
        text_blocks,
    })
}

fn extract_images(
    page: &oxidize_pdf::parser::ParsedPage,
    page_num: u32,
) -> Result<Vec<ImageElement>> {
    let mut images = Vec::new();
    let mut image_index = 0;
    
    // 注意: 画像抽出はフロントエンドのPDF.jsでも行われているため、
    // バックエンドでの画像抽出は補助的な役割
    // 位置情報は後でPDF.jsから取得される可能性がある
    if let Some(resources) = page.get_resources() {
        if let Some(xobjects) = resources.get("XObject") {
            if let Some(xobjects_dict) = xobjects.as_dict() {
                for _name in xobjects_dict.0.keys() {
                    // 画像オブジェクトを検出（実際のデータ抽出は将来の実装）
                    let image_element = ImageElement {
                        id: format!("image_{page_num}_{image_index}"),
                        x: 0.0,
                        y: 0.0,
                        width: 100.0,
                        height: 100.0,
                        original_width: 100.0,
                        original_height: 100.0,
                        data: String::new(), // データはフロントエンドで取得
                        format: ImageFormat::Png,
                    };
                    images.push(image_element);
                    image_index += 1;
                }
            }
        }
    }
    
    Ok(images)
}

fn extract_text_blocks<R: Read + Seek>(
    doc: &PdfDocument<R>,
    _page: &oxidize_pdf::parser::ParsedPage,
    page_num: u32,
    page_width: f64,
    page_height: f64,
) -> Result<Vec<TextBlock>> {
    let mut text_blocks = Vec::with_capacity(20);
    
    // why: oxidize-pdfのextract_textを使用してテキストを抽出
    // alt: 低レベルAPIを使用（複雑でエラーが発生しやすい）
    // evidence: oxidize-pdfのextract_textは高レベルで使いやすい
    if let Ok(text_pages) = doc.extract_text() {
        if let Some(page_text) = text_pages.get(page_num as usize - 1) {
            if !page_text.text.is_empty() {
                let trimmed_text = page_text.text.trim();
                let estimated_height = (trimmed_text.lines().count() as f64) * 14.0;
                
                let text_block = TextBlock {
                    id: format!("text_{page_num}"),
                    x: 72.0,
                    y: page_height - estimated_height - 72.0,
                    width: page_width - 144.0,
                    height: estimated_height.max(20.0),
                    text: trimmed_text.to_string(),
                    font_size: 12.0,
                    line_height: 1.2,
                    font_family: "Arial".to_string(),
                };
                text_blocks.push(text_block);
            }
        }
    }
    
    Ok(text_blocks)
}
