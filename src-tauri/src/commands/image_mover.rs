use tauri::command;
use crate::models::pdf_structure::PdfStructure;

#[command]
pub async fn move_image(
    pdf_structure: PdfStructure,
    image_id: String,
    new_x: f64,
    new_y: f64,
) -> Result<PdfStructure, String> {
    let mut pdf = pdf_structure;
    
    let mut found = false;
    for page in &mut pdf.pages {
        if let Some(image) = page.images.iter_mut().find(|img| img.id == image_id) {
            // ページ境界チェック
            if new_x < 0.0 || new_y < 0.0 || new_x + image.width > page.width || new_y + image.height > page.height {
                return Err(format!("画像がページ範囲外に出ます: 画像ID {image_id}"));
            }
            
            image.x = new_x;
            image.y = new_y;
            found = true;
            break;
        }
    }
    
    if !found {
        return Err(format!("画像が見つかりません: {image_id}"));
    }
    
    Ok(pdf)
}

