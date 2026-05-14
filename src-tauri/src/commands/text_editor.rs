use tauri::command;
use crate::models::pdf_structure::PdfStructure;
use anyhow::Result;

#[command]
pub async fn edit_text_block(
    pdf_structure: PdfStructure,
    text_block_id: String,
    new_text: String,
) -> Result<PdfStructure, String> {
    let mut pdf = pdf_structure;
    
    let mut found = false;
    for page in &mut pdf.pages {
        if let Some(text_block) = page.text_blocks.iter_mut().find(|tb| tb.id == text_block_id) {
            // why: Stringの所有権を移動するため、clone()が必要
            // alt: 参照を使用（ライフタイムの問題が発生する可能性）
            // evidence: text_block.textはString型で所有権が必要
            text_block.text = new_text.clone();
            // テキストの高さを再計算
            let estimated_height = (new_text.lines().count() as f64) * 14.0;
            text_block.height = estimated_height.max(20.0);
            found = true;
            break;
        }
    }
    
    if !found {
        return Err(format!("テキストブロックが見つかりません: {text_block_id}"));
    }
    
    Ok(pdf)
}

#[command]
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
) -> Result<PdfStructure, String> {
    let mut pdf = pdf_structure;
    
    if page_number == 0 || page_number > pdf.pages.len() as u32 {
        return Err(format!("無効なページ番号: {page_number}"));
    }
    
    if font_size <= 0.0 || !(0.5..=2.0).contains(&line_height) {
        return Err(format!("無効なフォント設定: size={font_size}, line_height={line_height}"));
    }
    
    let page_index = (page_number - 1) as usize;
    let page = &pdf.pages[page_index];
    
    // 座標の検証
    if x < 0.0 || y < 0.0 || x + width > page.width || y + height > page.height {
        return Err(format!("テキストブロックの位置がページ範囲外です: ({x}, {y})"));
    }
    
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
    let text_block_id = format!("text_{page_number}_{short_uuid}");
    
    let new_text_block = crate::models::pdf_structure::TextBlock {
        id: text_block_id,
        x,
        y,
        width,
        height: height.max(20.0),
        text,
        font_size,
        line_height,
        font_family,
    };
    
    pdf.pages[page_index].text_blocks.push(new_text_block);
    
    Ok(pdf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::pdf_structure::*;

    #[tokio::test]
    async fn test_edit_text_block() {
        let pdf = PdfStructure {
            pages: vec![Page {
                page_number: 1,
                source_page_number: Some(1),
                width: 612.0,
                height: 792.0,
                images: vec![],
                text_blocks: vec![TextBlock {
                    id: "text-1".to_string(),
                    x: 72.0,
                    y: 720.0,
                    width: 468.0,
                    height: 100.0,
                    text: "Original text".to_string(),
                    font_size: 12.0,
                    line_height: 1.2,
                    font_family: "Arial".to_string(),
                }],
            }],
            metadata: PdfMetadata::default(),
            file_path: "/test/path.pdf".to_string(),
        };
        
        let result = edit_text_block(pdf, "text-1".to_string(), "Edited text".to_string()).await;
        assert!(result.is_ok(), "edit_text_block should succeed");
        let updated = result.expect("result should be Ok");
        assert_eq!(updated.pages[0].text_blocks[0].text, "Edited text");
    }
    
    #[tokio::test]
    async fn test_add_text_block() {
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
        
        let result = add_text_block(
            pdf,
            1,
            72.0,
            720.0,
            468.0,
            50.0,
            "New text".to_string(),
            12.0,
            1.2,
            "Arial".to_string(),
        ).await;
        assert!(result.is_ok(), "add_text_block should succeed");
        let updated = result.expect("result should be Ok");
        assert_eq!(updated.pages[0].text_blocks.len(), 1);
        assert_eq!(updated.pages[0].text_blocks[0].text, "New text");
    }
}

