use tauri::command;
use tauri_plugin_dialog::DialogExt;
use std::fs;
use anyhow::Context;

#[command]
pub async fn save_file_dialog(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let file_path = app
        .dialog()
        .file()
        .set_file_name("output.pdf")
        .add_filter("PDF", &["pdf"])
        .blocking_save_file();
    
    Ok(file_path.map(|p| p.to_string()))
}

#[command]
pub async fn save_pdf(file_path: String, pdf_data: Vec<u8>) -> Result<(), String> {
    fs::write(&file_path, pdf_data)
        .with_context(|| format!("PDFファイルの保存に失敗しました: {file_path}"))
        .map_err(|e| format!("{e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_save_pdf_invalid_path() {
        // 存在しないディレクトリへの保存は失敗する
        let result = save_pdf("/nonexistent/dir/file.pdf".to_string(), vec![1, 2, 3]).await;
        assert!(result.is_err());
    }
}
