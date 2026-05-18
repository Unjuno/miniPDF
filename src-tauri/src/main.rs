#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;
mod utils;

fn main() {
    // why: ログシステムを初期化（バグ特定のため）
    // alt: ログシステムを初期化しない（デバッグが困難）
    // evidence: ログシステムにより、エラー発生時の状況を把握できる
    utils::logger::init_logger();
    log::info!("miniPDF application starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::file_dialog::open_file_dialog,
            commands::pdf_loader::load_pdf,
            commands::image_resizer::resize_image,
            commands::pdf_generator::generate_pdf,
            commands::file_saver::save_file_dialog,
            commands::file_saver::save_pdf,
            commands::page_break::adjust_page_break,
            commands::page_editor::add_page,
            commands::page_editor::delete_page,
            commands::page_editor::reorder_pages,
            commands::text_editor::edit_text_block,
            commands::text_editor::add_text_block,
            commands::image_inserter::insert_image,
            commands::image_mover::move_image,
            commands::text_mover::move_text_block,
            commands::markdown_preview::render_markdown_to_pdf_preview,
            commands::markdown_preview::render_markdown_to_html_preview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
