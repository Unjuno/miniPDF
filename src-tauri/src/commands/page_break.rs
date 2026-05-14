use tauri::command;
use crate::models::pdf_structure::PdfStructure;

#[command]
pub async fn adjust_page_break(
    pdf_structure: PdfStructure,
    page_number: u32,
    new_position: f64,
) -> Result<PdfStructure, String> {
    let mut pdf = pdf_structure;
    
    if page_number == 0 || page_number > pdf.pages.len() as u32 {
        return Err(format!("無効なページ番号: {page_number}"));
    }

    let page_index = (page_number - 1) as usize;
    let page = &pdf.pages[page_index];
    
    // 最小ページ高さの制約（ページサイズの10%）
    let min_page_height = page.height * 0.1;
    if new_position < min_page_height || new_position > page.height - min_page_height {
        return Err(format!("改ページ位置が範囲外です。最小ページ高さ: {min_page_height:.1}pt, 最大: {:.1}pt", 
            page.height - min_page_height));
    }

    // 改ページ位置を調整（現在のページの高さを変更）
    // why: 改ページ位置の調整は、ページの高さを変更することで実現
    // alt: ページ分割・結合（複雑な処理が必要）
    // evidence: 現在の実装では、ページ高さの変更で改ページ位置を調整
    // assumption: ページ内のコンテンツは改ページ位置に応じて自動的に調整される
    pdf.pages[page_index].height = new_position;

    // ページ内のコンテンツがページ外に出ないようにチェック
    let page = &pdf.pages[page_index];
    for image in &page.images {
        if image.y + image.height > page.height {
            return Err(format!("画像がページ範囲外に出ます: 画像ID {image_id}", image_id = image.id));
        }
    }
    
    for text_block in &page.text_blocks {
        if text_block.y + text_block.height > page.height {
            return Err(format!("テキストブロックがページ範囲外に出ます: テキストブロックID {text_block_id}", text_block_id = text_block.id));
        }
    }

    Ok(pdf)
}
