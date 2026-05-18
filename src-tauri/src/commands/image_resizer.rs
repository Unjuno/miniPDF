use crate::models::pdf_structure::ImageElement;
use anyhow::Context;
use base64::{engine::general_purpose, Engine as _};
use image;
use tauri::command;

#[command]
pub async fn resize_image(
    image_id: String,
    new_width: f64,
    new_height: f64,
    image_data: String,
    format: String,
    x: f64,
    y: f64,
    original_width: f64,
    original_height: f64,
) -> Result<ImageElement, String> {
    let image_bytes = general_purpose::STANDARD
        .decode(&image_data)
        .with_context(|| format!("画像データのBase64デコードに失敗しました (image_id: {image_id})"))
        .map_err(|e| format!("{e}"))?;

    let img = image::load_from_memory(&image_bytes)
        .context("画像データの読み込みに失敗しました")
        .map_err(|e| format!("{e}"))?;
    let resized = img.resize_exact(
        new_width as u32,
        new_height as u32,
        image::imageops::FilterType::Lanczos3,
    );

    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    resized
        .write_to(&mut cursor, image::ImageOutputFormat::Png)
        .with_context(|| format!("画像のリサイズに失敗しました (size: {new_width}x{new_height})"))
        .map_err(|e| format!("{e}"))?;

    let resized_base64 = general_purpose::STANDARD.encode(&buffer);

    let image_format = if format == "jpeg" {
        crate::models::pdf_structure::ImageFormat::Jpeg
    } else {
        crate::models::pdf_structure::ImageFormat::Png
    };

    Ok(ImageElement {
        id: image_id,
        x,
        y,
        width: new_width,
        height: new_height,
        original_width,
        original_height,
        data: resized_base64,
        format: image_format,
    })
}
