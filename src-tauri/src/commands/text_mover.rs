use tauri::command;
use crate::models::pdf_structure::PdfStructure;

#[command]
pub async fn move_text_block(
    pdf_structure: PdfStructure,
    text_block_id: String,
    new_x: f64,
    new_y: f64,
) -> Result<PdfStructure, String> {
    let mut pdf = pdf_structure;
    
    let mut found = false;
    for page in &mut pdf.pages {
        if let Some(text_block) = page.text_blocks.iter_mut().find(|tb| tb.id == text_block_id) {
            // ページ境界チェック
            if new_x < 0.0 || new_y < 0.0 || new_x + text_block.width > page.width || new_y + text_block.height > page.height {
                return Err(format!("テキストブロックがページ範囲外に出ます: テキストブロックID {text_block_id}"));
            }
            
            text_block.x = new_x;
            text_block.y = new_y;
            found = true;
            break;
        }
    }
    
    if !found {
        return Err(format!("テキストブロックが見つかりません: {text_block_id}"));
    }
    
    Ok(pdf)
}

