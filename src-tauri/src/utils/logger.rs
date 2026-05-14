/// ログシステム
/// why: 構造化ログでバグを特定しやすくする
/// alt: println!を直接使用（ログレベル制御ができない）
/// evidence: 構造化ログにより、ログレベルでフィルタリングできる

use log::{debug, error, info};

/// why: ログを初期化（環境変数からログレベルを設定）
/// alt: ログを初期化しない（ログが出力されない）
/// evidence: ログを初期化することで、デバッグ時に詳細なログを取得できる
pub fn init_logger() {
    // why: 環境変数からログレベルを設定（開発時はDEBUG、本番時はINFO）
    // alt: ログレベルを固定（デバッグ時に不便）
    // evidence: 環境変数からログレベルを設定することで、実行時に制御できる
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(
        if cfg!(debug_assertions) { "debug" } else { "info" }
    ))
        .format_timestamp_secs()
        .format_module_path(false)
        .format_target(false)
        .init();
}

/// why: エラーログを記録（エラー発生時の詳細情報を記録）
/// alt: エラーログを記録しない（デバッグが困難）
/// evidence: エラーログを記録することで、エラー発生時の状況を把握できる
#[allow(dead_code)]
pub fn log_error(operation: &str, error: &(dyn std::error::Error + 'static)) {
    error!("{} failed: {}", operation, error);
    if let Some(source) = error.source() {
        error!("  Caused by: {}", source);
    }
}

/// why: 文字列エラーログを記録（エラーメッセージのみの場合）
/// alt: エラーログを記録しない（デバッグが困難）
/// evidence: エラーログを記録することで、エラー発生時の状況を把握できる
#[allow(dead_code)]
pub fn log_error_str(operation: &str, error_msg: &str) {
    error!("{} failed: {}", operation, error_msg);
}

/// why: 操作開始ログを記録（処理の開始を記録）
/// alt: 操作開始ログを記録しない（処理の流れが追えない）
/// evidence: 操作開始ログを記録することで、処理の流れを追跡できる
#[allow(dead_code)]
pub fn log_operation_start(operation: &str, context: Option<&str>) {
    if let Some(ctx) = context {
        info!("{} started: {}", operation, ctx);
    } else {
        info!("{} started", operation);
    }
}

/// why: 操作完了ログを記録（処理の完了を記録）
/// alt: 操作完了ログを記録しない（処理の流れが追えない）
/// evidence: 操作完了ログを記録することで、処理の流れを追跡できる
#[allow(dead_code)]
pub fn log_operation_complete(operation: &str, context: Option<&str>) {
    if let Some(ctx) = context {
        debug!("{} completed: {}", operation, ctx);
    } else {
        debug!("{} completed", operation);
    }
}

