use anyhow::Context;
use std::fs;
use tauri::command;
use tauri_plugin_dialog::DialogExt;

#[command]
pub async fn save_file_dialog(
    app: tauri::AppHandle,
    target: String,
) -> Result<Option<String>, String> {
    let preset = save_dialog_preset(&target);
    let file_path = app
        .dialog()
        .file()
        .set_file_name(preset.file_name)
        .add_filter(preset.filter_label, preset.filter_extensions)
        .blocking_save_file();

    Ok(file_path.map(|p| p.to_string()))
}

#[command]
pub async fn save_pdf(file_path: String, pdf_data: Vec<u8>) -> Result<(), String> {
    fs::write(&file_path, pdf_data)
        .with_context(|| format!("PDFファイルの保存に失敗しました: {file_path}"))
        .map_err(|e| format!("{e}"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SaveDialogPreset {
    file_name: &'static str,
    filter_label: &'static str,
    filter_extensions: &'static [&'static str],
}

fn save_dialog_preset(target: &str) -> SaveDialogPreset {
    match target {
        "markdown" => SaveDialogPreset {
            file_name: "output.md",
            filter_label: "Markdown",
            filter_extensions: &["md"],
        },
        _ => SaveDialogPreset {
            file_name: "output.pdf",
            filter_label: "PDF",
            filter_extensions: &["pdf"],
        },
    }
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

    #[test]
    fn test_save_dialog_preset_for_markdown() {
        let preset = save_dialog_preset("markdown");
        assert_eq!(preset.file_name, "output.md");
        assert_eq!(preset.filter_label, "Markdown");
        assert_eq!(preset.filter_extensions, &["md"]);
    }

    #[test]
    fn test_save_dialog_preset_defaults_to_pdf() {
        let preset = save_dialog_preset("pdf");
        assert_eq!(preset.file_name, "output.pdf");
        assert_eq!(preset.filter_label, "PDF");
        assert_eq!(preset.filter_extensions, &["pdf"]);
    }
}
