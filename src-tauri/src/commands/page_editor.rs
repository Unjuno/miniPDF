use tauri::command;
use crate::models::pdf_structure::PdfStructure;
use anyhow::Result;

#[command]
pub async fn add_page(
    pdf_structure: PdfStructure,
    page_number: u32,
    width: f64,
    height: f64,
) -> Result<PdfStructure, String> {
    let mut pdf = pdf_structure;
    
    if page_number == 0 || page_number > pdf.pages.len() as u32 + 1 {
        return Err(format!("無効なページ番号: {page_number}"));
    }
    
    if width <= 0.0 || height <= 0.0 {
        return Err(format!("ページサイズが無効です: {width}x{height}"));
    }
    
    let new_page = crate::models::pdf_structure::Page {
        page_number,
        source_page_number: None,
        width,
        height,
        images: vec![],
        text_blocks: vec![],
    };
    
    let insert_index = (page_number - 1) as usize;
    pdf.pages.insert(insert_index, new_page);
    
    // why: ページ番号の管理を一箇所に集約（renumber_pagesメソッドを使用）
    // alt: 個別にページ番号を再割り当て（不整合が発生する可能性）
    // evidence: 一箇所で管理することで、ページ番号の不整合を防ぐ
    pdf.renumber_pages();
    
    Ok(pdf)
}

#[command]
pub async fn delete_page(
    pdf_structure: PdfStructure,
    page_number: u32,
) -> Result<PdfStructure, String> {
    let mut pdf = pdf_structure;
    
    if page_number == 0 || page_number > pdf.pages.len() as u32 {
        return Err(format!("無効なページ番号: {page_number}"));
    }
    
    if pdf.pages.len() <= 1 {
        return Err("最後のページは削除できません".to_string());
    }
    
    let remove_index = (page_number - 1) as usize;
    pdf.pages.remove(remove_index);
    
    // why: ページ番号の管理を一箇所に集約（renumber_pagesメソッドを使用）
    // alt: 個別にページ番号を再割り当て（不整合が発生する可能性）
    // evidence: 一箇所で管理することで、ページ番号の不整合を防ぐ
    pdf.renumber_pages();
    
    Ok(pdf)
}

#[command]
pub async fn reorder_pages(
    pdf_structure: PdfStructure,
    from_index: u32,
    to_index: u32,
) -> Result<PdfStructure, String> {
    let mut pdf = pdf_structure;
    
    let page_count = pdf.pages.len() as u32;
    if from_index == 0 || from_index > page_count || to_index == 0 || to_index > page_count {
        return Err(format!("無効なページインデックス: {from_index} -> {to_index}"));
    }
    
    if from_index == to_index {
        return Ok(pdf);
    }
    
    let from_idx = (from_index - 1) as usize;
    let to_idx = (to_index - 1) as usize;
    
    // why: removeとinsertの順序を考慮して、正しい位置に移動
    // alt: 常にremoveを先に実行（from_idx < to_idxの場合にto_idxがずれる）
    // evidence: from_idx < to_idxの場合、removeを先に実行するとto_idxが1つずれるため、調整が必要
    let page = pdf.pages.remove(from_idx);
    // why: from_idx < to_idxの場合、remove後にto_idxを調整
    // alt: to_idxを調整しない（間違った位置に挿入される）
    // evidence: removeを先に実行すると、from_idxより後のインデックスが1つずれる
    let adjusted_to_idx = if from_idx < to_idx {
        to_idx - 1
    } else {
        to_idx
    };
    pdf.pages.insert(adjusted_to_idx, page);
    
    // why: ページ番号の管理を一箇所に集約（renumber_pagesメソッドを使用）
    // alt: 個別にページ番号を再割り当て（不整合が発生する可能性）
    // evidence: 一箇所で管理することで、ページ番号の不整合を防ぐ
    pdf.renumber_pages();
    
    Ok(pdf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::pdf_structure::*;

    #[tokio::test]
    async fn test_add_page() {
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
        
        let result = add_page(pdf, 2, 612.0, 792.0).await;
        assert!(result.is_ok(), "add_page should succeed");
        let updated = result.expect("result should be Ok");
        assert_eq!(updated.pages.len(), 2);
        assert_eq!(updated.pages[1].page_number, 2);
    }
    
    #[tokio::test]
    async fn test_delete_page() {
        let pdf = PdfStructure {
            pages: vec![
                Page {
                    page_number: 1,
                    source_page_number: Some(1),
                    width: 612.0,
                    height: 792.0,
                    images: vec![],
                    text_blocks: vec![],
                },
                Page {
                    page_number: 2,
                    source_page_number: Some(2),
                    width: 612.0,
                    height: 792.0,
                    images: vec![],
                    text_blocks: vec![],
                },
            ],
            metadata: PdfMetadata::default(),
            file_path: "/test/path.pdf".to_string(),
        };
        
        let result = delete_page(pdf, 1).await;
        assert!(result.is_ok(), "delete_page should succeed");
        let updated = result.expect("result should be Ok");
        assert_eq!(updated.pages.len(), 1);
        assert_eq!(updated.pages[0].page_number, 1);
    }
    
    #[tokio::test]
    async fn test_reorder_pages() {
        let pdf = PdfStructure {
            pages: vec![
                Page {
                    page_number: 1,
                    source_page_number: Some(1),
                    width: 612.0,
                    height: 792.0,
                    images: vec![],
                    text_blocks: vec![],
                },
                Page {
                    page_number: 2,
                    source_page_number: Some(2),
                    width: 612.0,
                    height: 792.0,
                    images: vec![],
                    text_blocks: vec![],
                },
            ],
            metadata: PdfMetadata::default(),
            file_path: "/test/path.pdf".to_string(),
        };
        
        let result = reorder_pages(pdf, 1, 2).await;
        assert!(result.is_ok(), "reorder_pages should succeed");
        let updated = result.expect("result should be Ok");
        assert_eq!(updated.pages[0].page_number, 1);
        assert_eq!(updated.pages[1].page_number, 2);
    }
}

