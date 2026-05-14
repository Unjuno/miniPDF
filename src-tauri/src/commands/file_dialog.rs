use tauri::command;
use tauri_plugin_dialog::DialogExt;

#[command]
pub async fn open_file_dialog(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let file_path = app
        .dialog()
        .file()
        .add_filter("PDF", &["pdf"])
        .blocking_pick_file();
    
    Ok(file_path.map(|p| p.to_string()))
}

#[cfg(test)]
mod tests {

    // 注意: ファイルダイアログのテストは実際のUI環境が必要なため、
    // 統合テストまたはE2Eテストで実装することを推奨
    #[test]
    fn test_placeholder() {
        // プレースホルダーテスト（実際のテストは統合テストで実装）
    }
}
