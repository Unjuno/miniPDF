use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfStructure {
    pub pages: Vec<Page>,
    pub metadata: PdfMetadata,
    #[serde(rename = "filePath")]
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub page_number: u32,
    pub source_page_number: Option<u32>,
    pub width: f64,
    pub height: f64,
    pub images: Vec<ImageElement>,
    pub text_blocks: Vec<TextBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageElement {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub original_width: f64,
    pub original_height: f64,
    pub data: String,
    pub format: ImageFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFormat {
    Jpeg,
    Png,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextBlock {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub text: String,
    pub font_size: f64,
    pub line_height: f64,
    pub font_family: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PdfMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
}

impl PdfStructure {
    /// why: ページ番号の管理を一箇所に集約して不具合を防ぐ
    /// alt: 各操作で個別にページ番号を再割り当て（不整合が発生する可能性）
    /// evidence: 一箇所で管理することで、ページ番号の不整合を防ぐ
    /// assumption: すべてのページ操作後、この関数を呼び出すことでページ番号が正しく管理される
    pub fn renumber_pages(&mut self) {
        for (index, page) in self.pages.iter_mut().enumerate() {
            page.page_number = (index + 1) as u32;
        }
    }

    /// why: ページ番号の検証を一箇所で行う
    /// alt: 各操作で個別に検証（検証ロジックが分散する）
    /// evidence: 一箇所で検証することで、検証ロジックの不整合を防ぐ
    pub fn validate_page_numbers(&self) -> Result<(), String> {
        // why: 空のページリストは有効（空のPDFも生成可能）
        // alt: 空のページリストをエラーとする（空のPDFを生成できない）
        // evidence: 空のPDFも有効なPDFとして生成できる
        if self.pages.is_empty() {
            return Ok(());
        }

        for (index, page) in self.pages.iter().enumerate() {
            let expected_number = (index + 1) as u32;
            if page.page_number != expected_number {
                return Err(format!(
                    "ページ番号が不正です: 期待値 {expected_number}, 実際の値 {}",
                    page.page_number
                ));
            }
        }

        Ok(())
    }
}
