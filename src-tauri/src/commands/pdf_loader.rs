use crate::models::pdf_structure::*;
use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use oxidize_pdf::{
    extract_images_from_pages,
    parser::{ParsedPage, PdfDocument, PdfReader},
    ExtractImagesOptions,
};
use std::{
    fs,
    io::{Cursor, Read, Seek},
};
use tauri::command;
use tempfile::TempDir;

#[command]
pub async fn load_pdf(file_path: String) -> Result<PdfStructure, String> {
    log::info!("load_pdf started: {file_path}");

    let reader = PdfReader::open(&file_path).map_err(|e| {
        log::error!("load_pdf failed to open file: {e}");
        format!("PDFファイルを開けませんでした: {e}")
    })?;

    let doc = PdfDocument::new(reader);

    let metadata = extract_metadata(&doc).map_err(|e| {
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

    let page_count = doc
        .page_count()
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
    file_path: &str,
) -> Result<Page> {
    let parsed_page = doc
        .get_page(page_index as u32)
        .with_context(|| format!("ページ{page_num}の取得に失敗しました"))?;

    let width = parsed_page.width();
    let height = parsed_page.height();

    let images = extract_images(&parsed_page, page_index, page_num, file_path)
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
    page: &ParsedPage,
    page_index: usize,
    page_num: u32,
    file_path: &str,
) -> Result<Vec<ImageElement>> {
    let resources_have_xobjects = page
        .get_resources()
        .and_then(|resources| resources.get("XObject"))
        .and_then(|xobjects| xobjects.as_dict())
        .map(|dict| !dict.0.is_empty())
        .unwrap_or(false);

    if !resources_have_xobjects {
        return Ok(Vec::new());
    }

    let temp_dir = TempDir::new().context("画像抽出用の一時ディレクトリを作成できませんでした")?;
    let options = ExtractImagesOptions {
        output_dir: temp_dir.path().to_path_buf(),
        min_size: None,
        ..ExtractImagesOptions::default()
    };

    let extracted_images = match extract_images_from_pages(file_path, &[page_index], options) {
        Ok(images) => images,
        Err(error) => {
            log::warn!(
                "ページ{page_num}の画像抽出に失敗しました。画像データなしで継続します: {error}"
            );
            return fallback_image_placeholders(page, page_num);
        }
    };

    let mut images = Vec::with_capacity(extracted_images.len());
    for (image_index, extracted_image) in extracted_images.into_iter().enumerate() {
        let image_bytes = fs::read(&extracted_image.file_path).with_context(|| {
            format!(
                "画像ファイルの読み込みに失敗しました: {}",
                extracted_image.file_path.display()
            )
        })?;
        let (data, format) = normalize_extracted_image(&image_bytes, extracted_image.format)
            .with_context(|| {
                format!(
                    "ページ{page_num}の画像{}の正規化に失敗しました",
                    image_index + 1
                )
            })?;

        images.push(ImageElement {
            id: format!("image_{page_num}_{image_index}"),
            x: 0.0,
            y: 0.0,
            width: extracted_image.width as f64,
            height: extracted_image.height as f64,
            original_width: extracted_image.width as f64,
            original_height: extracted_image.height as f64,
            data,
            format,
        });
    }

    Ok(images)
}

fn normalize_extracted_image(
    image_bytes: &[u8],
    extracted_format: oxidize_pdf::ImageFormat,
) -> Result<(String, ImageFormat)> {
    match extracted_format {
        oxidize_pdf::ImageFormat::Jpeg => Ok((
            general_purpose::STANDARD.encode(image_bytes),
            ImageFormat::Jpeg,
        )),
        _ => {
            let decoded = image::load_from_memory(image_bytes)
                .context("画像データをデコードできませんでした")?;
            let mut png_bytes = Vec::new();
            decoded
                .write_to(
                    &mut Cursor::new(&mut png_bytes),
                    image::ImageOutputFormat::Png,
                )
                .context("画像データをPNGに変換できませんでした")?;
            Ok((
                general_purpose::STANDARD.encode(png_bytes),
                ImageFormat::Png,
            ))
        }
    }
}

fn fallback_image_placeholders(page: &ParsedPage, page_num: u32) -> Result<Vec<ImageElement>> {
    let mut images = Vec::new();
    let mut image_index = 0;

    if let Some(resources) = page.get_resources() {
        if let Some(xobjects) = resources.get("XObject") {
            if let Some(xobjects_dict) = xobjects.as_dict() {
                for _name in xobjects_dict.0.keys() {
                    images.push(ImageElement {
                        id: format!("image_{page_num}_{image_index}"),
                        x: 0.0,
                        y: 0.0,
                        width: 100.0,
                        height: 100.0,
                        original_width: 100.0,
                        original_height: 100.0,
                        data: String::new(),
                        format: ImageFormat::Png,
                    });
                    image_index += 1;
                }
            }
        }
    }

    Ok(images)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::pdf_generator::generate_pdf;
    use base64::engine::general_purpose;
    use image::{DynamicImage, ImageBuffer, ImageOutputFormat, Rgba};
    use std::{fs, io::Cursor};
    use tempfile::tempdir;

    fn sample_png_base64() -> String {
        let image = ImageBuffer::from_pixel(2, 2, Rgba([255, 0, 0, 255]));
        let mut png_bytes = Vec::new();
        DynamicImage::ImageRgba8(image)
            .write_to(&mut Cursor::new(&mut png_bytes), ImageOutputFormat::Png)
            .expect("sample png should encode");
        general_purpose::STANDARD.encode(png_bytes)
    }

    #[tokio::test]
    async fn load_pdf_preserves_extracted_image_data() {
        let image_data = sample_png_base64();
        let pdf = PdfStructure {
            pages: vec![Page {
                page_number: 1,
                source_page_number: Some(1),
                width: 200.0,
                height: 200.0,
                images: vec![ImageElement {
                    id: "image_1".to_string(),
                    x: 20.0,
                    y: 20.0,
                    width: 40.0,
                    height: 40.0,
                    original_width: 2.0,
                    original_height: 2.0,
                    data: image_data,
                    format: ImageFormat::Png,
                }],
                text_blocks: vec![],
            }],
            metadata: PdfMetadata::default(),
            file_path: "roundtrip-source.pdf".to_string(),
        };

        let generated_pdf = generate_pdf(pdf)
            .await
            .expect("PDF generation should succeed");
        let temp_dir = tempdir().expect("tempdir should be created");
        let pdf_path = temp_dir.path().join("roundtrip.pdf");
        fs::write(&pdf_path, generated_pdf).expect("generated PDF should be written");

        let loaded = load_pdf(pdf_path.to_string_lossy().to_string())
            .await
            .expect("PDF should load");

        assert_eq!(loaded.pages.len(), 1);
        assert_eq!(loaded.pages[0].images.len(), 1);
        assert!(!loaded.pages[0].images[0].data.is_empty());

        let extracted = general_purpose::STANDARD
            .decode(&loaded.pages[0].images[0].data)
            .expect("extracted image should be base64");
        assert!(
            extracted.starts_with(b"\x89PNG\r\n\x1a\n") || extracted.starts_with(&[0xFF, 0xD8]),
            "extracted image should be a real PNG or JPEG payload"
        );
    }
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
