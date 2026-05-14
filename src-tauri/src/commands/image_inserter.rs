use tauri::command;
use crate::models::pdf_structure::{PdfStructure, ImageFormat, ImageElement};
use anyhow::Result;

#[command]
pub async fn insert_image(
    pdf_structure: PdfStructure,
    page_number: u32,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    image_data: String, // Base64 encoded
    format: String, // "png" or "jpeg"
) -> Result<PdfStructure, String> {
    let mut pdf = pdf_structure;
    
    if page_number == 0 || page_number > pdf.pages.len() as u32 {
        return Err(format!("無効なページ番号: {page_number}"));
    }
    
    if width <= 0.0 || height <= 0.0 {
        return Err(format!("画像サイズが無効です: {width}x{height}"));
    }
    
    let page_index = (page_number - 1) as usize;
    let page = &pdf.pages[page_index];
    
    // 座標の検証
    if x < 0.0 || y < 0.0 || x + width > page.width || y + height > page.height {
        return Err(format!("画像の位置がページ範囲外です: ({x}, {y})"));
    }
    
    // Base64データの検証
    if image_data.is_empty() {
        return Err("画像データが空です".to_string());
    }
    
    let image_format = match format.to_lowercase().as_str() {
        "png" => ImageFormat::Png,
        "jpeg" | "jpg" => ImageFormat::Jpeg,
        _ => return Err(format!("サポートされていない画像フォーマット: {format}")),
    };
    
    // why: UUIDの生成を安全に行う（unwrap()が失敗する可能性がある）
    // alt: unwrap()を使用（UUID生成に失敗した場合にパニックする）
    // evidence: UUID生成は通常失敗しないが、安全のためエラーハンドリングを行う
    let uuid_str = uuid::Uuid::new_v4().to_string();
    // why: split('-')の結果が空でないことを確認してから使用（理論的には空の可能性がある）
    // alt: unwrap_orを使用（理論的には問題ないが、より明示的）
    // evidence: UUIDは常に'-'を含むが、安全性のため明示的にチェック
    let short_uuid = uuid_str.split('-').next()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            // UUIDが予期しない形式の場合のフォールバック
            &uuid_str
        });
    let image_id = format!("img_{page_number}_{short_uuid}");
    
    let new_image = ImageElement {
        id: image_id,
        x,
        y,
        width,
        height,
        original_width: width,
        original_height: height,
        data: image_data,
        format: image_format,
    };
    
    pdf.pages[page_index].images.push(new_image);
    
    Ok(pdf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::pdf_structure::*;

    #[tokio::test]
    async fn test_insert_image() {
        let pdf = PdfStructure {
            pages: vec![Page {
                page_number: 1,
                source_page_number: Some(1),
                width: 612.0,
                height: 792.0,
                images: vec![],
                text_blocks: vec![],
            }],
            metadata: PdfMetadata::default(),
            file_path: "/test/path.pdf".to_string(),
        };
        
        // 最小限のBase64データ（1x1 PNG）
        let base64_data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
        
        let result = insert_image(
            pdf,
            1,
            100.0,
            100.0,
            200.0,
            200.0,
            base64_data.to_string(),
            "png".to_string(),
        ).await;
        assert!(result.is_ok(), "insert_image should succeed");
        let updated = result.expect("result should be Ok");
        assert_eq!(updated.pages[0].images.len(), 1);
        assert_eq!(updated.pages[0].images[0].width, 200.0);
    }
    
    #[tokio::test]
    async fn test_insert_image_invalid_page() {
        let pdf = PdfStructure {
            pages: vec![Page {
                page_number: 1,
                source_page_number: Some(1),
                width: 612.0,
                height: 792.0,
                images: vec![],
                text_blocks: vec![],
            }],
            metadata: PdfMetadata::default(),
            file_path: "/test/path.pdf".to_string(),
        };
        
        let base64_data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
        
        let result = insert_image(
            pdf,
            2, // 無効なページ番号
            100.0,
            100.0,
            200.0,
            200.0,
            base64_data.to_string(),
            "png".to_string(),
        ).await;
        assert!(result.is_err());
    }
}

