fn main() {
    // why: テスト実行時にはアイコンファイルのビルドエラーを回避するため
    // alt: アイコンファイルを正しいフォーマットに変換する（本番ビルド時）
    // evidence: cargo test実行時にicon.icoのフォーマットエラーが発生する
    // assumption: テスト実行時にはSKIP_TAURI_BUILD環境変数を設定する
    if std::env::var("SKIP_TAURI_BUILD").is_ok() {
        println!("cargo:warning=Skipping Tauri build for tests");
        return;
    }

    tauri_build::build();
}
