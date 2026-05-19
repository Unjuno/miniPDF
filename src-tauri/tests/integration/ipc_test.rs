// 統合テスト: Tauriコマンドの統合動作をテスト

use minipdf::commands;
use minipdf::models::pdf_structure::*;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

// テスト用のヘルパー関数
fn create_test_pdf_structure() -> PdfStructure {
    PdfStructure {
        pages: vec![Page {
            page_number: 1,
            source_page_number: Some(1),
            width: 612.0,
            height: 792.0,
            images: vec![],
            text_blocks: vec![TextBlock {
                id: "text_1".to_string(),
                x: 50.0,
                y: 700.0,
                width: 500.0,
                height: 50.0,
                text: "Test PDF Content".to_string(),
                font_size: 12.0,
                line_height: 1.2,
                font_family: "Arial".to_string(),
            }],
        }],
        metadata: PdfMetadata {
            title: Some("Test PDF".to_string()),
            author: Some("Test Author".to_string()),
            ..Default::default()
        },
        file_path: "/test/path.pdf".to_string(),
    }
}

#[tokio::test]
async fn test_generate_pdf_basic() {
    let pdf_structure = create_test_pdf_structure();

    let result = commands::pdf_generator::generate_pdf(pdf_structure).await;

    assert!(result.is_ok(), "PDF生成は成功する必要があります");
    let pdf_data = result.unwrap();
    assert!(
        !pdf_data.is_empty(),
        "生成されたPDFデータは空であってはなりません"
    );

    // PDFヘッダーを確認（%PDF-1.4）
    assert!(
        pdf_data.starts_with(b"%PDF"),
        "生成されたデータはPDFフォーマットである必要があります"
    );
}

#[tokio::test]
async fn test_generate_pdf_with_metadata() {
    let mut pdf_structure = create_test_pdf_structure();
    pdf_structure.metadata.title = Some("Custom Title".to_string());
    pdf_structure.metadata.author = Some("Custom Author".to_string());
    pdf_structure.metadata.subject = Some("Test Subject".to_string());

    let result = commands::pdf_generator::generate_pdf(pdf_structure).await;

    assert!(result.is_ok());
    let pdf_data = result.unwrap();
    assert!(!pdf_data.is_empty());
}

#[tokio::test]
async fn test_generate_pdf_empty_pages() {
    let pdf_structure = PdfStructure {
        pages: vec![],
        metadata: PdfMetadata::default(),
        file_path: "/test/path.pdf".to_string(),
    };

    let result = commands::pdf_generator::generate_pdf(pdf_structure).await;

    // 空のページリストでもPDFは生成できる
    assert!(result.is_ok());
    let pdf_data = result.unwrap();
    assert!(!pdf_data.is_empty());
}

#[tokio::test]
async fn test_save_pdf_to_file() {
    let temp_dir = TempDir::new().expect("一時ディレクトリの作成に失敗");
    let file_path = temp_dir.path().join("test_output.pdf");
    let file_path_str = file_path.to_string_lossy().to_string();

    let pdf_structure = create_test_pdf_structure();
    let pdf_data = commands::pdf_generator::generate_pdf(pdf_structure)
        .await
        .expect("PDF生成に失敗");

    let result = commands::file_saver::save_pdf(file_path_str.clone(), pdf_data).await;

    assert!(result.is_ok(), "PDFファイルの保存は成功する必要があります");
    assert!(
        file_path.exists(),
        "保存されたファイルが存在する必要があります"
    );

    // ファイルサイズを確認
    let metadata = fs::metadata(&file_path).expect("ファイルメタデータの取得に失敗");
    assert!(
        metadata.len() > 0,
        "保存されたファイルは空であってはなりません"
    );
}

#[tokio::test]
async fn test_save_pdf_invalid_path() {
    let invalid_path = "/nonexistent/directory/file.pdf".to_string();
    let pdf_data = vec![1, 2, 3, 4, 5];

    let result = commands::file_saver::save_pdf(invalid_path, pdf_data).await;

    assert!(
        result.is_err(),
        "存在しないディレクトリへの保存は失敗する必要があります"
    );
}

#[tokio::test]
async fn test_load_pdf_nonexistent() {
    let result = commands::pdf_loader::load_pdf("nonexistent_file.pdf".to_string()).await;

    assert!(
        result.is_err(),
        "存在しないファイルの読み込みは失敗する必要があります"
    );
    let error_msg = result.unwrap_err();
    // anyhowのエラーメッセージは複数のコンテキストを含む可能性があるため、
    // エラーメッセージにファイル名またはエラー情報が含まれていることを確認
    assert!(
        error_msg.contains("PDFファイルを開けませんでした")
            || error_msg.contains("nonexistent_file.pdf")
            || error_msg.contains("PDF")
            || error_msg.to_lowercase().contains("file")
            || error_msg.to_lowercase().contains("not found")
            || error_msg.to_lowercase().contains("no such"),
        "エラーメッセージに適切な情報が含まれている必要があります。実際のエラー: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_resize_image_invalid_base64() {
    let result = commands::image_resizer::resize_image(
        "img-1".to_string(),
        100.0,
        100.0,
        "invalid base64 data".to_string(),
        "png".to_string(),
        0.0,
        0.0,
        200.0,
        200.0,
    )
    .await;

    assert!(
        result.is_err(),
        "無効なBase64データのリサイズは失敗する必要があります"
    );
}

#[tokio::test]
async fn test_adjust_page_break_invalid_page() {
    let pdf_structure = PdfStructure {
        pages: vec![],
        metadata: PdfMetadata::default(),
        file_path: "/test/path.pdf".to_string(),
    };

    let result = commands::page_break::adjust_page_break(pdf_structure, 1, 400.0).await;

    assert!(
        result.is_err(),
        "存在しないページの改ページ調整は失敗する必要があります"
    );
}

#[tokio::test]
async fn test_pdf_generation_and_save_flow() {
    // PDF生成から保存までの完全なフローをテスト
    let temp_dir = TempDir::new().expect("一時ディレクトリの作成に失敗");
    let file_path = temp_dir.path().join("complete_flow_test.pdf");
    let file_path_str = file_path.to_string_lossy().to_string();

    // 1. PDF構造を作成
    let pdf_structure = create_test_pdf_structure();

    // 2. PDFを生成
    let pdf_data = commands::pdf_generator::generate_pdf(pdf_structure)
        .await
        .expect("PDF生成に失敗");

    // 3. PDFを保存
    commands::file_saver::save_pdf(file_path_str.clone(), pdf_data)
        .await
        .expect("PDF保存に失敗");

    // 4. 保存されたファイルを確認
    assert!(
        file_path.exists(),
        "保存されたファイルが存在する必要があります"
    );

    // 5. 保存されたPDFを読み込んで検証（可能であれば）
    // 注意: 実際のPDF読み込みテストには有効なPDFファイルが必要
}

#[tokio::test]
async fn test_generate_pdf_with_multiple_pages() {
    let pdf_structure = PdfStructure {
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

    let result = commands::pdf_generator::generate_pdf(pdf_structure).await;

    assert!(
        result.is_ok(),
        "複数ページのPDF生成は成功する必要があります"
    );
    let pdf_data = result.unwrap();
    assert!(!pdf_data.is_empty());
}

// mian.pdfを使用した実際のPDFファイルのテスト
#[tokio::test]
async fn test_load_mian_pdf() {
    // プロジェクトルートのmian.pdfを読み込む
    let pdf_path = std::env::current_dir()
        .expect("カレントディレクトリの取得に失敗")
        .parent()
        .expect("親ディレクトリの取得に失敗")
        .join("mian.pdf");

    let pdf_path_str = pdf_path.to_string_lossy().to_string();

    // ファイルが存在することを確認
    if !pdf_path.exists() {
        eprintln!("警告: mian.pdfが見つかりません: {}", pdf_path_str);
        return;
    }

    let result = commands::pdf_loader::load_pdf(pdf_path_str.clone()).await;

    // PDF読み込みが成功するか確認
    match result {
        Ok(pdf_structure) => {
            // PDF構造が正しく読み込まれたことを確認
            assert!(
                !pdf_structure.pages.is_empty(),
                "mian.pdfには少なくとも1ページが必要です"
            );
            assert_eq!(
                pdf_structure.file_path, pdf_path_str,
                "ファイルパスが正しく設定されている必要があります"
            );

            // ページ情報を確認
            for (index, page) in pdf_structure.pages.iter().enumerate() {
                assert!(
                    page.width > 0.0,
                    "ページ{}の幅は0より大きい必要があります",
                    index + 1
                );
                assert!(
                    page.height > 0.0,
                    "ページ{}の高さは0より大きい必要があります",
                    index + 1
                );
                assert_eq!(
                    page.page_number,
                    (index + 1) as u32,
                    "ページ番号が正しく設定されている必要があります"
                );
            }

            // メタデータの確認（存在する場合）
            if let Some(ref title) = pdf_structure.metadata.title {
                assert!(!title.is_empty(), "タイトルが空であってはなりません");
            }
        }
        Err(e) => {
            // エラーの詳細を出力
            eprintln!("mian.pdfの読み込みに失敗しました: {}", e);
            panic!("mian.pdfの読み込みは成功する必要があります: {}", e);
        }
    }
}

#[tokio::test]
async fn test_load_and_regenerate_mian_pdf() {
    // mian.pdfを読み込んで、再生成できることを確認
    let pdf_path = std::env::current_dir()
        .expect("カレントディレクトリの取得に失敗")
        .parent()
        .expect("親ディレクトリの取得に失敗")
        .join("mian.pdf");

    let pdf_path_str = pdf_path.to_string_lossy().to_string();

    if !pdf_path.exists() {
        eprintln!("警告: mian.pdfが見つかりません: {}", pdf_path_str);
        return;
    }

    // 1. PDFを読み込む
    let original_pdf_structure = commands::pdf_loader::load_pdf(pdf_path_str.clone())
        .await
        .expect("mian.pdfの読み込みに失敗");

    // メタデータを保存
    let original_metadata = original_pdf_structure.metadata.clone();

    // 2. PDFを再生成
    let result = commands::pdf_generator::generate_pdf(original_pdf_structure).await;

    assert!(result.is_ok(), "mian.pdfの再生成は成功する必要があります");
    let regenerated_pdf = result.unwrap();
    assert!(
        !regenerated_pdf.is_empty(),
        "再生成されたPDFデータは空であってはなりません"
    );
    assert!(
        regenerated_pdf.starts_with(b"%PDF"),
        "再生成されたデータはPDFフォーマットである必要があります"
    );

    // 3. 再生成されたPDFを一時ファイルに保存して読み込み、メタデータが保持されているか確認
    let temp_dir = TempDir::new().expect("一時ディレクトリの作成に失敗");
    let temp_pdf_path = temp_dir.path().join("regenerated.pdf");
    let temp_pdf_path_str = temp_pdf_path.to_string_lossy().to_string();

    commands::file_saver::save_pdf(temp_pdf_path_str.clone(), regenerated_pdf)
        .await
        .expect("再生成PDFの保存に失敗");

    let regenerated_structure = commands::pdf_loader::load_pdf(temp_pdf_path_str)
        .await
        .expect("再生成PDFの読み込みに失敗");

    // メタデータの保持を確認
    if let Some(ref original_title) = original_metadata.title {
        if let Some(ref regenerated_title) = regenerated_structure.metadata.title {
            assert_eq!(
                original_title, regenerated_title,
                "タイトルが保持されている必要があります"
            );
        }
    }
    if let Some(ref original_author) = original_metadata.author {
        if let Some(ref regenerated_author) = regenerated_structure.metadata.author {
            assert_eq!(
                original_author, regenerated_author,
                "著者が保持されている必要があります"
            );
        }
    }
}

#[tokio::test]
async fn test_mian_pdf_images_extraction() {
    // mian.pdfから画像が正しく抽出されることを確認
    let pdf_path = std::env::current_dir()
        .expect("カレントディレクトリの取得に失敗")
        .parent()
        .expect("親ディレクトリの取得に失敗")
        .join("mian.pdf");

    let pdf_path_str = pdf_path.to_string_lossy().to_string();

    if !pdf_path.exists() {
        eprintln!("警告: mian.pdfが見つかりません: {}", pdf_path_str);
        return;
    }

    let pdf_structure = commands::pdf_loader::load_pdf(pdf_path_str)
        .await
        .expect("mian.pdfの読み込みに失敗");

    // 画像が抽出されているか確認（画像が存在する場合）
    let total_images: usize = pdf_structure
        .pages
        .iter()
        .map(|page| page.images.len())
        .sum();

    println!("mian.pdfから抽出された画像数: {}", total_images);

    // 画像が存在する場合、各画像のデータが有効であることを確認
    // 注意: 一部の画像は抽出に失敗する可能性があるため、空のデータは警告として扱う
    for (page_num, page) in pdf_structure.pages.iter().enumerate() {
        for (img_num, image) in page.images.iter().enumerate() {
            assert!(
                !image.id.is_empty(),
                "ページ{}の画像{}のIDは空であってはなりません",
                page_num + 1,
                img_num + 1
            );
            assert!(
                image.width > 0.0,
                "ページ{}の画像{}の幅は0より大きい必要があります",
                page_num + 1,
                img_num + 1
            );
            assert!(
                image.height > 0.0,
                "ページ{}の画像{}の高さは0より大きい必要があります",
                page_num + 1,
                img_num + 1
            );
            if image.data.is_empty() {
                eprintln!(
                    "警告: ページ{}の画像{}のデータが空です（抽出に失敗した可能性があります）",
                    page_num + 1,
                    img_num + 1
                );
            }
        }
    }
}

#[tokio::test]
async fn test_mian_pdf_text_extraction() {
    // mian.pdfからテキストが正しく抽出されることを確認
    let pdf_path = std::env::current_dir()
        .expect("カレントディレクトリの取得に失敗")
        .parent()
        .expect("親ディレクトリの取得に失敗")
        .join("mian.pdf");

    let pdf_path_str = pdf_path.to_string_lossy().to_string();

    if !pdf_path.exists() {
        eprintln!("警告: mian.pdfが見つかりません: {}", pdf_path_str);
        return;
    }

    let pdf_structure = commands::pdf_loader::load_pdf(pdf_path_str)
        .await
        .expect("mian.pdfの読み込みに失敗");

    // テキストブロックが抽出されているか確認
    let total_text_blocks: usize = pdf_structure
        .pages
        .iter()
        .map(|page| page.text_blocks.len())
        .sum();

    println!(
        "mian.pdfから抽出されたテキストブロック数: {}",
        total_text_blocks
    );

    // テキストブロックが存在する場合、各ブロックのデータが有効であることを確認
    for (page_num, page) in pdf_structure.pages.iter().enumerate() {
        for (text_num, text_block) in page.text_blocks.iter().enumerate() {
            assert!(
                !text_block.id.is_empty(),
                "ページ{}のテキストブロック{}のIDは空であってはなりません",
                page_num + 1,
                text_num + 1
            );
            assert!(
                text_block.font_size > 0.0,
                "ページ{}のテキストブロック{}のフォントサイズは0より大きい必要があります",
                page_num + 1,
                text_num + 1
            );
            assert!(
                text_block.line_height > 0.0,
                "ページ{}のテキストブロック{}の行間は0より大きい必要があります",
                page_num + 1,
                text_num + 1
            );
        }
    }
}

#[tokio::test]
async fn test_mian_pdf_full_workflow() {
    // mian.pdfの完全なワークフローをテスト（読み込み→編集→保存）
    let pdf_path = std::env::current_dir()
        .expect("カレントディレクトリの取得に失敗")
        .parent()
        .expect("親ディレクトリの取得に失敗")
        .join("mian.pdf");

    let pdf_path_str = pdf_path.to_string_lossy().to_string();

    if !pdf_path.exists() {
        eprintln!("警告: mian.pdfが見つかりません: {}", pdf_path_str);
        return;
    }

    // 1. PDFを読み込む
    let pdf_structure = commands::pdf_loader::load_pdf(pdf_path_str.clone())
        .await
        .expect("mian.pdfの読み込みに失敗");

    // 2. 編集後のPDFを生成
    let result = commands::pdf_generator::generate_pdf(pdf_structure).await;

    assert!(
        result.is_ok(),
        "編集後のmian.pdfの生成は成功する必要があります"
    );
    let edited_pdf = result.unwrap();
    assert!(
        !edited_pdf.is_empty(),
        "編集後のPDFデータは空であってはなりません"
    );

    // 4. 一時ファイルに保存
    let temp_dir = TempDir::new().expect("一時ディレクトリの作成に失敗");
    let output_path = temp_dir.path().join("mian_edited.pdf");
    let output_path_str = output_path.to_string_lossy().to_string();

    let save_result = commands::file_saver::save_pdf(output_path_str.clone(), edited_pdf).await;
    assert!(
        save_result.is_ok(),
        "編集後のmian.pdfの保存は成功する必要があります"
    );
    assert!(
        output_path.exists(),
        "保存されたファイルが存在する必要があります"
    );
}

#[tokio::test]
async fn test_mian_pdf_layout_issues() {
    // mian.pdfのレイアウト問題を診断するテスト
    let pdf_path = std::env::current_dir()
        .expect("カレントディレクトリの取得に失敗")
        .parent()
        .expect("親ディレクトリの取得に失敗")
        .join("mian.pdf");

    let pdf_path_str = pdf_path.to_string_lossy().to_string();

    if !pdf_path.exists() {
        eprintln!("警告: mian.pdfが見つかりません: {}", pdf_path_str);
        return;
    }

    let pdf_structure = commands::pdf_loader::load_pdf(pdf_path_str)
        .await
        .expect("mian.pdfの読み込みに失敗");

    // 問題1: テキストブロックの位置情報がすべて0.0になっている
    let mut zero_position_text_blocks = 0;
    let mut total_text_blocks = 0;

    for (page_num, page) in pdf_structure.pages.iter().enumerate() {
        for text_block in &page.text_blocks {
            total_text_blocks += 1;
            if text_block.x == 0.0
                && text_block.y == 0.0
                && text_block.width == 0.0
                && text_block.height == 0.0
            {
                zero_position_text_blocks += 1;
                println!(
                    "警告: ページ{}のテキストブロック{}の位置情報がすべて0.0です",
                    page_num + 1,
                    text_block.id
                );
            }
        }
    }

    println!("総テキストブロック数: {}", total_text_blocks);
    println!(
        "位置情報が0.0のテキストブロック数: {}",
        zero_position_text_blocks
    );

    // 問題2: 画像の位置情報がすべて0.0になっている
    let mut zero_position_images = 0;
    let mut total_images = 0;

    for (page_num, page) in pdf_structure.pages.iter().enumerate() {
        for image in &page.images {
            total_images += 1;
            if image.x == 0.0 && image.y == 0.0 {
                zero_position_images += 1;
                println!(
                    "警告: ページ{}の画像{}の位置情報が0.0です",
                    page_num + 1,
                    image.id
                );
            }
        }
    }

    println!("総画像数: {}", total_images);
    println!("位置情報が0.0の画像数: {}", zero_position_images);

    // 問題3: フォント情報が固定値になっている
    let mut default_font_text_blocks = 0;
    for page in &pdf_structure.pages {
        for text_block in &page.text_blocks {
            if text_block.font_size == 12.0
                && text_block.font_family == "Arial"
                && text_block.line_height == 1.2
            {
                default_font_text_blocks += 1;
            }
        }
    }

    println!(
        "デフォルトフォント情報のテキストブロック数: {}",
        default_font_text_blocks
    );

    // 問題の診断結果を出力
    if zero_position_text_blocks > 0 {
        eprintln!(
            "問題: {}個のテキストブロックの位置情報が失われています",
            zero_position_text_blocks
        );
    }
    if zero_position_images > 0 {
        eprintln!(
            "問題: {}個の画像の位置情報が失われています",
            zero_position_images
        );
    }
    if default_font_text_blocks == total_text_blocks && total_text_blocks > 0 {
        eprintln!("問題: すべてのテキストブロックがデフォルトフォント情報になっています");
    }

    // これらの問題がある場合、PDF再生成時にレイアウトが崩れる
    if zero_position_text_blocks > 0 || zero_position_images > 0 {
        eprintln!(
            "警告: 位置情報が失われているため、PDF再生成時にレイアウトが崩れる可能性があります"
        );
    }
}

#[tokio::test]
async fn test_markdown_preview_cli_smoke() {
    let temp_dir = TempDir::new().expect("一時ディレクトリの作成に失敗");
    let input_path = temp_dir.path().join("visual-check.md");
    let output_path = temp_dir.path().join("visual-check.preview.pdf");
    let markdown = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/markdown-renderer-visual-check.md"
    ));
    fs::write(&input_path, markdown).expect("入力Markdownの書き込みに失敗");

    let bin = env!("CARGO_BIN_EXE_markdown_preview_cli");
    let status = Command::new(bin)
        .arg(&input_path)
        .arg(&output_path)
        .status()
        .expect("CLIの起動に失敗");

    assert!(status.success(), "CLIは成功終了する必要があります");
    assert!(output_path.exists(), "出力PDFが生成される必要があります");

    let loaded = commands::pdf_loader::load_pdf(output_path.to_string_lossy().to_string())
        .await
        .expect("生成PDFの読み込みに失敗");
    let all_text = loaded
        .pages
        .iter()
        .flat_map(|page| page.text_blocks.iter().map(|block| block.text.as_str()))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(all_text.contains("Unicode / 日本語 / 絵文字"), "{all_text}");
    assert!(all_text.contains("終端確認"), "{all_text}");
    assert!(all_text.contains("XSS script tag"), "{all_text}");
}

#[tokio::test]
async fn test_markdown_preview_npm_script_smoke() {
    let repo_root = std::env::current_dir()
        .expect("カレントディレクトリの取得に失敗")
        .parent()
        .expect("リポジトリルートの取得に失敗")
        .to_path_buf();
    let temp_dir = TempDir::new().expect("一時ディレクトリの作成に失敗");
    let input_path = temp_dir.path().join("visual-check.md");
    let output_path = temp_dir.path().join("visual-check.preview.pdf");
    let markdown = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/markdown-renderer-visual-check.md"
    ));
    fs::write(&input_path, markdown).expect("入力Markdownの書き込みに失敗");

    let npm_command = if cfg!(windows) { "npm.cmd" } else { "npm" };
    let status = Command::new(npm_command)
        .current_dir(&repo_root)
        .args([
            "run",
            "markdown:preview",
            "--",
            input_path.to_string_lossy().as_ref(),
            output_path.to_string_lossy().as_ref(),
        ])
        .status()
        .expect("npm scriptの起動に失敗");

    assert!(status.success(), "npm scriptは成功終了する必要があります");
    assert!(output_path.exists(), "出力PDFが生成される必要があります");

    let loaded = commands::pdf_loader::load_pdf(output_path.to_string_lossy().to_string())
        .await
        .expect("生成PDFの読み込みに失敗");
    let all_text = loaded
        .pages
        .iter()
        .flat_map(|page| page.text_blocks.iter().map(|block| block.text.as_str()))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        all_text.contains("Markdown Renderer Visual Check"),
        "{all_text}"
    );
    assert!(all_text.contains("終端確認"), "{all_text}");
}

#[test]
fn test_cargo_toml_sets_default_run_binary() {
    let cargo_toml = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("Cargo.tomlの読み込みに失敗");

    assert!(
        cargo_toml.contains("default-run = \"minipdf\""),
        "Cargo.toml should define the main binary for cargo run"
    );
}
