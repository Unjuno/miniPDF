use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex, OnceLock};

use base64::{engine::general_purpose, Engine as _};
use comrak::nodes::{AstNode, ListType, NodeCode, NodeCodeBlock, NodeMath, NodeValue};
use comrak::{parse_document, Arena, Options};
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageOutputFormat, Rgb};
use oxidize_pdf::{Color, Document, Font, Image, Page as OxidizePage};
use serde::{Deserialize, Serialize};
use swash::scale::{image::Content as SwashContent, Render, ScaleContext, Source, StrikeWith};
use swash::zeno::Format;
use swash::FontRef;
use tauri::command;
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

use crate::utils::font_manager;

const PAGE_WIDTH: f64 = 595.0;
const PAGE_HEIGHT: f64 = 842.0;
const MARGIN: f64 = 40.0;
const BASE_LINE_HEIGHT: f64 = 16.0;
/// Upper bound for consecutive blank source lines rendered as vertical space between blocks.
/// why: test fixtures may contain huge blank runs; a cap prevents runaway PDF height while still
///      allowing intentional blank lines for page-boundary tuning (~1 page of slack at 16pt/line).
const MAX_SPACER_LINES: usize = 48;
const MERMAID_MAX_DISPLAY_HEIGHT: f64 = 260.0;
const MERMAID_PT_PER_PX: f64 = 0.75;
const MATH_INLINE_Y_OFFSET_PT: f64 = -3.0;
const MATH_INLINE_EXTRA_LINE_PT: f64 = 8.0;
const JP_SANS_FONT: &str = "NotoSansJP";
const JP_SANS_BOLD: &str = "NotoSansJP-Bold";
const EMOJI_FONT: &str = "Emoji";
const LIST_MARKER_X: f64 = MARGIN + 4.0;
const LIST_INDENT_STEP_PT: f64 = 11.0;
const LIST_MARKER_TEXT_GAP_PT: f64 = 8.0;
const EMOJI_RASTER_PX_PER_PT: f32 = 6.0;
const EMOJI_DISPLAY_SCALE: f64 = 1.45;
// PDF の座標は pt なので、いったん追加オフセットなしに戻す。
const EMOJI_VERTICAL_ADJUST_PT: f64 = 0.0;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct TextStyle {
    bold: bool,
    italic: bool,
    code: bool,
    link: bool,
    math: bool,
}

#[derive(Debug, Clone)]
struct InlineSpan {
    text: String,
    style: TextStyle,
}

impl InlineSpan {
    fn push(out: &mut Vec<InlineSpan>, text: &str, style: TextStyle) {
        if text.is_empty() {
            return;
        }
        if let Some(last) = out.last_mut() {
            if last.style == style {
                last.text.push_str(text);
                return;
            }
        }
        out.push(InlineSpan {
            text: text.to_string(),
            style,
        });
    }
}

#[derive(Debug, Clone, Copy)]
struct BlockSource {
    start_line: u32,
    end_line: u32,
}

impl BlockSource {
    fn single(line: u32) -> Self {
        Self {
            start_line: line,
            end_line: line,
        }
    }

    fn from_node<'a>(node: &'a AstNode<'a>) -> Self {
        let pos = node.data.borrow().sourcepos;
        Self {
            start_line: pos.start.line.max(1) as u32,
            end_line: pos.end.line.max(1) as u32,
        }
    }
}

#[derive(Debug, Clone)]
struct LayoutLine {
    spans: Vec<InlineSpan>,
    source_line: u32,
}

#[derive(Debug)]
struct PreviewLayoutState {
    page_index: u32,
    line_pages: Vec<u32>,
}

impl PreviewLayoutState {
    fn new(source_line_count: usize) -> Self {
        Self {
            page_index: 1,
            line_pages: vec![1; source_line_count.max(1)],
        }
    }

    fn record_line(&mut self, line: u32) {
        let idx = line.saturating_sub(1) as usize;
        if idx < self.line_pages.len() {
            self.line_pages[idx] = self.page_index;
        }
    }

    fn record_source(&mut self, source: BlockSource) {
        for line in source.start_line..=source.end_line {
            self.record_line(line);
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownPreviewResult {
    pub file_path: String,
    pub line_page_map: Vec<u32>,
}

#[derive(Debug, Clone)]
enum MarkdownBlock {
    Heading {
        level: usize,
        spans: Vec<InlineSpan>,
        source: BlockSource,
    },
    Paragraph {
        spans: Vec<InlineSpan>,
        source: BlockSource,
    },
    /// Markdown の空行をページレイアウトに反映するための余白ブロック。
    Spacer {
        lines: usize,
        source: BlockSource,
    },
    /// ネストした箇条書きは `indent` を増やしてフラット化する。
    ListItem {
        indent: u8,
        spans: Vec<InlineSpan>,
        source: BlockSource,
    },
    OrderedListItem {
        n: u32,
        indent: u8,
        spans: Vec<InlineSpan>,
        source: BlockSource,
    },
    /// `---` / `***` / `___` などの区切り線。
    ThematicBreak {
        style: ThematicBreakStyle,
        source: BlockSource,
    },
    /// 各行はセルごとのインライン列（列数は行ごとに揃う想定）
    Table {
        rows: Vec<Vec<Vec<InlineSpan>>>,
        source: BlockSource,
    },
    CodeBlock {
        lang: String,
        code: String,
        source: BlockSource,
    },
    DisplayMath {
        expr: String,
        source: BlockSource,
    },
    Blockquote {
        lines: Vec<BlockquoteLine>,
        source: BlockSource,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThematicBreakStyle {
    Hyphen,
    Asterisk,
    Underscore,
}

#[derive(Debug, Clone)]
enum BlockquoteLine {
    Text(Vec<InlineSpan>),
    DisplayMath(String),
    CodeBlock { code: String },
}

#[derive(Debug, Clone)]
struct MathRenderImage {
    jpeg_data: Vec<u8>,
    width_pt: f64,
    height_pt: f64,
    baseline_pt: f64,
}

#[derive(Debug, Default)]
struct MathRenderCache {
    entries: HashMap<(String, bool), Option<MathRenderImage>>,
}

impl MathRenderCache {
    fn render(&mut self, expr: &str, display_mode: bool) -> Option<MathRenderImage> {
        let key = (expr.to_string(), display_mode);
        if let Some(cached) = self.entries.get(&key) {
            return cached.clone();
        }
        let rendered = render_math_with_katex(expr, display_mode).ok().flatten();
        self.entries.insert(key, rendered.clone());
        rendered
    }
}

#[derive(Debug, Deserialize)]
struct MathRenderResponse {
    #[serde(rename = "width")]
    _width: f64,
    #[serde(rename = "height")]
    _height: f64,
    #[serde(rename = "baselineOffset")]
    baseline_offset: f64,
    #[serde(rename = "devicePixelRatio")]
    device_pixel_ratio: f64,
    png: String,
}

fn scale_math_image(rendered: &MathRenderImage, max_height_pt: f64) -> (f64, f64) {
    let scale = if rendered.height_pt > 0.0 {
        (max_height_pt / rendered.height_pt).min(1.0)
    } else {
        1.0
    };
    (rendered.width_pt * scale, rendered.height_pt * scale)
}

#[command]
pub async fn render_markdown_to_pdf_preview(
    markdown: String,
) -> Result<MarkdownPreviewResult, String> {
    let output_path = build_preview_path();
    let normalized = normalize_markdown_newlines(markdown.as_str());
    let source_line_count = normalized.lines().count().max(1);
    let blocks = parse_markdown_blocks(&normalized);
    let (bytes, line_page_map) = build_preview_pdf(&blocks, source_line_count)?;

    fs::write(&output_path, &bytes)
        .map_err(|e| format!("プレビューPDFの書き込みに失敗しました: {e}"))?;

    Ok(MarkdownPreviewResult {
        file_path: output_path.to_string_lossy().to_string(),
        line_page_map,
    })
}

pub fn render_markdown_preview_pdf_bytes(markdown: &str) -> Result<Vec<u8>, String> {
    let normalized = normalize_markdown_newlines(markdown);
    let source_line_count = normalized.lines().count().max(1);
    let blocks = parse_markdown_blocks(&normalized);
    build_preview_pdf(&blocks, source_line_count).map(|(bytes, _)| bytes)
}

#[command]
pub async fn render_markdown_to_html_preview(markdown: String) -> Result<String, String> {
    let blocks = parse_markdown_blocks(&markdown);
    Ok(build_preview_html(&blocks))
}

fn build_preview_path() -> PathBuf {
    std::env::temp_dir().join(format!("miniPDF-preview-{}.pdf", Uuid::new_v4()))
}

fn build_preview_html(blocks: &[MarkdownBlock]) -> String {
    let mut pages: Vec<Vec<String>> = vec![Vec::new()];
    let mut current_height = 0.0_f64;
    let page_limit = 732.0_f64;

    for block in blocks {
        match block {
            MarkdownBlock::Blockquote { lines, .. } => {
                for (html, est_h) in render_blockquote_html_chunks(lines, page_limit) {
                    if !pages.last().unwrap().is_empty() && current_height + est_h > page_limit {
                        pages.push(Vec::new());
                        current_height = 0.0;
                    }
                    pages.last_mut().unwrap().push(html);
                    current_height += est_h;
                }
            }
            _ => {
                let (html, est_h) = render_block_to_html(block);
                if !pages.last().unwrap().is_empty() && current_height + est_h > page_limit {
                    pages.push(Vec::new());
                    current_height = 0.0;
                }
                pages.last_mut().unwrap().push(html);
                current_height += est_h;
            }
        }
    }

    let mut out = String::new();
    out.push_str("<style>");
    out.push_str(r#"
      @page { size: A4; margin: 0; }
      :root {
        --page-w: 595pt;
        --page-h: 842pt;
        --page-pad: 40pt;
        --ink: #1d2330;
        --muted: #5e6472;
        --bg: #d7d7d7;
        --paper: #fff;
      }
      * { box-sizing: border-box; }
      html, body { margin: 0; padding: 0; color: var(--ink); font-family: "Noto Sans JP", "Yu Gothic UI", "Hiragino Sans", sans-serif; }
      body { background: var(--bg); padding: 16pt 0 32pt; }
      .page {
        width: var(--page-w);
        min-height: var(--page-h);
        margin: 0 auto 16pt;
        padding: var(--page-pad);
        background: var(--paper);
        box-shadow: 0 10px 24px rgba(0,0,0,.16);
        overflow: hidden;
        break-after: page;
        page-break-after: always;
      }
      .page:last-child { break-after: auto; page-break-after: auto; }
      h1 { font-size: 24pt; font-weight: 700; margin: 0 0 14pt; line-height: 1.2; }
      h2 { font-size: 20pt; font-weight: 700; margin: 18pt 0 12pt; line-height: 1.25; }
      h3 { font-size: 18pt; font-weight: 700; margin: 14pt 0 10pt; line-height: 1.25; }
      p { margin: 0 0 10pt; line-height: 1.5; font-size: 11.5pt; }
      ul, ol { margin: 0 0 10pt 1.4em; padding: 0; line-height: 1.5; font-size: 11.5pt; }
      li { margin: 0 0 3pt; }
      blockquote {
        margin: 0 0 10pt;
        padding: 10pt 12pt;
        border-left: 6px solid #7d92cf;
        background: #f5f7fc;
        line-height: 1.45;
        font-size: 11.5pt;
        overflow-wrap: anywhere;
        word-break: break-word;
        break-inside: avoid;
        page-break-inside: avoid;
      }
      blockquote .blockquote-line {
        margin: 0 0 4pt;
        overflow-wrap: anywhere;
        word-break: break-word;
      }
      blockquote .blockquote-line:last-child {
        margin-bottom: 0;
      }
      blockquote .blockquote-math {
        margin: 6pt 0;
        text-align: center;
        font-family: "Cambria Math", "Times New Roman", serif;
        font-style: italic;
        overflow-wrap: anywhere;
        word-break: break-word;
      }
      blockquote .blockquote-code {
        margin: 6pt 0;
        padding: 8pt 10pt;
        background: #eef2f8;
        border: 1px solid #d7deea;
        border-radius: 6px;
        font-family: "Cascadia Mono", "Consolas", monospace;
        white-space: pre-wrap;
        overflow-wrap: anywhere;
        word-break: break-word;
      }
      strong { font-weight: 700; }
      em { font-style: italic; }
      .link { color: #2156d1; text-decoration: underline; }
      .math { font-family: "Cambria Math", "Times New Roman", serif; font-style: italic; }
      .math-block {
        margin: 0 0 10pt;
        padding: 12pt 14pt;
        background: #f6f8ff;
        border: 1px solid #d7def1;
        border-radius: 8px;
        font-family: "Cambria Math", "Times New Roman", serif;
      }
      code {
        font-family: "Cascadia Mono", "Consolas", monospace;
        background: #eef2f8;
        padding: 0.12em 0.28em;
        border-radius: 4px;
      }
      pre {
        margin: 0 0 10pt;
        padding: 10pt 12pt;
        background: #f4f6fa;
        border: 1px solid #dde3ef;
        border-radius: 8px;
        white-space: pre-wrap;
        word-break: break-word;
        break-inside: avoid;
        page-break-inside: avoid;
      }
      pre code { background: transparent; padding: 0; }
      table { width: 100%; border-collapse: collapse; margin: 0 0 10pt; break-inside: avoid; page-break-inside: avoid; font-size: 10.5pt; }
      thead { display: table-header-group; }
      tbody { display: table-row-group; }
      tr { break-inside: avoid; page-break-inside: avoid; }
      th, td { border: 1px solid #adb5c7; padding: 6pt 8pt; text-align: left; vertical-align: top; }
      th { background: #f1f4fb; }
      .mermaid-frame {
        margin: 8pt 0 10pt;
        break-inside: avoid;
        page-break-inside: avoid;
      }
      img.mermaid {
        max-width: 100%;
        max-height: 260pt;
        width: auto;
        height: auto;
        display: block;
        margin: 0 auto;
        object-fit: contain;
        break-inside: avoid;
        page-break-inside: avoid;
      }
      .mermaid-fallback {
        padding: 10pt 12pt;
        border: 1px dashed #aab4cc;
        background: #fafbfe;
        margin: 0 0 10pt;
        break-inside: avoid;
        page-break-inside: avoid;
      }
      .mermaid-fallback .label { font-weight: 700; margin-bottom: 6px; }
      .muted { color: var(--muted); }
      hr { border: none; margin: 16pt 0; }
      hr.hr-hyphen { border-top: 1.5px solid #b6c0cf; }
      hr.hr-asterisk { border-top: 1.5px dashed #9daac0; }
      hr.hr-underscore { border-top: 3px double #c0c9d8; }
      a { color: #2156d1; }
      @media print {
        html, body { background: #fff; }
        body { padding: 0; }
        .page {
          margin: 0;
          box-shadow: none;
          overflow: visible;
          border: 0;
        }
        h1 { font-size: 24pt; }
        h2 { font-size: 20pt; }
        h3 { font-size: 18pt; }
        p, ul, ol, table { font-size: 11.5pt; }
        * {
          -webkit-print-color-adjust: exact;
          print-color-adjust: exact;
        }
      }
    "#);
    out.push_str("</style>");
    for page in pages {
        out.push_str("<section class=\"page\">");
        for block in page {
            out.push_str(&block);
        }
        out.push_str("</section>");
    }
    out
}

fn render_block_to_html(block: &MarkdownBlock) -> (String, f64) {
    match block {
        MarkdownBlock::Heading { level, spans, .. } => {
            let tag = match level {
                1 => "h1",
                2 => "h2",
                3 => "h3",
                _ => "h4",
            };
            let est = if *level == 1 { 56.0 } else { 40.0 };
            (format!("<{tag}>{}</{tag}>", render_spans_html(spans)), est)
        }
        MarkdownBlock::Paragraph { spans, .. } => {
            let text = render_spans_html(spans);
            (format!("<p>{text}</p>"), 40.0 + (spans.len() as f64 * 8.0))
        }
        MarkdownBlock::Spacer { lines, .. } => {
            let height = (*lines as f64).max(1.0) * 14.0;
            (
                format!("<div class=\"blank-line-spacer\" style=\"height:{height}pt\"></div>"),
                height,
            )
        }
        MarkdownBlock::ListItem { indent, spans, .. } => {
            let pad = 1 + *indent as usize;
            let text = render_spans_html(spans);
            (
                format!("<ul style=\"margin-left:{pad}em\"><li>{text}</li></ul>"),
                28.0,
            )
        }
        MarkdownBlock::OrderedListItem { n, indent, spans, .. } => {
            let pad = 1 + *indent as usize;
            let text = render_spans_html(spans);
            (
                format!("<ol start=\"{n}\" style=\"margin-left:{pad}em\"><li>{text}</li></ol>"),
                28.0,
            )
        }
        MarkdownBlock::ThematicBreak { style, .. } => {
            let class = match style {
                ThematicBreakStyle::Hyphen => "hr-hyphen",
                ThematicBreakStyle::Asterisk => "hr-asterisk",
                ThematicBreakStyle::Underscore => "hr-underscore",
            };
            (format!("<hr class=\"{class}\" />"), 18.0)
        }
        MarkdownBlock::Table { rows, .. } => (render_table_html(rows), 48.0 + rows.len() as f64 * 28.0),
        MarkdownBlock::CodeBlock { lang, code, .. } => {
            if lang == "mermaid" {
                if let Ok(Some((png_data, _width, height))) = render_mermaid_png(code) {
                    let encoded = general_purpose::STANDARD.encode(png_data);
                    return (
                        mermaid_preview_image_html(&encoded),
                        mermaid_preview_height(height),
                    );
                }
                return (mermaid_preview_fallback_html(), 96.0);
            }
            let escaped = escape_html(code);
            (
                format!("<pre><code>{escaped}</code></pre>"),
                code.lines().count().max(1) as f64 * 20.0 + 28.0,
            )
        }
        MarkdownBlock::DisplayMath { expr, .. } => {
            let escaped = escape_html(expr);
            (
                format!("<div class=\"math-block\">{escaped}</div>"),
                46.0 + expr.lines().count().max(1) as f64 * 12.0,
            )
        }
        MarkdownBlock::Blockquote { lines, .. } => {
            let html = render_blockquote_html_chunks(lines, f64::INFINITY)
                .into_iter()
                .next()
                .map(|(html, _)| html)
                .unwrap_or_else(|| "<blockquote></blockquote>".to_string());
            (html, 48.0)
        }
    }
}

fn render_blockquote_html_chunks(lines: &[BlockquoteLine], page_limit: f64) -> Vec<(String, f64)> {
    let mut chunks = Vec::new();
    let mut current_lines: Vec<String> = Vec::new();
    let mut current_height = 32.0_f64;

    for line in lines {
        let (rendered, line_height) = render_blockquote_html_line(line);
        if !current_lines.is_empty() && current_height + line_height > page_limit {
            chunks.push((
                format!("<blockquote>{}</blockquote>", current_lines.join("")),
                current_height,
            ));
            current_lines.clear();
            current_height = 32.0;
        }

        current_lines.push(rendered);
        current_height += line_height;
    }

    if !current_lines.is_empty() {
        chunks.push((
            format!("<blockquote>{}</blockquote>", current_lines.join("")),
            current_height,
        ));
    }

    if chunks.is_empty() {
        chunks.push(("<blockquote></blockquote>".to_string(), 32.0));
    }

    chunks
}

fn render_blockquote_html_line(line: &BlockquoteLine) -> (String, f64) {
    match line {
        BlockquoteLine::Text(spans) => {
            let rendered = render_spans_html(spans);
            (
                format!("<div class=\"blockquote-line\">{rendered}</div>"),
                22.0,
            )
        }
        BlockquoteLine::DisplayMath(expr) => (
            format!("<div class=\"blockquote-math\">{}</div>", escape_html(expr)),
            34.0,
        ),
        BlockquoteLine::CodeBlock { code, .. } => {
            let escaped = escape_html(code);
            let line_count = code.lines().count().max(1) as f64;
            (
                format!("<pre class=\"blockquote-code\"><code>{escaped}</code></pre>"),
                22.0 + line_count * 18.0,
            )
        }
    }
}

fn mermaid_preview_height(image_height: f64) -> f64 {
    (image_height + 20.0)
        .min(MERMAID_MAX_DISPLAY_HEIGHT)
        .max(120.0)
}

fn fit_mermaid_image_to_page(image_width: f64, image_height: f64) -> (f64, f64, f64) {
    let available_width = PAGE_WIDTH - 2.0 * MARGIN;
    let natural_width = image_width.max(1.0) * MERMAID_PT_PER_PX;
    let natural_height = image_height.max(1.0) * MERMAID_PT_PER_PX;
    let scale = (available_width / natural_width)
        .min(MERMAID_MAX_DISPLAY_HEIGHT / natural_height)
        .min(1.0);
    let draw_width = natural_width * scale;
    let draw_height = natural_height * scale;
    let draw_x = MARGIN + (available_width - draw_width) / 2.0;
    (draw_x, draw_width, draw_height)
}

fn mermaid_preview_image_html(encoded_png: &str) -> String {
    format!(
        "<div class=\"mermaid-frame\"><img class=\"mermaid\" alt=\"Mermaid diagram\" src=\"data:image/png;base64,{encoded_png}\" /></div>"
    )
}

fn mermaid_preview_fallback_html() -> String {
    String::from(
        "<div class=\"mermaid-fallback\"><div class=\"label\">Mermaid diagram preview unavailable</div><div class=\"muted\">Mermaid CLI を確認してください。</div></div>"
    )
}

fn normalize_mermaid_jpeg_bytes(png_data: &[u8]) -> Result<Vec<u8>, String> {
    let (jpeg, _, _) = normalize_png_to_jpeg_bytes(png_data)?;
    Ok(jpeg)
}

fn normalize_png_to_jpeg_bytes(png_data: &[u8]) -> Result<(Vec<u8>, u32, u32), String> {
    let decoded = image::load_from_memory(png_data)
        .map_err(|e| format!("PNG画像の正規化に失敗しました: {e}"))?;
    let rgba = decoded.to_rgba8();
    let (trimmed, width, height) = trim_transparent_border(&rgba);
    let mut flattened = ImageBuffer::from_pixel(width, height, Rgb([255, 255, 255]));

    for (x, y, pixel) in trimmed.enumerate_pixels() {
        let src = pixel.0;
        let alpha = src[3] as u16;
        let inv_alpha = 255u16.saturating_sub(alpha);
        let r = ((src[0] as u16 * alpha) + (255u16 * inv_alpha)) / 255;
        let g = ((src[1] as u16 * alpha) + (255u16 * inv_alpha)) / 255;
        let b = ((src[2] as u16 * alpha) + (255u16 * inv_alpha)) / 255;
        flattened.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
    }

    let mut normalized = Vec::new();
    DynamicImage::ImageRgb8(flattened)
        .write_to(
            &mut std::io::Cursor::new(&mut normalized),
            ImageOutputFormat::Jpeg(90),
        )
        .map_err(|e| format!("PNG画像の再エンコードに失敗しました: {e}"))?;
    Ok((normalized, width, height))
}

fn trim_transparent_border(image: &image::RgbaImage) -> (image::RgbaImage, u32, u32) {
    let mut min_x = image.width();
    let mut min_y = image.height();
    let mut max_x = 0u32;
    let mut max_y = 0u32;
    let mut found = false;

    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel.0[3] == 0 {
            continue;
        }
        found = true;
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }

    if !found {
        return (image.clone(), image.width(), image.height());
    }

    let width = (max_x - min_x + 1).max(1);
    let height = (max_y - min_y + 1).max(1);
    let cropped = image.view(min_x, min_y, width, height).to_image();
    (cropped, width, height)
}

fn render_table_html(rows: &[Vec<Vec<InlineSpan>>]) -> String {
    let mut out = String::from("<table><tbody>");
    for row in rows {
        out.push_str("<tr>");
        for cell in row {
            let cell_html = render_spans_html(cell);
            let _ = write!(out, "<td>{cell_html}</td>");
        }
        out.push_str("</tr>");
    }
    out.push_str("</tbody></table>");
    out
}

fn render_spans_html(spans: &[InlineSpan]) -> String {
    let mut out = String::new();
    for span in spans {
        let mut text = escape_html(&span.text).replace('\n', "<br />");
        if span.style.link {
            text = format!("<span class=\"link\">{text}</span>");
        }
        if span.style.math {
            text = format!("<span class=\"math\">{text}</span>");
        }
        if span.style.code {
            text = format!("<code>{text}</code>");
        }
        if span.style.bold {
            text = format!("<strong>{text}</strong>");
        }
        if span.style.italic {
            text = format!("<em>{text}</em>");
        }
        out.push_str(&text);
    }
    out
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn comrak_options() -> Options<'static> {
    let mut opts = Options::default();
    opts.render.sourcepos = true;
    opts.extension.strikethrough = true;
    opts.extension.table = true;
    opts.extension.tagfilter = true;
    opts.extension.autolink = true;
    opts.extension.tasklist = true;
    opts.extension.math_dollars = true;
    opts.extension.math_code = true;
    opts
}

fn parse_markdown_blocks(markdown: &str) -> Vec<MarkdownBlock> {
    let markdown = normalize_markdown_newlines(markdown);
    let markdown = normalize_markdown_math_delimiters(&markdown);
    let markdown = normalize_unclosed_mermaid_fences(&markdown);
    let markdown_lines: Vec<&str> = markdown.lines().collect();
    let arena = Arena::new();
    let root = parse_document(&arena, &markdown, &comrak_options());
    let mut blocks = Vec::new();
    push_blocks_from_children(root, &mut blocks, &markdown_lines);
    finalize_block_spacers(blocks, &markdown_lines)
}

fn block_source(block: &MarkdownBlock) -> BlockSource {
    match block {
        MarkdownBlock::Heading { source, .. }
        | MarkdownBlock::Paragraph { source, .. }
        | MarkdownBlock::Spacer { source, .. }
        | MarkdownBlock::ListItem { source, .. }
        | MarkdownBlock::OrderedListItem { source, .. }
        | MarkdownBlock::ThematicBreak { source, .. }
        | MarkdownBlock::Table { source, .. }
        | MarkdownBlock::CodeBlock { source, .. }
        | MarkdownBlock::DisplayMath { source, .. }
        | MarkdownBlock::Blockquote { source, .. } => *source,
    }
}

fn count_blank_source_lines(markdown_lines: &[&str], start_line: u32, end_line: u32) -> usize {
    if end_line < start_line {
        return 0;
    }
    (start_line..=end_line)
        .filter_map(|line| markdown_lines.get(line.saturating_sub(1) as usize))
        .filter(|line| line.trim().is_empty())
        .count()
}

fn trim_trailing_blank_source_lines(
    markdown_lines: &[&str],
    start_line: u32,
    end_line: u32,
) -> u32 {
    let mut effective_end = end_line;
    while effective_end > start_line {
        let Some(line) = markdown_lines.get(effective_end as usize - 1) else {
            break;
        };
        if line.trim().is_empty() {
            effective_end -= 1;
        } else {
            break;
        }
    }
    effective_end
}

fn trim_leading_blank_source_lines(
    markdown_lines: &[&str],
    start_line: u32,
    end_line: u32,
) -> u32 {
    let mut effective_start = start_line;
    while effective_start < end_line {
        let Some(line) = markdown_lines.get(effective_start as usize - 1) else {
            break;
        };
        if line.trim().is_empty() {
            effective_start += 1;
        } else {
            break;
        }
    }
    effective_start
}

fn effective_source_bounds(
    block: &MarkdownBlock,
    markdown_lines: &[&str],
) -> BlockSource {
    let source = block_source(block);
    if matches!(
        block,
        MarkdownBlock::CodeBlock { .. } | MarkdownBlock::DisplayMath { .. }
    ) {
        return source;
    }
    let start = trim_leading_blank_source_lines(
        markdown_lines,
        source.start_line,
        source.end_line,
    );
    let end = trim_trailing_blank_source_lines(markdown_lines, start, source.end_line);
    BlockSource {
        start_line: start,
        end_line: end,
    }
}

/// Inserts `Spacer` blocks for blank source lines between rendered blocks (lists, HR, headings, etc.).
fn finalize_block_spacers(
    blocks: Vec<MarkdownBlock>,
    markdown_lines: &[&str],
) -> Vec<MarkdownBlock> {
    let content_blocks: Vec<MarkdownBlock> = blocks
        .into_iter()
        .filter(|block| !matches!(block, MarkdownBlock::Spacer { .. }))
        .collect();

    let mut out = Vec::new();
    let mut prev_end: Option<u32> = None;

    for block in content_blocks {
        let effective = effective_source_bounds(&block, markdown_lines);
        if let Some(prev_end_line) = prev_end {
            let gap_start = prev_end_line.saturating_add(1);
            let gap_end = effective.start_line.saturating_sub(1);
            if gap_start <= gap_end {
                let blank_lines = count_blank_source_lines(markdown_lines, gap_start, gap_end);
                if blank_lines > 0 {
                    out.push(MarkdownBlock::Spacer {
                        lines: blank_lines.min(MAX_SPACER_LINES),
                        source: BlockSource {
                            start_line: gap_start,
                            end_line: gap_end,
                        },
                    });
                }
            }
        }
        out.push(block);
        prev_end = Some(effective.end_line);
    }

    out
}

fn normalize_markdown_newlines(markdown: &str) -> String {
    markdown.replace("\r\n", "\n").replace('\r', "\n")
}

fn normalize_markdown_math_delimiters(markdown: &str) -> String {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut normalized = Vec::with_capacity(lines.len());
    let mut i = 0usize;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if let Some(expr) = trimmed
            .strip_prefix("\\(")
            .and_then(|value| value.strip_suffix("\\)"))
        {
            let expr = expr.trim();
            if !expr.is_empty() {
                normalized.push("$$".to_string());
                normalized.push(expr.to_string());
                normalized.push("$$".to_string());
                i += 1;
                continue;
            }
        }

        if trimmed == "\\[" {
            let mut expr_lines = Vec::new();
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim() != "\\]" {
                expr_lines.push(lines[j]);
                j += 1;
            }
            if j < lines.len() {
                let expr = expr_lines.join("\n").trim().to_string();
                if !expr.is_empty() {
                    normalized.push("$$".to_string());
                    normalized.push(expr);
                    normalized.push("$$".to_string());
                    i = j + 1;
                    continue;
                }
            }
        }

        normalized.push(line.to_string());
        i += 1;
    }

    normalized.join("\n")
}

fn normalize_unclosed_mermaid_fences(markdown: &str) -> String {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut normalized = Vec::with_capacity(lines.len());
    let mut i = 0usize;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();
        if trimmed.starts_with("```mermaid") {
            normalized.push(line.to_string());
            i += 1;
            let mut closed = false;

            while i < lines.len() {
                let next_line = lines[i];
                let next_trimmed = next_line.trim_start();
                if next_trimmed == "```" {
                    normalized.push(next_line.to_string());
                    i += 1;
                    closed = true;
                    break;
                }
                if is_markdown_heading_line(next_line) {
                    normalized.push("```".to_string());
                    closed = true;
                    break;
                }
                normalized.push(next_line.to_string());
                i += 1;
            }

            if !closed {
                normalized.push("```".to_string());
            }
            continue;
        }

        normalized.push(line.to_string());
        i += 1;
    }

    normalized.join("\n")
}

fn is_markdown_heading_line(line: &str) -> bool {
    matches!(
        line.trim_start(),
        s if s.starts_with("# ") || s.starts_with("## ") || s.starts_with("### ") || s.starts_with("#### ") || s.starts_with("##### ") || s.starts_with("###### ")
    )
}

fn push_blocks_from_children<'a>(
    node: &'a AstNode<'a>,
    blocks: &mut Vec<MarkdownBlock>,
    markdown_lines: &[&str],
) {
    for child in node.children() {
        push_block(child, blocks, markdown_lines);
    }
}

/// `List` ノードをフラットな `ListItem` / `OrderedListItem` に展開する（子 `List` は `indent+1`）。
fn flatten_list_blocks<'a>(
    list_node: &'a AstNode<'a>,
    indent: u8,
    blocks: &mut Vec<MarkdownBlock>,
    markdown_lines: &[&str],
) {
    let NodeValue::List(list) = &list_node.data.borrow().value else {
        return;
    };
    match list.list_type {
        ListType::Bullet => {
            for item in list_node.children() {
                if !matches!(&item.data.borrow().value, NodeValue::Item(_)) {
                    continue;
                }
                for child in item.children() {
                    match &child.data.borrow().value {
                        NodeValue::Paragraph => {
                            let spans = paragraph_spans(child);
                            if !spans.is_empty() {
                                blocks.push(MarkdownBlock::ListItem {
                                    indent,
                                    spans,
                                    source: BlockSource::from_node(child),
                                });
                            }
                        }
                        NodeValue::List(_) => {
                            flatten_list_blocks(
                                child,
                                indent.saturating_add(1),
                                blocks,
                                markdown_lines,
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
        ListType::Ordered => {
            let mut n = list.start.max(1) as u32;
            for item in list_node.children() {
                if !matches!(&item.data.borrow().value, NodeValue::Item(_)) {
                    continue;
                }
                for child in item.children() {
                    match &child.data.borrow().value {
                        NodeValue::Paragraph => {
                            let spans = paragraph_spans(child);
                            blocks.push(MarkdownBlock::OrderedListItem {
                                n,
                                indent,
                                spans,
                                source: BlockSource::from_node(child),
                            });
                        }
                        NodeValue::List(_) => {
                            flatten_list_blocks(
                                child,
                                indent.saturating_add(1),
                                blocks,
                                markdown_lines,
                            );
                        }
                        _ => {}
                    }
                }
                n = n.saturating_add(1);
            }
        }
    }
}

fn push_block<'a>(node: &'a AstNode<'a>, blocks: &mut Vec<MarkdownBlock>, markdown_lines: &[&str]) {
    match &node.data.borrow().value {
        NodeValue::Document => push_blocks_from_children(node, blocks, markdown_lines),
        NodeValue::FrontMatter(_) => {}
        NodeValue::BlockQuote => {
            let mut lines = Vec::new();
            collect_blockquote_content(node, 0, &mut lines, markdown_lines);
            if !lines.is_empty() {
                blocks.push(MarkdownBlock::Blockquote {
                    lines,
                    source: BlockSource::from_node(node),
                });
            }
        }
        NodeValue::MultilineBlockQuote(_) => {
            let mut lines = Vec::new();
            collect_blockquote_content(node, 0, &mut lines, markdown_lines);
            if !lines.is_empty() {
                blocks.push(MarkdownBlock::Blockquote {
                    lines,
                    source: BlockSource::from_node(node),
                });
            }
        }
        NodeValue::List(_) => {
            flatten_list_blocks(node, 0, blocks, markdown_lines);
        }
        NodeValue::Paragraph => {
            if let Some(expr) = paragraph_display_math_expr(node) {
                blocks.push(MarkdownBlock::DisplayMath {
                    expr,
                    source: BlockSource::from_node(node),
                });
            } else {
                let spans = paragraph_spans(node);
                if !spans.is_empty() {
                    blocks.push(MarkdownBlock::Paragraph {
                        spans,
                        source: BlockSource::from_node(node),
                    });
                }
            }
        }
        NodeValue::Math(math) => push_math_block(math, node, blocks),
        NodeValue::Heading(h) => {
            let spans = paragraph_spans(node);
            if !spans.is_empty() {
                blocks.push(MarkdownBlock::Heading {
                    level: h.level as usize,
                    spans,
                    source: BlockSource::from_node(node),
                });
            }
        }
        NodeValue::CodeBlock(nb) => {
            blocks.push(code_block_from_node(nb, node));
        }
        NodeValue::Table(_) => {
            if let Some(t) = table_from_node(node) {
                blocks.push(t);
            }
        }
        NodeValue::ThematicBreak => {
            let source_line = node.data.borrow().sourcepos.start.line.saturating_sub(1);
            let style = markdown_lines
                .get(source_line)
                .map(|line| classify_thematic_break_style(line))
                .unwrap_or(ThematicBreakStyle::Hyphen);
            blocks.push(MarkdownBlock::ThematicBreak {
                style,
                source: BlockSource::from_node(node),
            });
        }
        NodeValue::HtmlBlock(nb) => {
            let t = nb.literal.trim();
            if !t.is_empty() {
                let mut v = Vec::new();
                InlineSpan::push(&mut v, t, TextStyle::default());
                blocks.push(MarkdownBlock::Paragraph {
                    spans: v,
                    source: BlockSource::from_node(node),
                });
            }
        }
        NodeValue::Item(_) => {
            let spans = item_first_paragraph_spans(node);
            if !spans.is_empty() {
                blocks.push(MarkdownBlock::ListItem {
                    indent: 0,
                    spans,
                    source: BlockSource::from_node(node),
                });
            }
        }
        _ => {
            for c in node.children() {
                push_block(c, blocks, markdown_lines);
            }
        }
    }
}

fn code_block_from_node<'a>(nb: &NodeCodeBlock, node: &'a AstNode<'a>) -> MarkdownBlock {
    MarkdownBlock::CodeBlock {
        lang: nb.info.trim().to_ascii_lowercase(),
        code: nb.literal.clone(),
        source: BlockSource::from_node(node),
    }
}

fn table_from_node<'a>(table: &'a AstNode<'a>) -> Option<MarkdownBlock> {
    let mut rows: Vec<Vec<Vec<InlineSpan>>> = Vec::new();
    for row in table.children() {
        if !matches!(&row.data.borrow().value, NodeValue::TableRow(_)) {
            continue;
        }
        let mut cells: Vec<Vec<InlineSpan>> = Vec::new();
        for cell in row.children() {
            if !matches!(&cell.data.borrow().value, NodeValue::TableCell) {
                continue;
            }
            let mut spans = Vec::new();
            for c in cell.children() {
                match &c.data.borrow().value {
                    NodeValue::Paragraph => {
                        for cc in c.children() {
                            collect_inline_spans(cc, TextStyle::default(), &mut spans);
                        }
                    }
                    _ => collect_inline_spans(c, TextStyle::default(), &mut spans),
                }
            }
            merge_adjacent_spans(&mut spans);
            cells.push(spans);
        }
        if !cells.is_empty() {
            rows.push(cells);
        }
    }
    if rows.is_empty() {
        None
    } else {
        Some(MarkdownBlock::Table {
            rows,
            source: BlockSource::from_node(table),
        })
    }
}

fn collect_blockquote_content<'a>(
    node: &'a AstNode<'a>,
    level: u8,
    out: &mut Vec<BlockquoteLine>,
    markdown_lines: &[&str],
) {
    for c in node.children() {
        match &c.data.borrow().value {
            NodeValue::Paragraph => {
                if let Some(expr) = paragraph_display_math_expr(c) {
                    append_blockquote_math(out, expr);
                } else {
                    append_blockquote_text(out, level, paragraph_spans(c));
                }
            }
            NodeValue::BlockQuote | NodeValue::MultilineBlockQuote(_) => {
                collect_blockquote_content(c, level.saturating_add(1), out, markdown_lines);
            }
            NodeValue::List(_) => {
                let mut flat = Vec::new();
                flatten_list_blocks(c, 0, &mut flat, markdown_lines);
                for b in flat {
                    match b {
                        MarkdownBlock::ListItem {
                            indent,
                            mut spans,
                            ..
                        } => {
                            if spans.is_empty() {
                                continue;
                            }
                            let mut line = Vec::new();
                            if level > 0 {
                                let prefix = format!("{} ", "> ".repeat(level as usize));
                                InlineSpan::push(&mut line, &prefix, TextStyle::default());
                            }
                            let pad = "  ".repeat(indent as usize);
                            InlineSpan::push(&mut line, &format!("{pad}• "), TextStyle::default());
                            line.append(&mut spans);
                            append_blockquote_text(out, level, line);
                        }
                        MarkdownBlock::OrderedListItem {
                            n,
                            indent,
                            mut spans,
                            ..
                        } => {
                            let mut line = Vec::new();
                            if level > 0 {
                                let prefix = format!("{} ", "> ".repeat(level as usize));
                                InlineSpan::push(&mut line, &prefix, TextStyle::default());
                            }
                            let pad = "  ".repeat(indent as usize);
                            InlineSpan::push(
                                &mut line,
                                &format!("{pad}{n}. "),
                                TextStyle::default(),
                            );
                            line.append(&mut spans);
                            append_blockquote_text(out, level, line);
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                let mut tmp = Vec::new();
                push_block(c, &mut tmp, markdown_lines);
                for b in tmp {
                    match b {
                        MarkdownBlock::Paragraph { spans: s, .. } => {
                            append_blockquote_text(out, level, s);
                        }
                        MarkdownBlock::Blockquote { lines: s, .. } => {
                            out.extend(s);
                        }
                        MarkdownBlock::CodeBlock { lang, code, .. } => {
                            append_blockquote_code_block(out, lang, code);
                        }
                        MarkdownBlock::Heading { spans, .. } if !spans.is_empty() => {
                            append_blockquote_text(out, level, spans);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn append_blockquote_text(out: &mut Vec<BlockquoteLine>, level: u8, mut spans: Vec<InlineSpan>) {
    if spans.is_empty() {
        return;
    }
    if level > 0 {
        let prefix = format!("{} ", "> ".repeat(level as usize));
        let mut prefixed = Vec::new();
        InlineSpan::push(&mut prefixed, &prefix, TextStyle::default());
        prefixed.append(&mut spans);
        spans = prefixed;
    }
    for line in layout_spans_lines(&spans, blockquote_wrap_cols()) {
        out.push(BlockquoteLine::Text(line));
    }
}

fn append_blockquote_math(out: &mut Vec<BlockquoteLine>, expr: String) {
    let expr = expr.trim().to_string();
    if expr.is_empty() {
        return;
    }
    out.push(BlockquoteLine::DisplayMath(expr));
}

fn append_blockquote_code_block(out: &mut Vec<BlockquoteLine>, _lang: String, code: String) {
    let code = code.trim_end_matches('\n').to_string();
    if code.is_empty() {
        return;
    }
    out.push(BlockquoteLine::CodeBlock { code });
}

fn paragraph_spans<'a>(p: &'a AstNode<'a>) -> Vec<InlineSpan> {
    let mut out = Vec::new();
    for c in p.children() {
        collect_inline_spans(c, TextStyle::default(), &mut out);
    }
    merge_adjacent_spans(&mut out);
    out
}

fn item_first_paragraph_spans<'a>(item: &'a AstNode<'a>) -> Vec<InlineSpan> {
    for c in item.children() {
        if matches!(&c.data.borrow().value, NodeValue::Paragraph) {
            return paragraph_spans(c);
        }
    }
    let mut out = Vec::new();
    for c in item.children() {
        collect_inline_spans(c, TextStyle::default(), &mut out);
    }
    merge_adjacent_spans(&mut out);
    out
}

fn collect_inline_spans<'a>(
    node: &'a AstNode<'a>,
    inherited: TextStyle,
    out: &mut Vec<InlineSpan>,
) {
    match &node.data.borrow().value {
        NodeValue::Text(t) => push_text_with_math(out, t, inherited),
        NodeValue::SoftBreak => InlineSpan::push(out, "\n", inherited),
        NodeValue::LineBreak => InlineSpan::push(out, "\n", inherited),
        NodeValue::Math(math) => {
            InlineSpan::push(
                out,
                &math.literal,
                TextStyle {
                    math: true,
                    ..inherited
                },
            );
        }
        NodeValue::Code(NodeCode { literal, .. }) => {
            InlineSpan::push(
                out,
                literal,
                TextStyle {
                    bold: false,
                    italic: false,
                    code: true,
                    link: inherited.link,
                    math: inherited.math,
                },
            );
        }
        NodeValue::Strong => {
            let st = TextStyle {
                bold: true,
                italic: inherited.italic,
                code: false,
                link: inherited.link,
                math: inherited.math,
            };
            for c in node.children() {
                collect_inline_spans(c, st, out);
            }
        }
        NodeValue::Emph => {
            let st = TextStyle {
                bold: inherited.bold,
                italic: true,
                code: false,
                link: inherited.link,
                math: inherited.math,
            };
            for c in node.children() {
                collect_inline_spans(c, st, out);
            }
        }
        NodeValue::Strikethrough => {
            for c in node.children() {
                collect_inline_spans(c, inherited, out);
            }
        }
        NodeValue::Link(_) | NodeValue::WikiLink(_) => {
            let st = TextStyle {
                link: true,
                math: inherited.math,
                ..inherited
            };
            for c in node.children() {
                collect_inline_spans(c, st, out);
            }
        }
        NodeValue::Image(link) => {
            let mut alt = String::new();
            for c in node.children() {
                if let NodeValue::Text(t) = &c.data.borrow().value {
                    alt.push_str(t);
                }
            }
            let label = if alt.is_empty() {
                link.url.clone()
            } else {
                alt
            };
            InlineSpan::push(out, &format!("[画像: {label}]"), inherited);
        }
        NodeValue::HtmlInline(raw) | NodeValue::Raw(raw) => InlineSpan::push(out, raw, inherited),
        NodeValue::TaskItem(checked) => {
            let mark = if checked.is_some() { "☑ " } else { "☐ " };
            InlineSpan::push(out, mark, inherited);
        }
        NodeValue::FootnoteReference(_) | NodeValue::Superscript | NodeValue::Subscript => {}
        NodeValue::Escaped => {
            for c in node.children() {
                collect_inline_spans(c, inherited, out);
            }
        }
        NodeValue::Underline => {
            for c in node.children() {
                collect_inline_spans(c, inherited, out);
            }
        }
        _ => {
            for c in node.children() {
                collect_inline_spans(c, inherited, out);
            }
        }
    }
}

fn push_text_with_math(out: &mut Vec<InlineSpan>, text: &str, inherited: TextStyle) {
    let mut rest = text;
    while !rest.is_empty() {
        let Some((pos, delimiter)) = next_math_delimiter(rest) else {
            InlineSpan::push(out, rest, inherited);
            return;
        };

        if pos > 0 {
            InlineSpan::push(out, &rest[..pos], inherited);
        }

        let after = &rest[pos..];
        let handled = match delimiter {
            MathDelimiter::DisplayDollar => after[2..].find("$$").map(|end| {
                let expr = after[2..2 + end].trim();
                if !expr.is_empty() {
                    InlineSpan::push(
                        out,
                        expr,
                        TextStyle {
                            math: true,
                            ..inherited
                        },
                    );
                }
                2 + end + 2
            }),
            MathDelimiter::InlineDollar => after[1..].find('$').map(|end| {
                let expr = after[1..1 + end].trim();
                if !expr.is_empty() {
                    InlineSpan::push(
                        out,
                        expr,
                        TextStyle {
                            math: true,
                            ..inherited
                        },
                    );
                }
                1 + end + 1
            }),
        };

        if let Some(advance) = handled {
            rest = &after[advance..];
            continue;
        }

        InlineSpan::push(out, &rest[pos..], inherited);
        return;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MathDelimiter {
    DisplayDollar,
    InlineDollar,
}

fn next_math_delimiter(rest: &str) -> Option<(usize, MathDelimiter)> {
    let mut best: Option<(usize, MathDelimiter)> = None;
    for (pattern, delimiter) in [
        ("$$", MathDelimiter::DisplayDollar),
        ("$", MathDelimiter::InlineDollar),
    ] {
        if let Some(pos) = rest.find(pattern) {
            let should_replace = match best {
                None => true,
                Some((best_pos, _)) => pos < best_pos,
            };
            if should_replace {
                best = Some((pos, delimiter));
            }
        }
    }
    best
}

fn paragraph_display_math_expr<'a>(node: &'a AstNode<'a>) -> Option<String> {
    let raw_text = paragraph_raw_text(node);
    let trimmed = raw_text.trim();
    if let Some(expr) = trimmed
        .strip_prefix("\\[")
        .and_then(|value| value.strip_suffix("\\]"))
    {
        let expr = expr.trim();
        if !expr.is_empty() {
            return Some(expr.to_string());
        }
    }

    let mut expr = String::new();
    let mut saw_math = false;
    for child in node.children() {
        match &child.data.borrow().value {
            NodeValue::Math(math) if math.display_math => {
                expr.push_str(math.literal.trim());
                saw_math = true;
            }
            NodeValue::SoftBreak | NodeValue::LineBreak => {
                if saw_math {
                    expr.push('\n');
                }
            }
            NodeValue::Text(t) if t.trim().is_empty() => {}
            _ => return None,
        }
    }
    if saw_math {
        let trimmed = expr.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    } else {
        None
    }
}

fn paragraph_raw_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut out = String::new();
    for child in node.children() {
        match &child.data.borrow().value {
            NodeValue::Text(t) => out.push_str(t),
            NodeValue::SoftBreak | NodeValue::LineBreak => out.push('\n'),
            _ => {}
        }
    }
    out
}

fn format_math_expression(expr: &str) -> Vec<String> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return vec![String::new()];
    }
    if let Some(lines) = format_math_environment(trimmed, "bmatrix", "[", "]") {
        return lines;
    }
    if let Some(lines) = format_math_environment(trimmed, "pmatrix", "(", ")") {
        return lines;
    }
    if let Some(lines) = format_cases_environment(trimmed) {
        return lines;
    }

    let lines: Vec<String> = trimmed
        .lines()
        .map(|line| simplify_math_line(line.trim()))
        .filter(|line| !line.is_empty())
        .collect();
    if lines.is_empty() {
        vec![simplify_math_line(trimmed)]
    } else {
        lines
    }
}

fn format_math_environment(
    expr: &str,
    env_name: &str,
    open: &str,
    close: &str,
) -> Option<Vec<String>> {
    let begin = format!("\\begin{{{env_name}}}");
    let end = format!("\\end{{{env_name}}}");
    let start = expr.find(&begin)?;
    let finish = expr.rfind(&end)?;
    if finish <= start {
        return None;
    }
    let prefix = expr[..start].trim();
    let body = expr[start + begin.len()..finish].trim();
    let rows = split_math_rows(body);
    if rows.is_empty() {
        return None;
    }

    let mut out = Vec::new();
    if !prefix.is_empty() {
        out.push(simplify_math_line(prefix));
    }
    for (idx, row) in rows.iter().enumerate() {
        let cells = row
            .split('&')
            .map(|cell| simplify_math_line(cell.trim()))
            .filter(|cell| !cell.is_empty())
            .collect::<Vec<_>>();
        let row_text = cells.join("  ");
        if idx == 0 {
            out.push(format!("{open} {row_text}"));
        } else if idx + 1 == rows.len() {
            out.push(format!("  {row_text} {close}"));
        } else {
            out.push(format!("  {row_text}"));
        }
    }
    Some(out)
}

fn format_cases_environment(expr: &str) -> Option<Vec<String>> {
    let begin = "\\begin{cases}";
    let end = "\\end{cases}";
    let start = expr.find(begin)?;
    let finish = expr.rfind(end)?;
    if finish <= start {
        return None;
    }
    let prefix = expr[..start].trim();
    let body = expr[start + begin.len()..finish].trim();
    let rows = split_math_rows(body);
    if rows.is_empty() {
        return None;
    }

    let mut out = Vec::new();
    if !prefix.is_empty() {
        out.push(simplify_math_line(prefix));
    }
    out.push("{".to_string());
    for row in rows {
        out.push(format!("  {}", simplify_math_line(&row.replace('&', " "))));
    }
    out.push("}".to_string());
    Some(out)
}

fn split_math_rows(body: &str) -> Vec<String> {
    body.split("\\\\")
        .map(|row| row.trim().to_string())
        .filter(|row| !row.is_empty())
        .collect()
}

fn simplify_math_line(line: &str) -> String {
    let mut s = line.to_string();
    s = s.replace("\\displaystyle", "");
    s = s.replace("\\left", "");
    s = s.replace("\\right", "");
    s = s.replace("\\lVert", "‖");
    s = s.replace("\\rVert", "‖");
    s = s.replace("\\lvert", "|");
    s = s.replace("\\rvert", "|");
    s = s.replace("\\|", "‖");
    s = s.replace("\\vert", "|");
    s = s.replace("\\cdot", "·");
    s = s.replace("\\times", "×");
    s = s.replace("\\sum", "∑");
    s = s.replace("\\int", "∫");
    s = s.replace("\\sqrt", "√");
    s = s.replace("\\infty", "∞");
    s = s.replace("\\nabla", "∇");
    s = s.replace("\\pi", "π");
    s = s.replace("\\alpha", "α");
    s = s.replace("\\beta", "β");
    s = s.replace("\\gamma", "γ");
    s = s.replace("\\rho", "ρ");
    s = s.replace("\\varepsilon", "ε");
    s = s.replace("\\le", "≤");
    s = s.replace("\\ge", "≥");
    s = s.replace("\\neq", "≠");
    s = s.replace("\\mid", "|");
    s = s.replace("\\to", "→");
    s = s.replace("\\mathrm", "");
    s = s.replace("\\mathbf", "");
    s = s.replace("\\text", "");
    s = s.replace("\\,", " ");
    s = s.replace("\\quad", "  ");
    s = replace_frac_notation(&s);
    s = simplify_sup_subscripts(&s);
    s = s.replace('{', "");
    s = s.replace('}', "");
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn replace_frac_notation(input: &str) -> String {
    let mut out = String::new();
    let mut rest = input;
    while let Some(pos) = rest.find("\\frac{") {
        out.push_str(&rest[..pos]);
        let after = &rest[pos + 5..];
        if let Some((num, after_num)) = take_braced(after) {
            if let Some((den, after_den)) = take_braced(after_num) {
                out.push('(');
                out.push_str(&simplify_math_line(num));
                out.push_str(")/(");
                out.push_str(&simplify_math_line(den));
                out.push(')');
                rest = after_den;
                continue;
            }
        }
        out.push_str("\\frac{");
        rest = after;
    }
    out.push_str(rest);
    out
}

fn take_braced(input: &str) -> Option<(&str, &str)> {
    let mut depth = 0usize;
    let mut start = None;
    for (idx, ch) in input.char_indices() {
        match ch {
            '{' => {
                depth += 1;
                if start.is_none() {
                    start = Some(idx + 1);
                }
            }
            '}' => {
                if depth == 0 {
                    return None;
                }
                depth -= 1;
                if depth == 0 {
                    let start = start?;
                    return Some((&input[start..idx], &input[idx + 1..]));
                }
            }
            _ => {}
        }
    }
    None
}

fn simplify_sup_subscripts(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::new();
    let mut i = 0usize;
    while i < chars.len() {
        match chars[i] {
            '^' | '_' => {
                let is_super = chars[i] == '^';
                i += 1;
                if i >= chars.len() {
                    break;
                }
                let token = if chars[i] == '{' {
                    let mut depth = 1usize;
                    let start = i + 1;
                    i += 1;
                    while i < chars.len() && depth > 0 {
                        match chars[i] {
                            '{' => depth += 1,
                            '}' => depth -= 1,
                            _ => {}
                        }
                        i += 1;
                    }
                    let end = i.saturating_sub(1);
                    chars[start..end].iter().collect::<String>()
                } else {
                    let token = chars[i].to_string();
                    i += 1;
                    token
                };
                let mapped = if is_super {
                    token.chars().map(super_script_char).collect::<String>()
                } else {
                    token.chars().map(sub_script_char).collect::<String>()
                };
                if mapped.is_empty() {
                    out.push_str(&token);
                } else {
                    out.push_str(&mapped);
                }
            }
            ch => {
                out.push(ch);
                i += 1;
            }
        }
    }
    out
}

fn super_script_char(ch: char) -> char {
    match ch {
        '0' => '⁰',
        '1' => '¹',
        '2' => '²',
        '3' => '³',
        '4' => '⁴',
        '5' => '⁵',
        '6' => '⁶',
        '7' => '⁷',
        '8' => '⁸',
        '9' => '⁹',
        '+' => '⁺',
        '-' => '⁻',
        '=' => '⁼',
        '(' => '⁽',
        ')' => '⁾',
        'n' => 'ⁿ',
        'i' => 'ⁱ',
        _ => ch,
    }
}

fn sub_script_char(ch: char) -> char {
    match ch {
        '0' => '₀',
        '1' => '₁',
        '2' => '₂',
        '3' => '₃',
        '4' => '₄',
        '5' => '₅',
        '6' => '₆',
        '7' => '₇',
        '8' => '₈',
        '9' => '₉',
        '+' => '₊',
        '-' => '₋',
        '=' => '₌',
        '(' => '₍',
        ')' => '₎',
        'a' => 'ₐ',
        'e' => 'ₑ',
        'h' => 'ₕ',
        'i' => 'ᵢ',
        'j' => 'ⱼ',
        'k' => 'ₖ',
        'l' => 'ₗ',
        'm' => 'ₘ',
        'n' => 'ₙ',
        'o' => 'ₒ',
        'p' => 'ₚ',
        'r' => 'ᵣ',
        's' => 'ₛ',
        't' => 'ₜ',
        'u' => 'ᵤ',
        'v' => 'ᵥ',
        'x' => 'ₓ',
        _ => ch,
    }
}

fn math_display_width(expr: &str) -> usize {
    let base = expr.chars().map(char_display_width).sum::<usize>();
    base.max(4)
}

fn render_math_with_katex(
    expr: &str,
    display_mode: bool,
) -> Result<Option<MathRenderImage>, String> {
    let script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("render_katex_math.cjs");
    if !script_path.exists() {
        return Ok(None);
    }

    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    let normalized_expr = normalize_math_for_katex(expr);
    let encoded = general_purpose::STANDARD.encode(normalized_expr.as_bytes());
    let output = Command::new("node")
        .current_dir(repo_root)
        .arg(&script_path)
        .arg(encoded)
        .arg(if display_mode { "1" } else { "0" })
        .output()
        .map_err(|e| format!("KaTeXレンダラの起動に失敗しました: {e}"))?;

    if !output.status.success() {
        log::warn!(
            "KaTeX renderer failed: status={} stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        return Ok(None);
    }

    let parsed: MathRenderResponse = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("KaTeXレンダラの出力解析に失敗しました: {e}"))?;
    let png_data = general_purpose::STANDARD
        .decode(parsed.png)
        .map_err(|e| format!("KaTeXレンダラのPNG復号に失敗しました: {e}"))?;
    let (jpeg_data, width_px, height_px) = normalize_png_to_jpeg_bytes(&png_data)?;
    let device_pixel_ratio = parsed.device_pixel_ratio.max(1.0);
    Ok(Some(MathRenderImage {
        jpeg_data,
        width_pt: (width_px as f64 / device_pixel_ratio) * MERMAID_PT_PER_PX,
        height_pt: (height_px as f64 / device_pixel_ratio) * MERMAID_PT_PER_PX,
        baseline_pt: (parsed.baseline_offset / device_pixel_ratio) * MERMAID_PT_PER_PX,
    }))
}

fn normalize_math_for_katex(expr: &str) -> String {
    let mut s = expr.to_string();
    s = s.replace("\\left\\lVert", "‖");
    s = s.replace("\\right\\rVert", "‖");
    s = s.replace("\\left\\|", "‖");
    s = s.replace("\\right\\|", "‖");
    s = s.replace("\\lVert", "‖");
    s = s.replace("\\rVert", "‖");
    s = s.replace("\\Vert", "‖");
    s = s.replace("\\lvert", "|");
    s = s.replace("\\rvert", "|");
    s = s.replace("\\vert", "|");
    s = s.replace("\\|", "‖");
    s
}

fn push_math_block<'a>(
    math: &NodeMath,
    node: &'a AstNode<'a>,
    blocks: &mut Vec<MarkdownBlock>,
) {
    let source = BlockSource::from_node(node);
    if math.display_math {
        blocks.push(MarkdownBlock::DisplayMath {
            expr: math.literal.clone(),
            source,
        });
    } else if !math.literal.trim().is_empty() {
        let mut spans = Vec::new();
        InlineSpan::push(
            &mut spans,
            &math.literal,
            TextStyle {
                math: true,
                ..TextStyle::default()
            },
        );
        blocks.push(MarkdownBlock::Paragraph { spans, source });
    }
}

fn merge_adjacent_spans(spans: &mut Vec<InlineSpan>) {
    let mut i = 1usize;
    while i < spans.len() {
        if spans[i].style == spans[i - 1].style {
            let t = spans[i].text.clone();
            spans[i - 1].text.push_str(&t);
            spans.remove(i);
        } else {
            i += 1;
        }
    }
}

struct PreviewFace {
    body: Font,
    body_bold: Font,
    body_italic: Font,
    body_bold_italic: Font,
    body_path: Option<PathBuf>,
    body_bold_path: Option<PathBuf>,
    emoji: Option<Font>,
    emoji_path: Option<PathBuf>,
    has_custom_bold_italic: bool,
    /// `NotoSansJP-Bold` が埋め込まれている（Helvetica 系で日本語太字にしない）
    has_custom_bold: bool,
}

impl PreviewFace {
    /// Courier / Helvetica Oblique は日本語に非対応。太字で Noto Bold が無いときは Helvetica を日本語に使わない。
    fn pick(&self, style: TextStyle, text: &str) -> Font {
        if style.code {
            if ascii_only(text) {
                return Font::Courier;
            }
            return self.body.clone();
        }
        if style.math {
            if ascii_only(text) {
                return Font::TimesItalic;
            }
            return self.body.clone();
        }
        let non_ascii = !ascii_only(text);
        match (style.bold, style.italic) {
            (true, true) if non_ascii => {
                if self.has_custom_bold {
                    self.body_bold.clone()
                } else {
                    self.body.clone()
                }
            }
            (true, false) if non_ascii => {
                if self.has_custom_bold {
                    self.body_bold.clone()
                } else {
                    self.body.clone()
                }
            }
            (true, false) => {
                if self.has_custom_bold {
                    self.body_bold.clone()
                } else {
                    Font::HelveticaBold
                }
            }
            (true, true) => {
                if self.has_custom_bold_italic {
                    self.body_bold_italic.clone()
                } else if self.has_custom_bold {
                    self.body_bold_italic.clone()
                } else {
                    Font::HelveticaBoldOblique
                }
            }
            (false, true) if non_ascii => self.body.clone(),
            (false, true) => self.body_italic.clone(),
            (false, false) => self.body.clone(),
        }
    }
}

fn is_potential_bold_face(face: &PreviewFace, style: TextStyle, text: &str) -> bool {
    if style.bold && style.italic && !ascii_only(text) {
        return !face.has_custom_bold_italic;
    }
    style.bold && !ascii_only(text) && !face.has_custom_bold
}

fn draw_fake_bold_text(
    page: &mut OxidizePage,
    text: &str,
    font: Font,
    size: f64,
    x: f64,
    y: f64,
    fill: Color,
) -> Result<(), String> {
    let offsets = [0.0_f64, 0.24, 0.48];
    for dx in offsets {
        page.text()
            .set_fill_color(fill)
            .set_font(font.clone(), size)
            .at(x + dx, y)
            .write(text)
            .map_err(|e| format!("テキスト描画に失敗しました: {e}"))?;
    }
    Ok(())
}

fn ascii_only(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii())
}

fn make_preview_face(doc: &Document, body: Font) -> PreviewFace {
    let has_bold = doc.has_custom_font(JP_SANS_BOLD);
    let body_path = font_manager::get_font_path(JP_SANS_FONT);
    let body_bold_path = font_manager::get_font_path(JP_SANS_BOLD).or_else(|| body_path.clone());
    PreviewFace {
        body: body.clone(),
        body_bold: if has_bold {
            Font::custom(JP_SANS_BOLD)
        } else {
            body.clone()
        },
        body_italic: if doc.has_custom_font("NotoSansJP-Italic") {
            Font::custom("NotoSansJP-Italic")
        } else {
            Font::HelveticaOblique
        },
        body_bold_italic: if doc.has_custom_font("NotoSansJP-BoldItalic") {
            Font::custom("NotoSansJP-BoldItalic")
        } else if has_bold {
            Font::custom(JP_SANS_BOLD)
        } else {
            body.clone()
        },
        body_path,
        body_bold_path,
        emoji: if doc.has_custom_font(EMOJI_FONT) {
            Some(Font::custom(EMOJI_FONT))
        } else {
            None
        },
        emoji_path: font_manager::get_font_path(EMOJI_FONT),
        has_custom_bold_italic: doc.has_custom_font("NotoSansJP-BoldItalic"),
        has_custom_bold: has_bold,
    }
}

fn draw_list_item_paginated(
    page: &mut OxidizePage,
    indent: u8,
    marker_text: &str,
    spans: &[InlineSpan],
    source: BlockSource,
    face: &PreviewFace,
    cursor_y: &mut f64,
    budget: usize,
    math_cache: &mut MathRenderCache,
    doc: &mut Document,
    layout: &mut PreviewLayoutState,
) -> Result<(), String> {
    let layout_lines = layout_spans_lines_with_source(spans, budget, source);
    let marker_x = list_marker_x(indent);
    let content_x = list_item_content_x(indent, marker_text, &face.body, 11.0);
    let marker = vec![InlineSpan {
        text: marker_text.to_string(),
        style: TextStyle::default(),
    }];

    for (index, line) in layout_lines.iter().enumerate() {
        if index == 0 {
            layout.record_line(source.start_line);
            *cursor_y = ensure_room(*cursor_y, BASE_LINE_HEIGHT, page, doc, layout);
            draw_rich_line_segments(
                page,
                &marker,
                face,
                11.0,
                marker_x,
                cursor_y,
                BASE_LINE_HEIGHT,
                math_cache,
            )?;
            *cursor_y += BASE_LINE_HEIGHT;
        }
        layout.record_line(line.source_line);
        *cursor_y = ensure_room(*cursor_y, BASE_LINE_HEIGHT, page, doc, layout);
        draw_rich_line_segments(
            page,
            &line.spans,
            face,
            11.0,
            content_x,
            cursor_y,
            BASE_LINE_HEIGHT,
            math_cache,
        )?;
    }
    *cursor_y -= 4.0;
    Ok(())
}

fn record_code_block_source_lines(
    layout: &mut PreviewLayoutState,
    source: BlockSource,
    rendered_line_count: usize,
) {
    if rendered_line_count == 0 {
        layout.record_source(source);
        return;
    }
    let span = source.end_line.saturating_sub(source.start_line);
    for index in 0..rendered_line_count {
        let line = source.start_line + (index as u32).min(span);
        layout.record_line(line);
    }
}

fn build_preview_pdf(
    blocks: &[MarkdownBlock],
    source_line_count: usize,
) -> Result<(Vec<u8>, Vec<u32>), String> {
    let mut doc = Document::new();
    font_manager::register_fonts_on_document(&mut doc)?;
    let mut math_cache = MathRenderCache::default();
    let mut layout = PreviewLayoutState::new(source_line_count);

    let body_font =
        if doc.has_custom_font(JP_SANS_FONT) {
            Font::custom(JP_SANS_FONT)
        } else {
            log::warn!(
            "プレビュー用に {} が埋め込めません。{} に NotoSansJP-Regular.ttf を置いてください。",
            JP_SANS_FONT,
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fonts").display()
        );
            Font::Helvetica
        };
    let face = make_preview_face(&doc, body_font);

    let mut page = OxidizePage::new(PAGE_WIDTH, PAGE_HEIGHT);
    let mut cursor_y = PAGE_HEIGHT - MARGIN;

    for block in blocks {
        match block {
            MarkdownBlock::Heading {
                level,
                spans,
                source,
            } => {
                let font_size = match level {
                    1 => 24.0,
                    2 => 20.0,
                    3 => 18.0,
                    _ => 16.0,
                };
                let spacing_before = if *level == 1 { 12.0 } else { 8.0 };
                let spacing_after = if *level == 1 { 10.0 } else { 8.0 };
                layout.record_line(source.start_line);
                cursor_y = ensure_room(cursor_y, spacing_before, &mut page, &mut doc, &mut layout);
                cursor_y -= spacing_before;
                draw_rich_block_paginated(
                    &mut page,
                    spans,
                    *source,
                    &face,
                    font_size,
                    MARGIN,
                    &mut cursor_y,
                    max_chars_for_font(font_size),
                    &mut math_cache,
                    &mut doc,
                    &mut layout,
                )?;
                cursor_y -= spacing_after;
            }
            MarkdownBlock::Paragraph { spans, source } => {
                draw_rich_block_paginated(
                    &mut page,
                    spans,
                    *source,
                    &face,
                    11.5,
                    MARGIN,
                    &mut cursor_y,
                    90,
                    &mut math_cache,
                    &mut doc,
                    &mut layout,
                )?;
                cursor_y -= 4.0;
            }
            MarkdownBlock::Spacer { lines, source } => {
                let visible_lines = (*lines).max(1);
                let span = source.end_line.saturating_sub(source.start_line) as usize;
                for index in 0..visible_lines {
                    let line = source.start_line + (index as u32).min(span as u32);
                    layout.record_line(line);
                    cursor_y =
                        ensure_room(cursor_y, BASE_LINE_HEIGHT, &mut page, &mut doc, &mut layout);
                    cursor_y -= BASE_LINE_HEIGHT;
                }
                let last_drawn = source
                    .start_line
                    .saturating_add((visible_lines as u32).saturating_sub(1))
                    .min(source.end_line);
                for line in last_drawn.saturating_add(1)..=source.end_line {
                    layout.record_line(line);
                }
            }
            MarkdownBlock::ListItem {
                indent,
                spans,
                source,
            } => {
                let ind = *indent;
                let bullet = "•";
                let prefix_cols = (ind as usize * 2) + unicode_display_width(bullet) + 2usize;
                let budget = 90usize.saturating_sub(prefix_cols + 4).max(20);
                draw_list_item_paginated(
                    &mut page,
                    ind,
                    bullet,
                    spans,
                    *source,
                    &face,
                    &mut cursor_y,
                    budget,
                    &mut math_cache,
                    &mut doc,
                    &mut layout,
                )?;
            }
            MarkdownBlock::OrderedListItem {
                n,
                indent,
                spans,
                source,
            } => {
                let ind = *indent;
                let prefix = format!("{n}.");
                let prefix_cols = (ind as usize * 2) + unicode_display_width(prefix.as_str()) + 2;
                let budget = 90usize.saturating_sub(prefix_cols + 4).max(20);
                draw_list_item_paginated(
                    &mut page,
                    ind,
                    &prefix,
                    spans,
                    *source,
                    &face,
                    &mut cursor_y,
                    budget,
                    &mut math_cache,
                    &mut doc,
                    &mut layout,
                )?;
            }
            MarkdownBlock::ThematicBreak { style, source } => {
                layout.record_line(source.start_line);
                cursor_y =
                    draw_thematic_break(&mut page, cursor_y, *style, &mut doc, &mut layout);
            }
            MarkdownBlock::Table { rows, source } => {
                layout.record_source(*source);
                let row_h = 16.0_f64;
                let n = rows.len().max(1);
                let est = row_h * n as f64 + 16.0;
                cursor_y = ensure_room(cursor_y, est, &mut page, &mut doc, &mut layout);
                draw_table(&mut page, rows, &face, &mut cursor_y, row_h)?;
                cursor_y -= 4.0;
            }
            MarkdownBlock::CodeBlock {
                lang,
                code,
                source,
            } => {
                if lang == "mermaid" {
                    if let Some((png_data, width, height)) = render_mermaid_png(code)? {
                        let jpeg_data = normalize_mermaid_jpeg_bytes(&png_data)?;
                        let (draw_x, draw_width, draw_height) =
                            fit_mermaid_image_to_page(width, height);
                        layout.record_source(*source);
                        cursor_y =
                            ensure_room(cursor_y, draw_height + 18.0, &mut page, &mut doc, &mut layout);
                        draw_image_on_page(
                            &mut page,
                            &jpeg_data,
                            "jpeg",
                            draw_x,
                            cursor_y - draw_height,
                            draw_width,
                            draw_height,
                        )?;
                        cursor_y -= draw_height + 8.0;
                        continue;
                    }
                }

                let lh = BASE_LINE_HEIGHT - 1.0;
                let pad_x = 10.0;
                let pad_y = 9.0;
                let fs = 9.5;
                if lang == "mermaid" {
                    let hint = "Mermaid diagram preview unavailable";
                    let detail = "MINIPDF_MERMAID_CLI / node_modules/.bin/mmdc / PATH を確認";
                    let mut hy = cursor_y - pad_y - fs * 0.85;
                    let label_font = face.body.clone();
                    page.text()
                        .set_fill_color(Color::rgb(0.05, 0.05, 0.07))
                        .set_font(label_font.clone(), 8.5)
                        .at(MARGIN + pad_x, hy)
                        .write(hint)
                        .map_err(|e| format!("テキスト描画に失敗しました: {e}"))?;
                    hy -= lh;
                    page.text()
                        .set_fill_color(Color::rgb(0.25, 0.28, 0.35))
                        .set_font(label_font, 8.0)
                        .at(MARGIN + pad_x, hy)
                        .write(detail)
                        .map_err(|e| format!("テキスト描画に失敗しました: {e}"))?;
                    cursor_y -= lh * 2.0 + 2.0;
                }
                let code_lines = wrap_code_lines(code, 88);
                record_code_block_source_lines(&mut layout, *source, code_lines.len());
                let whole_block_h = code_lines.len().max(1) as f64 * lh + pad_y * 2.0;
                if code_block_fits_on_single_page(whole_block_h) {
                    cursor_y =
                        ensure_room(cursor_y, whole_block_h + 8.0, &mut page, &mut doc, &mut layout);
                    let block_left = MARGIN + 4.0;
                    let block_width = PAGE_WIDTH - 2.0 * MARGIN - 8.0;
                    let block_bottom = cursor_y - whole_block_h;
                    page.graphics()
                        .set_fill_color(Color::rgb(0.965, 0.97, 0.985))
                        .rectangle(block_left, block_bottom, block_width, whole_block_h)
                        .fill();
                    page.graphics()
                        .set_stroke_color(Color::rgb(0.82, 0.86, 0.93))
                        .set_line_width(0.75)
                        .rectangle(block_left, block_bottom, block_width, whole_block_h)
                        .stroke();
                    let mut line_y = cursor_y - pad_y - fs * 0.85;
                    for line in &code_lines {
                        draw_code_block_line(
                            &mut page,
                            line,
                            &face,
                            fs,
                            block_left + pad_x,
                            line_y,
                        )?;
                        line_y -= lh;
                    }
                    cursor_y -= whole_block_h + 4.0;
                } else {
                    let mut idx = 0usize;
                    while idx < code_lines.len() {
                        let lines_remaining = code_lines.len() - idx;
                        let available_body_height = (cursor_y - MARGIN - 4.0).max(lh + pad_y * 2.0);
                        let lines_fit = ((available_body_height - pad_y * 2.0) / lh)
                            .floor()
                            .max(1.0) as usize;
                        let chunk_len = lines_remaining.min(lines_fit);
                        let chunk = &code_lines[idx..idx + chunk_len];
                        let block_h = chunk.len().max(1) as f64 * lh + pad_y * 2.0;
                        cursor_y =
                            ensure_room(cursor_y, block_h + 8.0, &mut page, &mut doc, &mut layout);
                        let block_left = MARGIN + 4.0;
                        let block_width = PAGE_WIDTH - 2.0 * MARGIN - 8.0;
                        let block_bottom = cursor_y - block_h;
                        page.graphics()
                            .set_fill_color(Color::rgb(0.965, 0.97, 0.985))
                            .rectangle(block_left, block_bottom, block_width, block_h)
                            .fill();
                        page.graphics()
                            .set_stroke_color(Color::rgb(0.82, 0.86, 0.93))
                            .set_line_width(0.75)
                            .rectangle(block_left, block_bottom, block_width, block_h)
                            .stroke();
                        let mut line_y = cursor_y - pad_y - fs * 0.85;
                        for line in chunk {
                            draw_code_block_line(
                                &mut page,
                                line,
                                &face,
                                fs,
                                block_left + pad_x,
                                line_y,
                            )?;
                            line_y -= lh;
                        }
                        cursor_y -= block_h + 4.0;
                        idx += chunk_len;
                    }
                }
                continue;
            }

            MarkdownBlock::DisplayMath { expr, source } => {
                layout.record_source(*source);
                cursor_y = draw_display_math_block(
                    &mut page,
                    expr,
                    &face,
                    cursor_y,
                    &mut doc,
                    &mut math_cache,
                    &mut layout,
                )?;
            }

            MarkdownBlock::Blockquote { lines, source } => {
                layout.record_source(*source);
                cursor_y = draw_blockquote(
                    &mut page,
                    lines,
                    &face,
                    cursor_y,
                    &mut doc,
                    &mut math_cache,
                    &mut layout,
                )?;
            }
        }
    }

    doc.add_page(page);
    let bytes = doc
        .to_bytes()
        .map_err(|e| format!("プレビューPDFの生成に失敗しました: {e}"))?;
    Ok((bytes, layout.line_pages))
}

fn estimate_rich_line_height(line: &[InlineSpan], font_size: f64) -> f64 {
    let base = if font_size >= 18.0 {
        (font_size * 1.2).max(BASE_LINE_HEIGHT)
    } else {
        BASE_LINE_HEIGHT
    };
    if line.iter().any(|span| span.style.math) {
        base.max(BASE_LINE_HEIGHT + 4.0)
    } else {
        base
    }
}

fn draw_rich_block_paginated(
    page: &mut OxidizePage,
    spans: &[InlineSpan],
    source: BlockSource,
    face: &PreviewFace,
    default_size: f64,
    x0: f64,
    cursor_y: &mut f64,
    max_cols: usize,
    math_cache: &mut MathRenderCache,
    doc: &mut Document,
    layout: &mut PreviewLayoutState,
) -> Result<(), String> {
    let lines = layout_spans_lines_with_source(spans, max_cols, source);
    for line in lines {
        let line_height = estimate_rich_line_height(&line.spans, default_size);
        layout.record_line(line.source_line);
        *cursor_y = ensure_room(*cursor_y, line_height, page, doc, layout);
        draw_rich_line_segments(
            page,
            &line.spans,
            face,
            default_size,
            x0,
            cursor_y,
            line_height,
            math_cache,
        )?;
    }
    Ok(())
}

fn layout_spans_lines(spans: &[InlineSpan], max_cols: usize) -> Vec<Vec<InlineSpan>> {
    layout_spans_lines_with_source(spans, max_cols, BlockSource::single(1))
        .into_iter()
        .map(|line| line.spans)
        .collect()
}

fn layout_spans_lines_with_source(
    spans: &[InlineSpan],
    max_cols: usize,
    source: BlockSource,
) -> Vec<LayoutLine> {
    let flat = flatten_spans(spans);
    if flat.is_empty() {
        return vec![LayoutLine {
            spans: vec![],
            source_line: source.start_line,
        }];
    }
    let mut lines: Vec<LayoutLine> = Vec::new();
    let mut cur_line: Vec<(String, TextStyle)> = Vec::new();
    let mut cur_width = 0usize;
    let mut current_source_line = source.start_line;

    let flush_line = |cur_line: &mut Vec<(String, TextStyle)>,
                          lines: &mut Vec<LayoutLine>,
                          current_source_line: u32| {
        if cur_line.is_empty() {
            return;
        }
        let mut out = Vec::new();
        for (t, st) in cur_line.drain(..) {
            InlineSpan::push(&mut out, &t, st);
        }
        merge_adjacent_spans(&mut out);
        lines.push(LayoutLine {
            spans: out,
            source_line: current_source_line,
        });
    };

    for token in flat {
        match token {
            LayoutToken::Break => {
                flush_line(&mut cur_line, &mut lines, current_source_line);
                cur_width = 0;
                if current_source_line < source.end_line {
                    current_source_line += 1;
                }
                continue;
            }
            LayoutToken::Char(ch, st) => {
                let w = char_display_width(ch);
                if cur_width + w > max_cols && cur_width > 0 {
                    flush_line(&mut cur_line, &mut lines, current_source_line);
                    cur_width = 0;
                }
                if let Some(last) = cur_line.last_mut() {
                    if last.1 == st {
                        last.0.push(ch);
                    } else {
                        cur_line.push((ch.to_string(), st));
                    }
                } else {
                    cur_line.push((ch.to_string(), st));
                }
                cur_width += w;
            }
            LayoutToken::Math(expr, st) => {
                let w = math_display_width(&expr);
                if cur_width + w > max_cols && cur_width > 0 {
                    flush_line(&mut cur_line, &mut lines, current_source_line);
                    cur_width = 0;
                }
                cur_line.push((expr, st));
                cur_width += w;
            }
        }
    }
    flush_line(&mut cur_line, &mut lines, current_source_line);
    if lines.is_empty() {
        lines.push(LayoutLine {
            spans: vec![],
            source_line: source.start_line,
        });
    }
    lines
}

enum LayoutToken {
    Break,
    Char(char, TextStyle),
    Math(String, TextStyle),
}

fn flatten_spans(spans: &[InlineSpan]) -> Vec<LayoutToken> {
    let mut v = Vec::new();
    for s in spans {
        if s.style.math {
            v.push(LayoutToken::Math(s.text.clone(), s.style));
            continue;
        }
        for ch in s.text.chars() {
            if ch == '\n' || ch == '\r' {
                v.push(LayoutToken::Break);
            } else {
                v.push(LayoutToken::Char(ch, s.style));
            }
        }
    }
    v
}

fn draw_rich_line_segments(
    page: &mut OxidizePage,
    line: &[InlineSpan],
    face: &PreviewFace,
    default_size: f64,
    x0: f64,
    cursor_y: &mut f64,
    line_height: f64,
    math_cache: &mut MathRenderCache,
) -> Result<(), String> {
    let y = *cursor_y;
    let mut x = x0;
    let mut consumed_height = line_height;
    for seg in line {
        let size = if seg.style.code {
            (default_size - 1.0).max(8.5)
        } else {
            default_size
        };
        let font = face.pick(seg.style, &seg.text);
        if seg.style.math {
            if let Some(rendered) = math_cache.render(&seg.text, false) {
                let max_height = (size * 1.45).max(line_height * 0.95);
                let (draw_width, draw_height) = scale_math_image(&rendered, max_height);
                consumed_height = consumed_height.max(draw_height + MATH_INLINE_EXTRA_LINE_PT);
                let baseline_offset = rendered.baseline_pt.max(draw_height * 0.45);
                let draw_y = y - baseline_offset - MATH_INLINE_Y_OFFSET_PT;
                draw_image_on_page(
                    page,
                    &rendered.jpeg_data,
                    "jpeg",
                    x,
                    draw_y,
                    draw_width,
                    draw_height,
                )?;
                x += draw_width;
                continue;
            }
        }
        let rendered_text = seg.text.clone();
        if rendered_text.is_empty() {
            continue;
        }
        if seg.style.code {
            let w = approx_width_pt(&rendered_text, &font, size).max(size * 0.6);
            let pad_y = size * 0.25;
            let box_h = size * 1.2;
            let box_bottom = y - pad_y;
            page.graphics()
                .set_fill_color(Color::rgb(0.91, 0.93, 0.96))
                .rectangle(x - 1.5, box_bottom - 1.0, w + 3.0, box_h + 2.0)
                .fill();
        }
        let fill = if seg.style.link {
            Color::rgb(0.13, 0.34, 0.82)
        } else if seg.style.math {
            Color::rgb(0.12, 0.12, 0.18)
        } else if seg.style.bold && !ascii_only(&seg.text) && !face.has_custom_bold {
            Color::rgb(0.04, 0.04, 0.07)
        } else if seg.style.code {
            Color::rgb(0.08, 0.1, 0.14)
        } else if seg.style.italic {
            Color::rgb(0.15, 0.16, 0.2)
        } else {
            Color::rgb(0.02, 0.02, 0.04)
        };
        let draw_width =
            draw_text_segment(page, face, &rendered_text, seg, &font, size, x, y, fill)?;
        if seg.style.link {
            let underline_y = y - (size * 0.12).max(1.0);
            page.graphics()
                .set_stroke_color(fill)
                .set_line_width(0.65)
                .move_to(x, underline_y)
                .line_to(x + draw_width, underline_y)
                .stroke();
        }
        x += draw_width;
    }
    *cursor_y -= consumed_height;
    Ok(())
}

fn draw_text_segment(
    page: &mut OxidizePage,
    face: &PreviewFace,
    rendered_text: &str,
    seg: &InlineSpan,
    font: &Font,
    size: f64,
    x: f64,
    y: f64,
    fill: Color,
) -> Result<f64, String> {
    let needs_fake_bold = is_potential_bold_face(face, seg.style, &seg.text);
    let needs_fake_italic = should_fake_italic(face, seg.style, &seg.text);
    let width = measure_text_segment_width(face, rendered_text, seg.style, font, size);

    if should_raster_faux_italic(face, seg.style, rendered_text) {
        if let Some(rendered) =
            rasterize_faux_italic_text(face, rendered_text, seg.style, size, fill)?
        {
            let draw_y = y - rendered.baseline_offset_pt;
            draw_image_on_page(
                page,
                &rendered.jpeg_data,
                "jpeg",
                x,
                draw_y,
                rendered.width_pt,
                rendered.height_pt,
            )?;
            return Ok(raster_faux_italic_advance_width(&rendered));
        }
    }

    if needs_fake_italic && !text_has_emoji(rendered_text) {
        let draw_width = draw_faux_italic_text(
            page,
            rendered_text,
            font.clone(),
            size,
            x,
            y,
            fill,
            seg.style.bold,
        )?;
        return Ok(draw_width.max(width));
    }

    if needs_fake_bold && !text_has_emoji(rendered_text) {
        draw_fake_bold_text(page, rendered_text, font.clone(), size, x, y, fill)?;
        return Ok(width);
    }

    if text_has_emoji(rendered_text) && !seg.style.code {
        draw_mixed_emoji_text(page, face, rendered_text, seg, size, x, y, fill)?;
        return Ok(width);
    }

    page.text()
        .set_fill_color(fill)
        .set_font(font.clone(), size)
        .at(x, y)
        .write(rendered_text)
        .map_err(|e| format!("テキスト描画に失敗しました: {e}"))?;
    Ok(width)
}

fn draw_mixed_emoji_text(
    page: &mut OxidizePage,
    face: &PreviewFace,
    text: &str,
    seg: &InlineSpan,
    size: f64,
    x: f64,
    y: f64,
    fill: Color,
) -> Result<(), String> {
    let mut cursor_x = x;
    for grapheme in UnicodeSegmentation::graphemes(text, true) {
        let draw_size = if seg.style.code {
            (size - 1.0).max(8.5)
        } else {
            size
        };
        if grapheme_has_emoji(grapheme) {
            if let Some((png_bytes, width_pt, height_pt, baseline_offset_pt)) =
                rasterize_emoji_grapheme(face, grapheme, draw_size)?
            {
                let display_width = width_pt * EMOJI_DISPLAY_SCALE;
                let display_height = height_pt * EMOJI_DISPLAY_SCALE;
                let emoji_y = emoji_draw_y(y, baseline_offset_pt);
                draw_image_on_page(
                    page,
                    &png_bytes,
                    "jpeg",
                    cursor_x,
                    emoji_y,
                    display_width,
                    display_height,
                )?;
                cursor_x += display_width;
                continue;
            }
        }
        let font = font_for_grapheme(face, seg.style, grapheme);
        let w = approx_width_pt(grapheme, &font, draw_size);
        let needs_fake_bold = is_potential_bold_face(face, seg.style, grapheme);
        if should_fake_italic_grapheme(face, seg.style, grapheme) {
            let draw_width = draw_faux_italic_text(
                page,
                grapheme,
                font.clone(),
                draw_size,
                cursor_x,
                y,
                fill,
                seg.style.bold,
            )?;
            cursor_x += draw_width.max(w);
            continue;
        }
        if needs_fake_bold {
            draw_fake_bold_text(page, grapheme, font.clone(), draw_size, cursor_x, y, fill)?;
            cursor_x += w;
            continue;
        }
        page.text()
            .set_fill_color(fill)
            .set_font(font, draw_size)
            .at(cursor_x, y)
            .write(grapheme)
            .map_err(|e| format!("テキスト描画に失敗しました: {e}"))?;
        cursor_x += w;
    }
    Ok(())
}

fn measure_text_segment_width(
    face: &PreviewFace,
    text: &str,
    style: TextStyle,
    font: &Font,
    size: f64,
) -> f64 {
    if !text_has_emoji(text) || style.code || style.math {
        return approx_width_pt(text, font, size);
    }

    UnicodeSegmentation::graphemes(text, true)
        .map(|grapheme| {
            if grapheme_has_emoji(grapheme) {
                emoji_grapheme_width_pt(face, grapheme, size)
            } else {
                let grapheme_font = font_for_grapheme(face, style, grapheme);
                approx_width_pt(grapheme, &grapheme_font, size)
            }
        })
        .sum()
}

fn emoji_grapheme_width_pt(face: &PreviewFace, grapheme: &str, font_size: f64) -> f64 {
    if let Some((_, width_pt, _, _)) = rasterize_emoji_grapheme(face, grapheme, font_size)
        .ok()
        .flatten()
    {
        return (width_pt * EMOJI_DISPLAY_SCALE).max(font_size * 0.85);
    }
    font_size * EMOJI_DISPLAY_SCALE
}

fn font_for_grapheme(face: &PreviewFace, style: TextStyle, grapheme: &str) -> Font {
    if style.code {
        if ascii_only(grapheme) {
            return Font::Courier;
        }
        return face.body.clone();
    }
    if style.math {
        if ascii_only(grapheme) {
            return Font::TimesItalic;
        }
        return face.body.clone();
    }
    if grapheme_has_emoji(grapheme) {
        if let Some(font) = &face.emoji {
            return font.clone();
        }
    }
    face.pick(style, grapheme)
}

fn text_has_emoji(text: &str) -> bool {
    UnicodeSegmentation::graphemes(text, true).any(grapheme_has_emoji)
}

fn grapheme_has_emoji(grapheme: &str) -> bool {
    grapheme.chars().any(|ch| {
        matches!(
            ch,
            '\u{1F1E6}'..='\u{1F1FF}'
                | '\u{1F300}'..='\u{1FAFF}'
                | '\u{2600}'..='\u{26FF}'
                | '\u{2700}'..='\u{27BF}'
                | '\u{FE0F}'
                | '\u{200D}'
        )
    })
}

fn font_data(path: &Path) -> Option<Arc<[u8]>> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Arc<[u8]>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache.lock().ok()?;
    if let Some(data) = cache.get(path) {
        return Some(Arc::clone(data));
    }
    let bytes = Arc::<[u8]>::from(std::fs::read(path).ok()?.into_boxed_slice());
    cache.insert(path.to_path_buf(), Arc::clone(&bytes));
    Some(bytes)
}

fn rasterize_emoji_grapheme(
    face: &PreviewFace,
    grapheme: &str,
    font_size: f64,
) -> Result<Option<(Vec<u8>, f64, f64, f64)>, String> {
    let Some(path) = &face.emoji_path else {
        return Ok(None);
    };
    let Some(data) = font_data(path) else {
        return Ok(None);
    };
    let Some(font) = FontRef::from_index(data.as_ref(), 0) else {
        return Ok(None);
    };

    let Some(ch) = grapheme.chars().next() else {
        return Ok(None);
    };
    let glyph_id = font.charmap().map(ch);
    if glyph_id == 0 {
        return Ok(None);
    }

    let mut context = ScaleContext::new();
    let mut scaler = context
        .builder(font)
        .size((font_size as f32) * EMOJI_RASTER_PX_PER_PT)
        .build();
    let image = Render::new(&[
        Source::ColorOutline(0),
        Source::ColorBitmap(StrikeWith::BestFit),
        Source::Outline,
    ])
    .format(Format::Alpha)
    .render(&mut scaler, glyph_id);
    let Some(image) = image else {
        return Ok(None);
    };

    let width = image.placement.width.max(1);
    let height = image.placement.height.max(1);
    let mut flattened = image::RgbImage::from_pixel(width, height, image::Rgb([255, 255, 255]));
    match image.content {
        SwashContent::Mask => {
            for (idx, alpha) in image.data.iter().copied().enumerate() {
                let x = (idx as u32) % width;
                let y = (idx as u32) / width;
                let shade = 255u8.saturating_sub(alpha);
                flattened.put_pixel(x, y, image::Rgb([shade, shade, shade]));
            }
        }
        SwashContent::Color | SwashContent::SubpixelMask => {
            for (idx, rgba) in image.data.chunks_exact(4).enumerate() {
                let x = (idx as u32) % width;
                let y = (idx as u32) / width;
                let r = rgba[0] as f32;
                let g = rgba[1] as f32;
                let b = rgba[2] as f32;
                let a = (rgba[3] as f32 / 255.0).clamp(0.0, 1.0);
                let blend = |src: f32| -> u8 {
                    ((src * a) + 255.0 * (1.0 - a)).round().clamp(0.0, 255.0) as u8
                };
                flattened.put_pixel(x, y, image::Rgb([blend(r), blend(g), blend(b)]));
            }
        }
    }

    let mut jpeg = Vec::new();
    DynamicImage::ImageRgb8(flattened)
        .write_to(
            &mut std::io::Cursor::new(&mut jpeg),
            ImageOutputFormat::Jpeg(85),
        )
        .map_err(|e| format!("絵文字画像の書き出しに失敗しました: {e}"))?;

    Ok(Some((
        jpeg,
        width as f64 / EMOJI_RASTER_PX_PER_PT as f64,
        height as f64 / EMOJI_RASTER_PX_PER_PT as f64,
        emoji_baseline_offset_pt(image.placement.top, image.placement.height as i32),
    )))
}

fn emoji_baseline_offset_pt(placement_top_px: i32, placement_height_px: i32) -> f64 {
    let baseline_from_bottom_px = placement_height_px.saturating_sub(placement_top_px).max(0);
    (baseline_from_bottom_px as f64 / EMOJI_RASTER_PX_PER_PT as f64) * EMOJI_DISPLAY_SCALE
}

fn emoji_draw_y(y: f64, baseline_offset_pt: f64) -> f64 {
    y - baseline_offset_pt - EMOJI_VERTICAL_ADJUST_PT
}

struct RasterizedFauxItalicText {
    jpeg_data: Vec<u8>,
    width_pt: f64,
    height_pt: f64,
    baseline_offset_pt: f64,
}

struct GlyphBitmap {
    data: Vec<u8>,
    is_mask: bool,
    width: u32,
    height: u32,
    left: i32,
    top: i32,
    advance: f32,
}

fn should_raster_faux_italic(face: &PreviewFace, style: TextStyle, text: &str) -> bool {
    style.italic
        && !style.code
        && !style.math
        && !ascii_only(text)
        && !text_has_emoji(text)
        && face.body_path.is_some()
}

fn raster_faux_italic_advance_width(rendered: &RasterizedFauxItalicText) -> f64 {
    rendered.width_pt
}

fn raster_faux_italic_baseline_offset(height_pt: f64, baseline_from_top_pt: f64) -> f64 {
    (height_pt - baseline_from_top_pt).max(0.0)
}

fn rasterize_faux_italic_text(
    face: &PreviewFace,
    text: &str,
    style: TextStyle,
    size: f64,
    fill: Color,
) -> Result<Option<RasterizedFauxItalicText>, String> {
    let font_path = if style.bold {
        face.body_bold_path.as_ref().or(face.body_path.as_ref())
    } else {
        face.body_path.as_ref()
    };
    let Some(font_path) = font_path else {
        return Ok(None);
    };
    let Some(data) = font_data(font_path) else {
        return Ok(None);
    };
    let Some(font) = FontRef::from_index(data.as_ref(), 0) else {
        return Ok(None);
    };

    let pixels_per_pt = EMOJI_RASTER_PX_PER_PT;
    let px_size = (size as f32) * pixels_per_pt;
    let mut context = ScaleContext::new();
    let mut scaler = context.builder(font).size(px_size).build();
    let metrics = font.glyph_metrics(&[]).scale(px_size);
    let mut glyphs = Vec::new();
    let mut top_max = 0_i32;
    let mut bottom_max = 0_i32;
    let mut advance_total = 0.0_f32;

    for ch in text.chars() {
        let glyph_id = font.charmap().map(ch);
        if glyph_id == 0 {
            return Ok(None);
        }
        let Some(image) = Render::new(&[Source::Outline])
            .format(Format::Alpha)
            .render(&mut scaler, glyph_id)
        else {
            return Ok(None);
        };
        let width = image.placement.width.max(1);
        let height = image.placement.height.max(1);
        let top = image.placement.top;
        let left = image.placement.left;
        let is_mask = matches!(image.content, SwashContent::Mask);
        top_max = top_max.max(top);
        bottom_max = bottom_max.max(height as i32 - top);
        let advance = metrics.advance_width(glyph_id).max(px_size * 0.5);
        advance_total += advance;
        glyphs.push(GlyphBitmap {
            data: image.data,
            is_mask,
            width,
            height,
            left,
            top,
            advance,
        });
    }

    if glyphs.is_empty() {
        return Ok(None);
    }

    let pad = (px_size * 0.35).ceil() as u32;
    let baseline = top_max.max(0) as u32 + pad;
    let canvas_height = (top_max.max(0) + bottom_max.max(0)) as u32 + pad * 2;
    let canvas_width = advance_total.ceil() as u32 + pad * 2 + (px_size * 0.8).ceil() as u32;
    let mut canvas = image::RgbImage::from_pixel(
        canvas_width.max(1),
        canvas_height.max(1),
        image::Rgb([255, 255, 255]),
    );
    let fill_rgb = fill_rgb(fill);
    let fake_bold = style.bold && !face.has_custom_bold;
    let bold_offsets: &[u32] = if fake_bold { &[0, 1, 2] } else { &[0] };
    let mut pen_x = pad as f32;

    for glyph in &glyphs {
        let base_x = pen_x.round() as i32 + glyph.left;
        let base_y = baseline as i32 - glyph.top;
        for dx in bold_offsets {
            blend_glyph_bitmap(&mut canvas, glyph, base_x + *dx as i32, base_y, fill_rgb);
        }
        pen_x += glyph.advance;
    }

    let slanted = shear_text_image(&canvas, 0.22);
    let (trimmed, trim_top) = trim_white_text_image(&slanted, 2);
    let width_pt = trimmed.width() as f64 / pixels_per_pt as f64;
    let height_pt = trimmed.height() as f64 / pixels_per_pt as f64;
    let baseline_from_top_pt = (baseline as i32 - trim_top).max(0) as f64 / pixels_per_pt as f64;
    let baseline_offset_pt = raster_faux_italic_baseline_offset(height_pt, baseline_from_top_pt);
    let mut jpeg = Vec::new();
    DynamicImage::ImageRgb8(trimmed)
        .write_to(
            &mut std::io::Cursor::new(&mut jpeg),
            ImageOutputFormat::Jpeg(90),
        )
        .map_err(|e| format!("斜体テキスト画像の書き出しに失敗しました: {e}"))?;

    Ok(Some(RasterizedFauxItalicText {
        jpeg_data: jpeg,
        width_pt,
        height_pt,
        baseline_offset_pt,
    }))
}

fn fill_rgb(fill: Color) -> [u8; 3] {
    [
        (fill.r() * 255.0).round().clamp(0.0, 255.0) as u8,
        (fill.g() * 255.0).round().clamp(0.0, 255.0) as u8,
        (fill.b() * 255.0).round().clamp(0.0, 255.0) as u8,
    ]
}

fn blend_glyph_bitmap(
    canvas: &mut image::RgbImage,
    glyph: &GlyphBitmap,
    origin_x: i32,
    origin_y: i32,
    fill: [u8; 3],
) {
    for y in 0..glyph.height {
        for x in 0..glyph.width {
            let alpha = if glyph.is_mask {
                glyph.data[(y * glyph.width + x) as usize]
            } else {
                let idx = ((y * glyph.width + x) * 4 + 3) as usize;
                glyph.data.get(idx).copied().unwrap_or(0)
            };
            if alpha == 0 {
                continue;
            }
            let dst_x = origin_x + x as i32;
            let dst_y = origin_y + y as i32;
            if dst_x < 0
                || dst_y < 0
                || dst_x >= canvas.width() as i32
                || dst_y >= canvas.height() as i32
            {
                continue;
            }
            let pixel = canvas.get_pixel_mut(dst_x as u32, dst_y as u32);
            let a = alpha as f32 / 255.0;
            for channel in 0..3 {
                pixel.0[channel] = ((fill[channel] as f32 * a)
                    + (pixel.0[channel] as f32 * (1.0 - a)))
                    .round()
                    .clamp(0.0, 255.0) as u8;
            }
        }
    }
}

fn shear_text_image(image: &image::RgbImage, shear: f32) -> image::RgbImage {
    let extra = ((image.height() as f32) * shear).ceil() as u32;
    let mut out = image::RgbImage::from_pixel(
        image.width() + extra,
        image.height(),
        image::Rgb([255, 255, 255]),
    );
    for y in 0..image.height() {
        let dx = (((image.height() - 1 - y) as f32) * shear).round() as u32;
        for x in 0..image.width() {
            out.put_pixel(x + dx, y, *image.get_pixel(x, y));
        }
    }
    out
}

fn trim_white_text_image(image: &image::RgbImage, pad: u32) -> (image::RgbImage, i32) {
    let mut min_x = image.width();
    let mut min_y = image.height();
    let mut max_x = 0_u32;
    let mut max_y = 0_u32;

    for y in 0..image.height() {
        for x in 0..image.width() {
            let pixel = image.get_pixel(x, y);
            if pixel.0.iter().any(|channel| *channel < 248) {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }

    if min_x > max_x || min_y > max_y {
        return (image.clone(), 0);
    }

    let crop_x = min_x.saturating_sub(pad);
    let crop_y = min_y.saturating_sub(pad);
    let crop_right = (max_x + pad).min(image.width().saturating_sub(1));
    let crop_bottom = (max_y + pad).min(image.height().saturating_sub(1));
    let width = crop_right - crop_x + 1;
    let height = crop_bottom - crop_y + 1;
    let cropped = image.view(crop_x, crop_y, width, height).to_image();
    (cropped, crop_y as i32)
}

fn draw_faux_italic_text(
    page: &mut OxidizePage,
    text: &str,
    font: Font,
    size: f64,
    x: f64,
    y: f64,
    fill: Color,
    fake_bold: bool,
) -> Result<f64, String> {
    draw_faux_italic_text_fallback(page, text, font.clone(), size, x, y, fill, fake_bold)
        .map(|draw_width| draw_width.max(approx_width_pt(text, &font, size)))
}

fn draw_faux_italic_text_fallback(
    page: &mut OxidizePage,
    text: &str,
    font: Font,
    size: f64,
    x: f64,
    y: f64,
    fill: Color,
    fake_bold: bool,
) -> Result<f64, String> {
    let mut cursor_x = x;
    for (idx, grapheme) in UnicodeSegmentation::graphemes(text, true).enumerate() {
        let draw_x =
            cursor_x + faux_italic_skew(idx, size) + faux_italic_bold_slant(idx, size, fake_bold);
        let grapheme_width = approx_width_pt(grapheme, &font, size);
        if fake_bold {
            draw_fake_bold_text(page, grapheme, font.clone(), size, draw_x, y, fill)?;
        } else {
            page.text()
                .set_fill_color(fill)
                .set_font(font.clone(), size)
                .at(draw_x, y)
                .write(grapheme)
                .map_err(|e| format!("テキスト描画に失敗しました: {e}"))?;
        }
        cursor_x += grapheme_width;
    }
    Ok(faux_italic_advance_width(text, &font, size, fake_bold))
}

fn faux_italic_skew(index: usize, size: f64) -> f64 {
    (index as f64 * (size * 0.15)).min(size * 0.60)
}

fn faux_italic_bold_slant(index: usize, size: f64, fake_bold: bool) -> f64 {
    if fake_bold {
        index as f64 * (size * 0.05)
    } else {
        0.0
    }
}

fn faux_italic_advance_width(text: &str, font: &Font, size: f64, fake_bold: bool) -> f64 {
    let mut cursor_x = 0.0_f64;
    let mut max_right = 0.0_f64;
    for (idx, grapheme) in UnicodeSegmentation::graphemes(text, true).enumerate() {
        let grapheme_width = approx_width_pt(grapheme, font, size);
        let draw_x =
            cursor_x + faux_italic_skew(idx, size) + faux_italic_bold_slant(idx, size, fake_bold);
        max_right = max_right.max(draw_x + grapheme_width);
        cursor_x += grapheme_width;
    }
    max_right.max(cursor_x)
}

fn code_block_fits_on_single_page(block_h: f64) -> bool {
    block_h + 8.0 <= PAGE_HEIGHT - 2.0 * MARGIN
}

fn draw_code_block_line(
    page: &mut OxidizePage,
    line: &str,
    face: &PreviewFace,
    size: f64,
    x: f64,
    y: f64,
) -> Result<(), String> {
    let font = face.pick(
        TextStyle {
            code: true,
            math: false,
            ..TextStyle::default()
        },
        line,
    );
    page.text()
        .set_fill_color(Color::rgb(0.08, 0.1, 0.14))
        .set_font(font, size)
        .at(x, y)
        .write(line)
        .map(|_| ())
        .map_err(|e| format!("テキスト描画に失敗しました: {e}"))
}

fn approx_width_pt(text: &str, font: &Font, font_size: f64) -> f64 {
    match font {
        Font::Helvetica
        | Font::HelveticaBold
        | Font::HelveticaOblique
        | Font::HelveticaBoldOblique
        | Font::Courier
        | Font::CourierBold
        | Font::CourierOblique
        | Font::CourierBoldOblique => {
            oxidize_pdf::text::measure_text(text, font.clone(), font_size)
        }
        Font::TimesRoman | Font::TimesBold | Font::TimesItalic | Font::TimesBoldItalic => {
            oxidize_pdf::text::measure_text(text, font.clone(), font_size)
        }
        Font::Custom(_) => {
            let cols: f64 = text.chars().map(|c| char_display_width(c) as f64).sum();
            cols * (font_size * 0.52)
        }
        _ => {
            let cols: f64 = text.chars().map(|c| char_display_width(c) as f64).sum();
            cols * (font_size * 0.52)
        }
    }
}

fn list_marker_x(indent: u8) -> f64 {
    LIST_MARKER_X + f64::from(indent) * LIST_INDENT_STEP_PT
}

fn list_item_content_x(indent: u8, marker: &str, font: &Font, font_size: f64) -> f64 {
    list_marker_x(indent) + approx_width_pt(marker, font, font_size) + LIST_MARKER_TEXT_GAP_PT
}

fn should_fake_italic(_face: &PreviewFace, style: TextStyle, text: &str) -> bool {
    style.italic && !ascii_only(text)
}

fn should_fake_italic_grapheme(_face: &PreviewFace, style: TextStyle, grapheme: &str) -> bool {
    style.italic && !ascii_only(grapheme) && !grapheme_has_emoji(grapheme)
}

fn wrap_table_row_cells(row: &[Vec<InlineSpan>], max_cell_cols: usize) -> Vec<Vec<String>> {
    row.iter()
        .map(|cell| {
            let text = cell.iter().map(|s| s.text.as_str()).collect::<String>();
            wrap_text(&text, max_cell_cols)
        })
        .collect()
}

/// 表全体を一度に罫線で囲み、行ごとの縦線の重ね描きを避ける（ずれ・破線の主因だった）。
fn draw_table(
    page: &mut OxidizePage,
    rows: &[Vec<Vec<InlineSpan>>],
    face: &PreviewFace,
    cursor_y: &mut f64,
    _row_h: f64,
) -> Result<(), String> {
    if rows.is_empty() {
        return Ok(());
    }
    let fs = 9.5;
    let line_height = 11.0;
    let row_padding_top = 5.0;
    let row_padding_bottom = 4.0;
    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(1).max(1);
    let nrows = rows.len();
    let table_left = MARGIN + 4.0;
    let table_width = PAGE_WIDTH - 2.0 * MARGIN - 12.0;
    let col_w = table_width / ncols as f64;
    let max_cell_cols = ((col_w - 8.0) / (fs * 0.52)).max(3.0).floor() as usize;

    let table_top = *cursor_y;
    let wrapped_rows: Vec<Vec<Vec<String>>> = rows
        .iter()
        .map(|row| wrap_table_row_cells(row, max_cell_cols))
        .collect();
    let row_heights: Vec<f64> = wrapped_rows
        .iter()
        .map(|cells| {
            let max_lines = cells
                .iter()
                .map(|cell| cell.len().max(1))
                .max()
                .unwrap_or(1);
            row_padding_top + row_padding_bottom + max_lines as f64 * line_height
        })
        .collect();
    let table_height: f64 = row_heights.iter().sum();
    let table_bottom = table_top - table_height;

    for ri in 0..=nrows {
        let y = if ri == 0 {
            table_top
        } else {
            table_top - row_heights.iter().take(ri).sum::<f64>()
        };
        page.graphics()
            .set_stroke_color(Color::gray(0.42))
            .set_line_width(0.45)
            .move_to(table_left, y)
            .line_to(table_left + table_width, y)
            .stroke();
    }
    for ci in 0..=ncols {
        let x = table_left + ci as f64 * col_w;
        page.graphics()
            .set_stroke_color(Color::gray(0.42))
            .set_line_width(0.45)
            .move_to(x, table_bottom)
            .line_to(x, table_top)
            .stroke();
    }

    let mut row_top = table_top;
    for (ri, row) in wrapped_rows.iter().enumerate() {
        let row_height = row_heights[ri];
        let baseline = row_top - row_padding_top - fs * 0.85;
        for ci in 0..ncols {
            let cell_lines = row.get(ci).cloned().unwrap_or_else(|| vec![String::new()]);
            let mut line_y = baseline;
            for line in cell_lines {
                page.text()
                    .set_fill_color(Color::rgb(0.02, 0.02, 0.04))
                    .set_font(face.body.clone(), fs)
                    .at(table_left + col_w * ci as f64 + 4.0, line_y)
                    .write(&line)
                    .map_err(|e| format!("テキスト描画に失敗しました: {e}"))?;
                line_y -= line_height;
            }
        }
        row_top -= row_height;
    }

    *cursor_y = table_bottom - 6.0;
    Ok(())
}

fn draw_blockquote(
    page: &mut OxidizePage,
    lines: &[BlockquoteLine],
    face: &PreviewFace,
    cursor_y: f64,
    doc: &mut Document,
    math_cache: &mut MathRenderCache,
    layout: &mut PreviewLayoutState,
) -> Result<f64, String> {
    let top_pad = 12.0;
    let bottom_pad = 10.0;
    let body_w = blockquote_body_width(lines, face, 11.0);
    let text_x = MARGIN + 10.0;
    let mut index = 0usize;
    let mut page_cursor = cursor_y;

    while index < lines.len() {
        let mut segment_end = index;
        let mut segment_height = top_pad + bottom_pad;
        let mut segment_line_heights = Vec::new();

        while segment_end < lines.len() {
            let line_height = blockquote_line_height(&lines[segment_end], math_cache);
            let tentative_height = segment_height + line_height;
            if segment_end > index && page_cursor - (tentative_height + 6.0) <= MARGIN {
                break;
            }
            segment_line_heights.push(line_height);
            segment_height = tentative_height;
            segment_end += 1;
        }

        if segment_end == index {
            let line_height = blockquote_line_height(&lines[segment_end], math_cache);
            segment_line_heights.push(line_height);
            segment_height = top_pad + bottom_pad + line_height;
            segment_end += 1;
        }

        page_cursor = ensure_room(page_cursor, segment_height + 6.0, page, doc, layout);
        let y_top = page_cursor;
        let y_bottom = y_top - segment_height;

        page.graphics()
            .set_fill_color(Color::rgb(0.96, 0.97, 0.99))
            .rectangle(MARGIN + 6.0, y_bottom, body_w, segment_height)
            .fill();
        page.graphics()
            .set_fill_color(Color::rgb(0.45, 0.55, 0.78))
            .rectangle(MARGIN + 2.0, y_bottom, 4.5, segment_height)
            .fill();

        let mut quoted_cursor = y_top - top_pad;
        for (line, line_height) in lines[index..segment_end]
            .iter()
            .zip(segment_line_heights.iter())
        {
            match line {
                BlockquoteLine::Text(spans) => {
                    draw_rich_line_segments(
                        page,
                        spans,
                        face,
                        11.0,
                        text_x,
                        &mut quoted_cursor,
                        *line_height,
                        math_cache,
                    )?;
                }
                BlockquoteLine::DisplayMath(expr) => {
                    draw_blockquote_display_math_line(
                        page,
                        expr,
                        body_w,
                        text_x,
                        &mut quoted_cursor,
                        *line_height,
                        face,
                        math_cache,
                    )?;
                }
                BlockquoteLine::CodeBlock { code } => {
                    draw_blockquote_code_block(
                        page,
                        code,
                        body_w,
                        text_x,
                        &mut quoted_cursor,
                        *line_height,
                        face,
                    )?;
                }
            }
        }
        page_cursor = y_bottom - 10.0;
        index = segment_end;
    }

    Ok(page_cursor)
}

fn blockquote_line_height(line: &BlockquoteLine, math_cache: &mut MathRenderCache) -> f64 {
    match line {
        BlockquoteLine::Text(spans) => layout_spans_lines(spans, blockquote_wrap_cols())
            .iter()
            .map(|line| blockquote_wrapped_text_line_height(line, math_cache))
            .sum(),
        BlockquoteLine::DisplayMath(expr) => {
            if let Some(rendered) = math_cache.render(expr, true) {
                rendered.height_pt + 8.0
            } else {
                26.0
            }
        }
        BlockquoteLine::CodeBlock { code } => {
            let code_lines = wrap_code_lines(code, 68);
            let line_height = (BASE_LINE_HEIGHT - 1.0).max(13.0);
            code_lines.len().max(1) as f64 * line_height + 18.0
        }
    }
}

fn blockquote_wrap_cols() -> usize {
    96
}

fn blockquote_wrapped_text_line_height(
    line: &[InlineSpan],
    math_cache: &mut MathRenderCache,
) -> f64 {
    let mut height = BASE_LINE_HEIGHT;
    for seg in line {
        if !seg.style.math {
            continue;
        }
        if let Some(rendered) = math_cache.render(&seg.text, false) {
            height = height.max(rendered.height_pt + MATH_INLINE_EXTRA_LINE_PT);
        } else {
            height = height.max(BASE_LINE_HEIGHT + 4.0);
        }
    }
    height
}

fn draw_blockquote_display_math_line(
    page: &mut OxidizePage,
    expr: &str,
    body_w: f64,
    text_x: f64,
    cursor_y: &mut f64,
    line_height: f64,
    _face: &PreviewFace,
    math_cache: &mut MathRenderCache,
) -> Result<(), String> {
    let Some(rendered) = math_cache.render(expr, true) else {
        *cursor_y -= line_height;
        return Ok(());
    };

    let max_height = rendered.height_pt.max(line_height - 2.0);
    let (draw_width, draw_height) = scale_math_image(&rendered, max_height);
    let draw_x = text_x + ((body_w - 20.0) - draw_width).max(0.0) / 2.0;
    let draw_y = *cursor_y - draw_height * 0.72;
    draw_image_on_page(
        page,
        &rendered.jpeg_data,
        "jpeg",
        draw_x,
        draw_y,
        draw_width,
        draw_height,
    )?;
    *cursor_y -= line_height;
    Ok(())
}

fn draw_blockquote_code_block(
    page: &mut OxidizePage,
    code: &str,
    body_w: f64,
    text_x: f64,
    cursor_y: &mut f64,
    line_height: f64,
    face: &PreviewFace,
) -> Result<(), String> {
    let code_lines = wrap_code_lines(code, 68);
    let pad_x = 8.0;
    let pad_y = 7.0;
    let lh = (BASE_LINE_HEIGHT - 1.0).max(13.0);
    let block_w = (body_w - 20.0).max(90.0);
    let block_h = code_lines.len().max(1) as f64 * lh + pad_y * 2.0;
    let block_bottom = *cursor_y - block_h + 2.0;
    page.graphics()
        .set_fill_color(Color::rgb(0.94, 0.95, 0.98))
        .rectangle(text_x, block_bottom, block_w, block_h)
        .fill();
    page.graphics()
        .set_stroke_color(Color::rgb(0.84, 0.87, 0.94))
        .set_line_width(0.6)
        .rectangle(text_x, block_bottom, block_w, block_h)
        .stroke();

    let mut baseline = *cursor_y - pad_y - 9.5 * 0.85;
    for line in code_lines {
        draw_code_block_line(page, &line, face, 9.5, text_x + pad_x, baseline)?;
        baseline -= lh;
    }
    *cursor_y -= line_height;
    Ok(())
}

fn draw_thematic_break(
    page: &mut OxidizePage,
    cursor_y: f64,
    style: ThematicBreakStyle,
    doc: &mut Document,
    layout: &mut PreviewLayoutState,
) -> f64 {
    let h = 18.0_f64;
    let cursor_y = ensure_room(cursor_y, h, page, doc, layout);
    let y = cursor_y - h * 0.5;
    let x0 = MARGIN + 10.0;
    let x1 = PAGE_WIDTH - MARGIN - 10.0;

    match style {
        ThematicBreakStyle::Hyphen => {
            page.graphics()
                .set_stroke_color(Color::gray(0.72))
                .set_line_width(1.15)
                .move_to(x0, y)
                .line_to(x1, y)
                .stroke();
        }
        ThematicBreakStyle::Asterisk => {
            draw_dashed_horizontal_line(page, x0, x1, y, 10.0, 6.0, 1.15, Color::gray(0.64));
        }
        ThematicBreakStyle::Underscore => {
            let gap = 2.2;
            page.graphics()
                .set_stroke_color(Color::gray(0.68))
                .set_line_width(0.95)
                .move_to(x0, y - gap)
                .line_to(x1, y - gap)
                .stroke();
            page.graphics()
                .set_stroke_color(Color::gray(0.68))
                .set_line_width(0.95)
                .move_to(x0, y + gap)
                .line_to(x1, y + gap)
                .stroke();
        }
    }

    cursor_y - h
}

fn draw_dashed_horizontal_line(
    page: &mut OxidizePage,
    x0: f64,
    x1: f64,
    y: f64,
    dash_len: f64,
    gap_len: f64,
    line_width: f64,
    color: Color,
) {
    let mut x = x0;
    while x < x1 {
        let end = (x + dash_len).min(x1);
        page.graphics()
            .set_stroke_color(color)
            .set_line_width(line_width)
            .move_to(x, y)
            .line_to(end, y)
            .stroke();
        x = end + gap_len;
    }
}

fn draw_display_math_block(
    page: &mut OxidizePage,
    expr: &str,
    face: &PreviewFace,
    cursor_y: f64,
    doc: &mut Document,
    math_cache: &mut MathRenderCache,
    layout: &mut PreviewLayoutState,
) -> Result<f64, String> {
    let font_size = 12.5;
    let top_pad = 12.0;
    let bottom_pad = 12.0;
    if let Some(rendered) = math_cache.render(expr, true) {
        let content_max_height = (PAGE_HEIGHT / 3.2).max(rendered.height_pt);
        let (draw_width, draw_height) = scale_math_image(&rendered, content_max_height);
        let block_h = draw_height + top_pad + bottom_pad;
        let mut page_cursor = ensure_room(cursor_y, block_h + 6.0, page, doc, layout);
        let y_top = page_cursor;
        let y_bottom = y_top - block_h;
        let draw_x = MARGIN + ((PAGE_WIDTH - 2.0 * MARGIN) - draw_width) / 2.0;
        let draw_y = y_bottom + bottom_pad + 1.0;
        draw_image_on_page(
            page,
            &rendered.jpeg_data,
            "jpeg",
            draw_x,
            draw_y,
            draw_width,
            draw_height,
        )?;

        page_cursor = y_bottom - 8.0;
        return Ok(page_cursor);
    }
    let lines = format_math_expression(expr);
    let line_height = 18.0;
    let block_h = lines.len().max(1) as f64 * line_height + top_pad + bottom_pad;
    let mut page_cursor = ensure_room(cursor_y, block_h + 6.0, page, doc, layout);
    let y_top = page_cursor;
    let y_bottom = y_top - block_h;

    let mut baseline = y_top - top_pad - font_size * 0.55;
    for line in lines {
        let font = face.pick(
            TextStyle {
                math: true,
                ..TextStyle::default()
            },
            &line,
        );
        let text_w = approx_width_pt(&line, &font, font_size);
        let x = MARGIN + ((PAGE_WIDTH - 2.0 * MARGIN) - text_w) / 2.0;
        page.text()
            .set_fill_color(Color::rgb(0.12, 0.12, 0.18))
            .set_font(font, font_size)
            .at(x, baseline)
            .write(&line)
            .map_err(|e| format!("テキスト描画に失敗しました: {e}"))?;
        baseline -= line_height;
    }

    page_cursor = y_bottom - 8.0;
    Ok(page_cursor)
}

fn classify_thematic_break_style(line: &str) -> ThematicBreakStyle {
    let trimmed = line.trim();
    let mut chars = trimmed.chars().filter(|ch| !ch.is_whitespace());
    let Some(first) = chars.next() else {
        return ThematicBreakStyle::Hyphen;
    };
    if !matches!(first, '-' | '*' | '_') {
        return ThematicBreakStyle::Hyphen;
    }
    if chars.any(|ch| ch != first) {
        return ThematicBreakStyle::Hyphen;
    }
    match first {
        '-' => ThematicBreakStyle::Hyphen,
        '*' => ThematicBreakStyle::Asterisk,
        '_' => ThematicBreakStyle::Underscore,
        _ => ThematicBreakStyle::Hyphen,
    }
}

fn blockquote_body_width(_lines: &[BlockquoteLine], _face: &PreviewFace, _font_size: f64) -> f64 {
    (PAGE_WIDTH - 2.0 * MARGIN - 12.0).max(110.0)
}

fn ensure_room(
    cursor_y: f64,
    required_height: f64,
    page: &mut OxidizePage,
    doc: &mut Document,
    layout: &mut PreviewLayoutState,
) -> f64 {
    if cursor_y - required_height > MARGIN {
        return cursor_y;
    }
    // why: ページ先頭で巨大ブロックが来た場合、空ページを挿入してから描画すると
    //      先頭に空白ページが混ざってしまう
    // alt: そのままページ分割する（空ページが発生しやすい）
    // evidence: 先頭の oversized block は同一ページへ描画した方が空白ページを避けられる
    let page_top = PAGE_HEIGHT - MARGIN;
    if (cursor_y - page_top).abs() < 0.5 && required_height > (PAGE_HEIGHT - 2.0 * MARGIN) {
        log::warn!(
            "Skipping automatic page break for oversized block on an empty page: required_height={}, available_height={}",
            required_height,
            PAGE_HEIGHT - 2.0 * MARGIN
        );
        return cursor_y;
    }
    let mut next_page = OxidizePage::new(PAGE_WIDTH, PAGE_HEIGHT);
    std::mem::swap(page, &mut next_page);
    doc.add_page(next_page);
    layout.page_index += 1;
    PAGE_HEIGHT - MARGIN
}

fn wrap_text(text: &str, max_display_width: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    let mut lines: Vec<String> = Vec::new();
    for paragraph in text.split('\n') {
        let mut current = String::new();
        let mut w = 0usize;
        for ch in paragraph.chars() {
            let cw = char_display_width(ch);
            if w + cw > max_display_width && !current.is_empty() {
                lines.push(std::mem::take(&mut current));
                w = 0;
            }
            current.push(ch);
            w += cw;
        }
        lines.push(current);
    }
    lines
}

fn char_display_width(ch: char) -> usize {
    match ch {
        '\u{3000}' => 2,
        c if ('\u{1100}'..='\u{115F}').contains(&c)
            || ('\u{2E80}'..='\u{3247}').contains(&c)
            || ('\u{3250}'..='\u{4DBF}').contains(&c)
            || ('\u{4E00}'..='\u{9FFF}').contains(&c)
            || ('\u{A960}'..='\u{A97C}').contains(&c)
            || ('\u{AC00}'..='\u{D7A3}').contains(&c)
            || ('\u{F900}'..='\u{FAFF}').contains(&c)
            || ('\u{FE10}'..='\u{FE19}').contains(&c)
            || ('\u{FE30}'..='\u{FE6B}').contains(&c)
            || ('\u{FF01}'..='\u{FF60}').contains(&c)
            || ('\u{FFE0}'..='\u{FFE6}').contains(&c) =>
        {
            2
        }
        _ => 1,
    }
}

fn unicode_display_width(s: &str) -> usize {
    s.chars().map(char_display_width).sum()
}

fn wrap_code_lines(code: &str, max_chars: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in code.trim_end_matches('\n').split('\n') {
        let mut buf = String::new();
        let mut count = 0usize;
        for ch in raw.chars() {
            if count >= max_chars && !buf.is_empty() {
                out.push(std::mem::take(&mut buf));
                count = 0;
            }
            buf.push(ch);
            count += 1;
        }
        out.push(buf);
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

fn resolve_mmdc_node_modules_bin_from(root: &Path) -> Option<PathBuf> {
    let bin = root.join("node_modules").join(".bin");

    #[cfg(windows)]
    {
        for name in ["mmdc.cmd", "mmdc.exe", "mmdc"] {
            let p = bin.join(name);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    #[cfg(not(windows))]
    {
        let p = bin.join("mmdc");
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// `npm install` 済みの開発用リポジトリでは `node_modules/.bin/mmdc` をそのまま使う。
fn resolve_mmdc_workspace_node_modules_bin() -> Option<PathBuf> {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent()?;
    resolve_mmdc_node_modules_bin_from(workspace_root)
}

fn resolve_mmdc_cwd_node_modules_bin() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    for dir in cwd.ancestors() {
        if let Some(found) = resolve_mmdc_node_modules_bin_from(dir) {
            return Some(found);
        }
    }
    None
}

fn resolve_mmdc_executable() -> PathBuf {
    if let Some(p) = std::env::var_os("MINIPDF_MERMAID_CLI") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }

    if let Some(p) = resolve_mmdc_workspace_node_modules_bin() {
        return p;
    }

    if let Some(p) = resolve_mmdc_cwd_node_modules_bin() {
        return p;
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            #[cfg(windows)]
            {
                for rel in ["mmdc.exe", r"bin\mmdc.exe", "mmdc"] {
                    let c = dir.join(rel);
                    if c.is_file() {
                        return c;
                    }
                }
            }
            #[cfg(not(windows))]
            {
                for rel in ["mmdc", "bin/mmdc"] {
                    let c = dir.join(rel);
                    if c.is_file() {
                        return c;
                    }
                }
            }
        }
    }

    PathBuf::from("mmdc")
}

fn mermaid_cli_is_available() -> bool {
    let path = resolve_mmdc_executable();
    Command::new(path).arg("--version").output().is_ok()
}

fn max_chars_for_font(font_size: f64) -> usize {
    if font_size >= 22.0 {
        45
    } else if font_size >= 18.0 {
        58
    } else {
        72
    }
}

fn render_mermaid_png(code: &str) -> Result<Option<(Vec<u8>, f64, f64)>, String> {
    let work_dir = std::env::temp_dir().join(format!("miniPDF-mermaid-{}", Uuid::new_v4()));
    fs::create_dir_all(&work_dir)
        .map_err(|e| format!("Mermaid作業ディレクトリの作成に失敗しました: {e}"))?;

    if !mermaid_cli_is_available() {
        cleanup_dir(&work_dir);
        return Ok(None);
    }

    let input_path = work_dir.join("diagram.mmd");
    let output_path = work_dir.join("diagram.png");
    fs::write(&input_path, code)
        .map_err(|e| format!("Mermaid入力ファイルの作成に失敗しました: {e}"))?;

    let mmdc = resolve_mmdc_executable();
    let run_result = Command::new(&mmdc)
        .arg("-i")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("-b")
        .arg("transparent")
        .arg("-s")
        .arg("2")
        .output();

    let output = match run_result {
        Ok(output) => output,
        Err(err) => {
            log::warn!("mmdc を起動できません ({:?}): {}", mmdc, err);
            cleanup_dir(&work_dir);
            return Ok(None);
        }
    };

    if !output.status.success() {
        log::warn!(
            "mmdc failed: status={} stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        cleanup_dir(&work_dir);
        return Ok(None);
    }

    let png_data =
        fs::read(&output_path).map_err(|e| format!("Mermaid画像の読み込みに失敗しました: {e}"))?;
    let image = image::load_from_memory(&png_data)
        .map_err(|e| format!("Mermaid画像サイズの取得に失敗しました: {e}"))?;
    let (width, height) = image.dimensions();

    cleanup_dir(&work_dir);
    Ok(Some((png_data, width as f64, height as f64)))
}

fn cleanup_dir(path: &Path) {
    if let Err(err) = fs::remove_dir_all(path) {
        log::warn!("Failed to remove temporary directory {:?}: {}", path, err);
    }
}

fn draw_image_on_page(
    page: &mut OxidizePage,
    image_data: &[u8],
    format: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    let image_obj = match format {
        "png" => Image::from_png_data(image_data.to_vec())
            .map_err(|e| format!("PNG画像の読み込みに失敗しました: {e}"))?,
        _ => {
            let encoded = general_purpose::STANDARD.encode(image_data);
            let decoded = general_purpose::STANDARD
                .decode(encoded)
                .map_err(|e| format!("画像データ処理に失敗しました: {e}"))?;
            Image::from_jpeg_data(decoded)
                .map_err(|e| format!("JPEG画像の読み込みに失敗しました: {e}"))?
        }
    };

    let image_name = format!("mermaid-{}", Uuid::new_v4());
    page.add_image(&image_name, image_obj);
    page.graphics().draw_image(&image_name, x, y, width, height);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn preview_pdf_bytes(blocks: &[MarkdownBlock]) -> Vec<u8> {
        build_preview_pdf(blocks, 512)
            .expect("preview PDF should be generated")
            .0
    }

    fn preview_layout(markdown_line_count: usize, blocks: &[MarkdownBlock]) -> (Vec<u8>, Vec<u32>) {
        build_preview_pdf(blocks, markdown_line_count.max(1))
            .expect("preview PDF should be generated")
    }

    #[test]
    fn hard_break_lines_map_to_later_preview_pages() {
        let mut lines: Vec<String> = Vec::new();
        for index in 1..=80 {
            lines.push(format!("line {index}"));
        }
        let pivot = 40usize;
        lines[pivot - 1].push_str("  ");
        let markdown = lines.join("\n");
        let line_count = markdown.lines().count();
        let blocks = parse_markdown_blocks(&markdown);
        let (_bytes, line_page_map) = preview_layout(line_count, &blocks);

        assert!(
            line_page_map.len() >= line_count,
            "map_len={}, line_count={line_count}",
            line_page_map.len()
        );
        assert!(
            line_page_map[line_count - 1] >= line_page_map[0],
            "tail_page={}, head_page={}",
            line_page_map[line_count - 1],
            line_page_map[0]
        );
        assert!(
            line_page_map[pivot] >= line_page_map[0],
            "break_line_page={}, head_page={}",
            line_page_map[pivot],
            line_page_map[0]
        );
    }

    #[test]
    fn parse_ordered_list_separate_blocks() {
        let md = "1. first\n2. second\n";
        let b = parse_markdown_blocks(md);
        assert_eq!(b.len(), 2);
        match (&b[0], &b[1]) {
            (
                MarkdownBlock::OrderedListItem {
                    n: 1,
                    indent: 0,
                    spans: a,
                    ..
                },
                MarkdownBlock::OrderedListItem {
                    n: 2,
                    indent: 0,
                    spans: b,
                    ..
                },
            ) => {
                assert_eq!(plain_text(&a), "first");
                assert_eq!(plain_text(&b), "second");
            }
            _ => panic!("unexpected blocks: {b:?}"),
        }
    }

    #[test]
    fn parse_table_rows() {
        let md = "| A | B |\n| --- | --- |\n| 1 | 2 |\n";
        let b = parse_markdown_blocks(md);
        assert_eq!(b.len(), 1);
        match &b[0] {
            // GFM の AST では区切り行は TableRow にならず、ヘッダ + データ行のみのことが多い
            MarkdownBlock::Table { rows, .. } => assert!(rows.len() >= 2, "rows={:?}", rows),
            _ => panic!("expected table"),
        }
    }

    #[test]
    fn table_cell_wrapping_preserves_all_lines() {
        let row = vec![vec![InlineSpan {
            text: "abcdef".to_string(),
            style: TextStyle::default(),
        }]];
        let wrapped = wrap_table_row_cells(&row, 3);
        assert_eq!(wrapped, vec![vec!["abc".to_string(), "def".to_string()]]);
    }

    #[test]
    fn wrap_text_counts_cjk_width() {
        let s = "あいう";
        let lines = wrap_text(s, 4);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].chars().count(), 2);
        assert_eq!(lines[1].chars().count(), 1);
    }

    #[test]
    fn bold_and_inline_code_in_paragraph() {
        let md = "Hello **world** and `code`.\n";
        let b = parse_markdown_blocks(md);
        match &b[0] {
            MarkdownBlock::Paragraph { spans, .. } => {
                assert!(
                    spans
                        .iter()
                        .any(|s| s.style.bold && s.text.contains("world")),
                    "{spans:?}"
                );
                assert!(
                    spans.iter().any(|s| s.style.code && s.text == "code"),
                    "{spans:?}"
                );
            }
            _ => panic!("expected paragraph: {b:?}"),
        }
    }

    #[test]
    fn paragraph_soft_break_becomes_newline() {
        let md = "line one\nline two\n";
        let b = parse_markdown_blocks(md);
        match &b[0] {
            MarkdownBlock::Paragraph { spans, .. } => {
                let t = plain_text(spans);
                assert!(
                    t.contains('\n'),
                    "expected single newline in paragraph to stay as line break, got {t:?}"
                );
            }
            _ => panic!("expected paragraph: {b:?}"),
        }
    }

    #[test]
    fn paragraph_hard_break_is_preserved_for_windows_newlines() {
        let md = "line one  \r\nline two\r\n";
        let b = parse_markdown_blocks(md);
        match &b[0] {
            MarkdownBlock::Paragraph { spans, .. } => {
                let t = plain_text(spans);
                assert!(
                    t.contains('\n'),
                    "expected Windows newline hard break to stay as line break, got {t:?}"
                );
                assert!(
                    t.contains("line one"),
                    "expected first line to survive, got {t:?}"
                );
                assert!(
                    t.contains("line two"),
                    "expected second line to survive, got {t:?}"
                );
            }
            _ => panic!("expected paragraph: {b:?}"),
        }
    }

    #[test]
    fn intro_cjk_soft_break_before_list_stays_single_paragraph() {
        let md = "# Title\n\nこのファイルは、Markdownレンダラーが各種Markdown構文を正し\nく描画できるか確認するためのテスト用Markdownです。\n\n確認対象:\n\n- CommonMark\n";
        let blocks = parse_markdown_blocks(md);
        let intro = blocks.iter().find_map(|block| {
            if let MarkdownBlock::Paragraph { spans, source } = block {
                let text = plain_text(spans);
                if text.contains("正し") && text.contains("く描画") {
                    return Some((spans.clone(), *source));
                }
                None
            } else {
                None
            }
        });
        let (spans, source) = intro.expect("expected one intro paragraph with a soft break");
        let text = plain_text(&spans);
        assert!(text.contains('\n'), "expected soft break newline, got {text:?}");
        assert!(source.end_line > source.start_line, "expected multi-line source range");

        let layout_lines = layout_spans_lines_with_source(&spans, 90, source);
        assert!(
            layout_lines.len() >= 2,
            "expected wrapped/soft-break layout lines, got {}",
            layout_lines.len()
        );
    }

    #[test]
    fn intro_cjk_blank_line_between_halves_splits_into_separate_paragraphs() {
        let md = "# Title\n\nこのファイルは、Markdownレンダラーが各種Markdown構文を正し\n\nく描画できるか確認するためのテスト用Markdownです。\n\n確認対象:\n\n- CommonMark\n";
        let blocks = parse_markdown_blocks(md);
        let intro_paras: Vec<_> = blocks
            .iter()
            .filter_map(|block| {
                if let MarkdownBlock::Paragraph { spans, .. } = block {
                    let text = plain_text(spans);
                    if text.contains("正し") || text.contains("く描画") {
                        Some(text)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            intro_paras.len(),
            2,
            "expected blank line to split intro into two paragraphs, got {intro_paras:?}"
        );
        assert!(
            blocks.iter().any(|block| matches!(block, MarkdownBlock::Spacer { .. })),
            "expected spacer between split paragraphs"
        );
    }

    #[tokio::test]
    async fn intro_cjk_soft_break_before_list_renders_without_large_vertical_gap() {
        let md = "# Title\n\nこのファイルは、Markdownレンダラーが各種Markdown構文を正し\nく描画できるか確認するためのテスト用Markdownです。\n\n確認対象:\n\n- CommonMark\n";
        let blocks = parse_markdown_blocks(md);
        let line_count = md.lines().count();
        let bytes = preview_layout(line_count, &blocks).0;
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let pdf_path = temp_dir.path().join("intro-soft-break.pdf");
        std::fs::write(&pdf_path, bytes).expect("preview PDF should be written");
        let loaded = crate::commands::pdf_loader::load_pdf(pdf_path.to_string_lossy().to_string())
            .await
            .expect("generated preview PDF should be readable");

        let mut first_y: Option<f64> = None;
        let mut second_y: Option<f64> = None;
        for page in &loaded.pages {
            for block in &page.text_blocks {
                if block.text.contains("正し") {
                    first_y = Some(first_y.map_or(block.y, |y| y.max(block.y)));
                }
                if block.text.contains("く描画") {
                    second_y = Some(second_y.map_or(block.y, |y| y.min(block.y)));
                }
            }
        }

        let first_y = first_y.expect("expected first intro line in PDF text");
        let second_y = second_y.expect("expected second intro line in PDF text");
        let gap = (first_y - second_y).abs();
        assert!(
            gap <= BASE_LINE_HEIGHT * 2.5,
            "expected intro soft-break lines to stay adjacent, gap={gap}"
        );
    }

    #[test]
    fn paragraph_soft_breaks_survive_ascii_and_cjk_input() {
        let md = "ASCII input\nnext line\n\n半角入力\n次の行\n";
        let b = parse_markdown_blocks(md);
        assert_eq!(b.len(), 3, "{b:?}");

        match (&b[0], &b[1], &b[2]) {
            (
                MarkdownBlock::Paragraph {
                    spans: first,
                    ..
                },
                MarkdownBlock::Spacer { lines: 1, .. },
                MarkdownBlock::Paragraph {
                    spans: second,
                    ..
                },
            ) => {
                let first_text = plain_text(first);
                let second_text = plain_text(second);
                assert!(
                    first_text.contains('\n'),
                    "expected ASCII paragraph to preserve line break, got {first_text:?}"
                );
                assert!(
                    second_text.contains('\n'),
                    "expected CJK paragraph to preserve line break, got {second_text:?}"
                );
            }
            _ => panic!("expected paragraph, spacer, paragraph blocks: {b:?}"),
        }
    }

    #[test]
    fn display_math_expression_is_simplified() {
        let lines = format_math_expression(
            "\\sum_{i=1}^{n} i = \\frac{n(n+1)}{2}\n\\int_{-\\infty}^{\\infty} e^{-x^2} dx = \\sqrt{\\pi}",
        );
        let joined = lines.join("\n");
        assert!(joined.contains('∑'), "{joined}");
        assert!(joined.contains('∫'), "{joined}");
        assert!(joined.contains('∞'), "{joined}");
        assert!(joined.contains('√'), "{joined}");
        assert!(!joined.contains("\\frac"), "{joined}");
        assert!(!joined.contains("\\sum"), "{joined}");
    }

    #[test]
    fn vector_norm_notation_is_preserved_in_fallback_math() {
        let lines = format_math_expression("\\|\\mathbf{x}\\|_2 = \\sqrt{x_1^2 + x_2^2 + x_3^2}");
        let joined = lines.join("\n");
        assert!(joined.contains('‖'), "{joined}");
        assert!(joined.contains('₂'), "{joined}");
        assert!(joined.contains('√'), "{joined}");
        assert!(!joined.contains("\\|"), "{joined}");
        assert!(!joined.contains("\\mathbf"), "{joined}");
    }

    #[test]
    fn norm_delimiters_are_normalized_for_katex_rendering() {
        let normalized =
            normalize_math_for_katex("\\left\\|\\mathbf{x}\\right\\|_2 + \\lVert y \\rVert");
        assert!(normalized.contains('‖'), "{normalized}");
        assert!(!normalized.contains("\\left"), "{normalized}");
        assert!(!normalized.contains("\\right"), "{normalized}");
        assert!(!normalized.contains("\\|"), "{normalized}");
        assert!(!normalized.contains("\\lVert"), "{normalized}");
        assert!(!normalized.contains("\\rVert"), "{normalized}");
    }

    #[test]
    fn matrix_math_environment_is_compacted() {
        let lines = format_math_expression(
            "A =\n\\begin{bmatrix}\n1 & 2 & 3 \\\\\n4 & 5 & 6 \\\\\n7 & 8 & 9\n\\end{bmatrix}",
        );
        let joined = lines.join("\n");
        assert!(joined.contains("A ="), "{joined}");
        assert!(joined.contains('['), "{joined}");
        assert!(joined.contains(']'), "{joined}");
        assert!(!joined.contains("\\begin"), "{joined}");
        assert!(!joined.contains("\\end"), "{joined}");
    }

    #[test]
    fn katex_renderer_renders_inline_math_to_png() {
        let rendered = render_math_with_katex("E = mc^2", false)
            .expect("katex renderer should execute")
            .expect("katex renderer should return an image");
        assert!(!rendered.jpeg_data.is_empty(), "expected JPEG data");
        assert!(rendered.width_pt > 0.0, "expected positive width");
        assert!(rendered.height_pt > 0.0, "expected positive height");
        assert!(rendered.baseline_pt > 0.0, "expected positive baseline");
        assert!(
            rendered.baseline_pt < rendered.height_pt,
            "expected baseline within image height"
        );
    }

    #[test]
    fn katex_renderer_renders_display_math_to_png() {
        let rendered = render_math_with_katex("\\sum_{i=1}^{n} i = \\frac{n(n+1)}{2}", true)
            .expect("katex renderer should execute")
            .expect("katex renderer should return an image");
        assert!(!rendered.jpeg_data.is_empty(), "expected JPEG data");
        assert!(rendered.width_pt > 0.0, "expected positive width");
        assert!(rendered.height_pt > 0.0, "expected positive height");
    }

    #[test]
    fn scale_math_image_never_expands_rendered_math() {
        let rendered = MathRenderImage {
            jpeg_data: vec![1, 2, 3],
            width_pt: 120.0,
            height_pt: 48.0,
            baseline_pt: 34.0,
        };
        let (w, h) = scale_math_image(&rendered, 24.0);
        assert!(w < rendered.width_pt, "{w}");
        assert!(h < rendered.height_pt, "{h}");

        let (w2, h2) = scale_math_image(&rendered, 96.0);
        assert_eq!(w2, rendered.width_pt, "{w2}");
        assert_eq!(h2, rendered.height_pt, "{h2}");
    }

    #[test]
    fn transparent_border_is_trimmed_from_math_renders() {
        let mut image = image::RgbaImage::from_pixel(6, 6, image::Rgba([0, 0, 0, 0]));
        image.put_pixel(2, 1, image::Rgba([20, 30, 40, 255]));
        image.put_pixel(3, 1, image::Rgba([20, 30, 40, 255]));
        image.put_pixel(2, 2, image::Rgba([20, 30, 40, 255]));
        image.put_pixel(3, 2, image::Rgba([20, 30, 40, 255]));

        let (trimmed, width, height) = trim_transparent_border(&image);
        assert_eq!(width, 2);
        assert_eq!(height, 2);
        assert_eq!(trimmed.width(), 2);
        assert_eq!(trimmed.height(), 2);
        assert_eq!(trimmed.get_pixel(0, 0).0[3], 255);
    }

    #[tokio::test]
    async fn preview_consistency_fixture_renders_repeatedly_without_drift() {
        let md = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../fixtures/preview-consistency-test.md"
        ));
        let blocks = parse_markdown_blocks(md);
        assert!(
            !blocks.is_empty(),
            "expected fixture to produce markdown blocks"
        );

        let mut first_page_count = None;
        for _ in 0..3 {
            let bytes = preview_pdf_bytes(&blocks);
            let temp_dir = tempfile::tempdir().expect("temp dir should be created");
            let pdf_path = temp_dir.path().join("preview-consistency.pdf");
            std::fs::write(&pdf_path, bytes).expect("preview PDF should be written");
            let loaded =
                crate::commands::pdf_loader::load_pdf(pdf_path.to_string_lossy().to_string())
                    .await
                    .expect("generated preview PDF should be readable");

            assert!(!loaded.pages.is_empty(), "expected at least one page");
            if let Some(previous) = first_page_count {
                assert_eq!(
                    previous,
                    loaded.pages.len(),
                    "expected stable page count across runs"
                );
            } else {
                first_page_count = Some(loaded.pages.len());
            }
        }
    }

    #[tokio::test]
    async fn markdown_renderer_visual_check_fixture_is_stable_and_safe() {
        let md = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../fixtures/markdown-renderer-visual-check.md"
        ));
        let blocks = parse_markdown_blocks(md);
        assert!(
            !blocks.is_empty(),
            "expected visual check fixture to produce blocks"
        );

        let mut page_counts = Vec::new();
        for _ in 0..2 {
            let bytes = preview_pdf_bytes(&blocks);
            let temp_dir = tempfile::tempdir().expect("temp dir should be created");
            let pdf_path = temp_dir.path().join("markdown-renderer-visual-check.pdf");
            std::fs::write(&pdf_path, bytes).expect("preview PDF should be written");
            let loaded =
                crate::commands::pdf_loader::load_pdf(pdf_path.to_string_lossy().to_string())
                    .await
                    .expect("generated preview PDF should be readable");

            page_counts.push(loaded.pages.len());
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
            assert!(all_text.contains("H1 見出し"), "{all_text}");
            assert!(all_text.contains("item 1"), "{all_text}");
            assert!(all_text.contains("引用内コードブロック"), "{all_text}");
            assert!(
                blocks.iter().any(|block| matches!(
                    block,
                    MarkdownBlock::Heading { level: 2, spans, .. }
                        if plain_text(spans).contains("Unicode / 日本語 / 絵文字")
                )),
                "expected the post-Mermaid heading to remain parseable: {blocks:?}"
            );
            assert!(
                blocks.iter().any(|block| matches!(
                    block,
                    MarkdownBlock::Heading { level: 2, spans, .. }
                        if plain_text(spans).contains("終端確認")
                )),
                "expected the visual fixture tail to remain parseable: {blocks:?}"
            );
            assert!(all_text.contains("XSS script tag"), "{all_text}");
            assert!(
                loaded.pages.iter().any(|page| !page.images.is_empty()),
                "expected the visual fixture to contain rendered images for math or mermaid"
            );
        }

        assert_eq!(page_counts[0], page_counts[1], "expected stable page count");
    }

    #[tokio::test]
    async fn extra_blank_lines_are_clamped_in_preview_layout() {
        let mut compact_lines = vec!["# Title".to_string()];
        for i in 1..=15 {
            compact_lines.push(format!(
                "Paragraph {i}. This sentence is intentionally a little longer so the compact version stays close to a page boundary without wrapping into the next page too early."
            ));
            compact_lines.push(String::new());
        }
        let compact = compact_lines.join("\n");

        let mut spaced_lines = vec!["# Title".to_string()];
        for i in 1..=15 {
            spaced_lines.push(format!(
                "Paragraph {i}. This sentence is intentionally a little longer so the compact version stays close to a page boundary without wrapping into the next page too early."
            ));
            if i == 8 {
                spaced_lines.extend(std::iter::repeat(String::new()).take(128));
            }
            spaced_lines.push(String::new());
        }
        let spaced = spaced_lines.join("\n");

        let compact_blocks = parse_markdown_blocks(&compact);
        let spaced_blocks = parse_markdown_blocks(&spaced);

        let compact_bytes =
            preview_pdf_bytes(&compact_blocks);
        let spaced_bytes = preview_pdf_bytes(&spaced_blocks);

        let temp_dir = tempdir().expect("temp dir should be created");
        let compact_path = temp_dir.path().join("compact.pdf");
        let spaced_path = temp_dir.path().join("spaced.pdf");
        std::fs::write(&compact_path, compact_bytes).expect("compact PDF should be written");
        std::fs::write(&spaced_path, spaced_bytes).expect("spaced PDF should be written");

        let compact_loaded =
            crate::commands::pdf_loader::load_pdf(compact_path.to_string_lossy().to_string())
                .await
                .expect("compact PDF should be readable");
        let spaced_loaded =
            crate::commands::pdf_loader::load_pdf(spaced_path.to_string_lossy().to_string())
                .await
                .expect("spaced PDF should be readable");

        assert!(
            spaced_loaded.pages.len() >= compact_loaded.pages.len(),
            "expected extra blank lines to push content toward later pages, compact_pages={}, spaced_pages={}",
            compact_loaded.pages.len(),
            spaced_loaded.pages.len()
        );
        let spaced_text = spaced_loaded
            .pages
            .iter()
            .flat_map(|page| page.text_blocks.iter().map(|block| block.text.as_str()))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(spaced_text.contains("Paragraph 15."), "{spaced_text}");
    }

    #[test]
    fn nested_bullet_list_is_flattened_with_indent() {
        let md = "- a\n- b\n  - nested\n";
        let b = parse_markdown_blocks(md);
        assert_eq!(b.len(), 3, "{b:?}");
        match (&b[0], &b[1], &b[2]) {
            (
                MarkdownBlock::ListItem {
                    indent: 0,
                    spans: a,
                    ..
                },
                MarkdownBlock::ListItem {
                    indent: 0,
                    spans: b,
                    ..
                },
                MarkdownBlock::ListItem {
                    indent: 1,
                    spans: n,
                    ..
                },
            ) => {
                assert!(plain_text(a).contains('a'));
                assert!(plain_text(b).contains('b'));
                assert!(plain_text(n).contains("nested"));
            }
            _ => panic!("unexpected blocks: {b:?}"),
        }
    }

    #[test]
    fn thematic_break_emits_block() {
        let md = "before\n\n---\n\nafter\n";
        let b = parse_markdown_blocks(md);
        assert!(
            b.iter()
                .any(|x| matches!(x, MarkdownBlock::ThematicBreak { .. })),
            "{b:?}"
        );
    }

    #[test]
    fn blank_lines_between_top_level_blocks_emit_spacers() {
        let md = "first paragraph\n\n\nsecond paragraph\n";
        let blocks = parse_markdown_blocks(md);
        assert!(
            blocks
                .iter()
                .any(|block| matches!(block, MarkdownBlock::Spacer { lines, .. } if *lines >= 1)),
            "{blocks:?}"
        );
    }

    #[test]
    fn blank_lines_after_thematic_break_before_heading_emit_spacer() {
        let md = "---\n  \n \n\n## 1. 見出し\n";
        let blocks = parse_markdown_blocks(md);
        let spacer = blocks.iter().find_map(|block| match block {
            MarkdownBlock::Spacer { lines, source } => Some((*lines, *source)),
            _ => None,
        });
        assert!(
            spacer.map(|(lines, _)| lines >= 3).unwrap_or(false),
            "expected blank lines after --- before heading, got {blocks:?}"
        );
        let order: Vec<_> = blocks
            .iter()
            .map(|block| match block {
                MarkdownBlock::ThematicBreak { .. } => "hr",
                MarkdownBlock::Spacer { .. } => "spacer",
                MarkdownBlock::Heading { .. } => "heading",
                _ => "other",
            })
            .collect();
        assert_eq!(
            order,
            vec!["hr", "spacer", "heading"],
            "{blocks:?}"
        );
    }

    #[test]
    fn blank_lines_between_list_items_emit_spacer() {
        let md = "- first item\n\n\n- second item\n";
        let blocks = parse_markdown_blocks(md);
        assert!(
            blocks.iter().any(|block| matches!(
                block,
                MarkdownBlock::Spacer { lines, .. } if *lines >= 2
            )),
            "{blocks:?}"
        );
        let order: Vec<_> = blocks
            .iter()
            .map(|block| match block {
                MarkdownBlock::ListItem { spans, .. } => plain_text(spans),
                MarkdownBlock::Spacer { .. } => "<spacer>".to_string(),
                _ => "?".to_string(),
            })
            .collect();
        assert_eq!(
            order,
            vec!["first item", "<spacer>", "second item"],
            "{blocks:?}"
        );
    }

    #[test]
    fn spacer_block_clamps_excess_blank_lines() {
        let moderate = "first paragraph\n\n\n\n\n\nsecond paragraph\n";
        let blocks = parse_markdown_blocks(moderate);
        let spacer_lines = blocks.iter().find_map(|block| match block {
            MarkdownBlock::Spacer { lines, .. } => Some(*lines),
            _ => None,
        });
        assert_eq!(spacer_lines, Some(5), "{blocks:?}");

        let mut huge = String::from("first paragraph\n");
        huge.extend(std::iter::repeat_n('\n', 60));
        huge.push_str("second paragraph\n");
        let blocks = parse_markdown_blocks(&huge);
        let spacer_lines = blocks.iter().find_map(|block| match block {
            MarkdownBlock::Spacer { lines, .. } => Some(*lines),
            _ => None,
        });
        assert_eq!(spacer_lines, Some(MAX_SPACER_LINES), "{blocks:?}");
    }

    #[tokio::test]
    async fn moderate_blank_lines_can_push_following_block_to_next_page() {
        let mut lines = vec!["# Section".to_string(), String::new()];
        lines.extend(std::iter::repeat_n(String::new(), MAX_SPACER_LINES));
        lines.push("Target paragraph that should move down.".to_string());
        let md = lines.join("\n");
        let blocks = parse_markdown_blocks(&md);
        let line_count = md.lines().count();
        let (_bytes, line_page_map) = preview_layout(line_count, &blocks);
        assert!(
            line_page_map[line_page_map.len() - 1] > line_page_map[0],
            "expected trailing content after many blank lines to map to a later page, head={}, tail={}",
            line_page_map[0],
            line_page_map[line_page_map.len() - 1]
        );
    }

    #[test]
    fn unclosed_mermaid_fence_stops_before_following_heading() {
        let md = "```mermaid\nflowchart TD\n    A -->\n\n## after\n";
        let blocks = parse_markdown_blocks(md);

        assert!(
            matches!(&blocks[0], MarkdownBlock::CodeBlock { lang, .. } if lang == "mermaid"),
            "{blocks:?}"
        );
        assert!(
            blocks.iter().any(|block| matches!(
                block,
                MarkdownBlock::Heading { level: 2, spans, .. }
                    if plain_text(spans).contains("after")
            )),
            "{blocks:?}"
        );
    }

    fn plain_text(spans: &[InlineSpan]) -> String {
        spans.iter().map(|s| s.text.as_str()).collect()
    }

    fn blockquote_plain_text(lines: &[BlockquoteLine]) -> String {
        let mut out = String::new();
        for (idx, line) in lines.iter().enumerate() {
            if idx > 0 {
                out.push('\n');
            }
            match line {
                BlockquoteLine::Text(spans) => out.push_str(&plain_text(spans)),
                BlockquoteLine::DisplayMath(expr) => out.push_str(expr),
                BlockquoteLine::CodeBlock { code } => out.push_str(code),
            }
        }
        out
    }

    #[test]
    fn links_are_marked_and_italic_japanese_requests_fake_slant() {
        let md = "これは [OpenAI](https://openai.com) と *日本語* の確認です。\n";
        let blocks = parse_markdown_blocks(md);
        assert_eq!(blocks.len(), 1, "{blocks:?}");
        let face = PreviewFace {
            body: Font::Helvetica,
            body_bold: Font::HelveticaBold,
            body_italic: Font::HelveticaOblique,
            body_bold_italic: Font::HelveticaBoldOblique,
            body_path: None,
            body_bold_path: None,
            emoji: None,
            emoji_path: None,
            has_custom_bold_italic: false,
            has_custom_bold: false,
        };

        match &blocks[0] {
            MarkdownBlock::Paragraph { spans, .. } => {
                assert!(spans.iter().any(|s| s.style.link), "{spans:?}");
                assert!(
                    spans
                        .iter()
                        .any(|s| should_fake_italic(&face, s.style, &s.text)),
                    "{spans:?}"
                );
            }
            other => panic!("unexpected block: {other:?}"),
        }
    }

    #[test]
    fn inline_math_spans_are_detected() {
        let md = "インライン数式 $E = mc^2$ を確認します。\n";
        let blocks = parse_markdown_blocks(md);
        assert_eq!(blocks.len(), 1, "{blocks:?}");

        match &blocks[0] {
            MarkdownBlock::Paragraph { spans, .. } => {
                assert!(spans.iter().any(|s| s.style.math), "{spans:?}");
                let text = plain_text(spans);
                assert!(text.contains("E = mc^2"), "{text}");
                assert!(!text.contains('$'), "{text}");
            }
            other => panic!("unexpected block: {other:?}"),
        }
    }

    #[test]
    fn emoji_detection_matches_common_symbols() {
        assert!(
            text_has_emoji("😀"),
            "expected grinning face to be detected"
        );
        assert!(
            text_has_emoji("日本語 🚀"),
            "expected mixed text with emoji to be detected"
        );
        assert!(
            grapheme_has_emoji("⚠️"),
            "expected warning sign with variation selector"
        );
        assert!(
            grapheme_has_emoji("🧪"),
            "expected test tube emoji to be detected"
        );
        assert!(
            !text_has_emoji("日本語"),
            "expected plain Japanese text to stay non-emoji"
        );
    }

    #[test]
    fn emoji_vertical_adjust_is_zero() {
        assert!(
            EMOJI_VERTICAL_ADJUST_PT.abs() < f64::EPSILON,
            "emoji vertical adjust should be zero"
        );
    }

    #[test]
    fn emoji_baseline_offset_scales_with_raster_placement() {
        let offset = emoji_baseline_offset_pt(20, 60);
        let expected = ((60 - 20) as f64 / EMOJI_RASTER_PX_PER_PT as f64) * EMOJI_DISPLAY_SCALE;
        assert!(
            (offset - expected).abs() < f64::EPSILON,
            "offset={offset}, expected={expected}"
        );
    }

    #[test]
    fn emoji_draw_y_uses_the_rasterized_baseline_offset() {
        let draw_y = emoji_draw_y(100.0, 12.5);
        assert!((draw_y - 87.5).abs() < f64::EPSILON, "draw_y={draw_y}");
    }

    #[test]
    fn faux_italic_skew_is_visible() {
        assert!(
            faux_italic_skew(1, 11.5) >= 1.0,
            "skew should be visually noticeable"
        );
    }

    #[test]
    fn faux_italic_advance_width_accounts_for_slant() {
        let font = Font::Helvetica;
        let base = approx_width_pt("太字斜体", &font, 11.5);
        let shifted = faux_italic_advance_width("太字斜体", &font, 11.5, true);
        assert!(
            shifted > base,
            "shifted={shifted}, base={base} should include the faux italic slant"
        );
    }

    #[test]
    fn japanese_italic_uses_body_font_for_faux_slant() {
        let face = PreviewFace {
            body: Font::Helvetica,
            body_bold: Font::HelveticaBold,
            body_italic: Font::custom("NotoSansJP-Italic"),
            body_bold_italic: Font::custom("NotoSansJP-BoldItalic"),
            body_path: Some(PathBuf::from("regular.ttf")),
            body_bold_path: Some(PathBuf::from("bold.ttf")),
            emoji: None,
            emoji_path: None,
            has_custom_bold_italic: true,
            has_custom_bold: true,
        };

        let italic_font = face.pick(
            TextStyle {
                italic: true,
                ..TextStyle::default()
            },
            "日本語",
        );
        let bold_italic_font = face.pick(
            TextStyle {
                italic: true,
                bold: true,
                ..TextStyle::default()
            },
            "日本語",
        );

        assert_eq!(italic_font, Font::Helvetica);
        assert_eq!(bold_italic_font, Font::HelveticaBold);
    }

    #[test]
    fn japanese_inline_emphasis_sample_marks_all_three_styles() {
        let md = "通常文の中に **太字**、*斜体*、***太字斜体*** を含めます。\n";
        let blocks = parse_markdown_blocks(md);
        assert_eq!(blocks.len(), 1, "{blocks:?}");

        match &blocks[0] {
            MarkdownBlock::Paragraph { spans, .. } => {
                assert!(
                    spans
                        .iter()
                        .any(|s| s.text == "太字" && s.style.bold && !s.style.italic),
                    "{spans:?}"
                );
                assert!(
                    spans
                        .iter()
                        .any(|s| s.text == "斜体" && !s.style.bold && s.style.italic),
                    "{spans:?}"
                );
                assert!(
                    spans
                        .iter()
                        .any(|s| s.text == "太字斜体" && s.style.bold && s.style.italic),
                    "{spans:?}"
                );
                assert!(plain_text(spans).contains("を含めます。"), "{spans:?}");
            }
            other => panic!("unexpected block: {other:?}"),
        }
    }

    #[test]
    fn japanese_non_emoji_italic_requests_full_span_raster_slant() {
        let face = PreviewFace {
            body: Font::Helvetica,
            body_bold: Font::HelveticaBold,
            body_italic: Font::custom("NotoSansJP-Italic"),
            body_bold_italic: Font::custom("NotoSansJP-BoldItalic"),
            body_path: Some(PathBuf::from("regular.ttf")),
            body_bold_path: Some(PathBuf::from("bold.ttf")),
            emoji: None,
            emoji_path: None,
            has_custom_bold_italic: true,
            has_custom_bold: true,
        };
        let italic = TextStyle {
            italic: true,
            ..TextStyle::default()
        };
        let bold_italic = TextStyle {
            bold: true,
            italic: true,
            ..TextStyle::default()
        };

        assert!(should_raster_faux_italic(&face, italic, "斜体"));
        assert!(should_raster_faux_italic(&face, bold_italic, "太字斜体"));
        assert!(!should_raster_faux_italic(&face, italic, "ABC"));
        assert!(!should_raster_faux_italic(&face, italic, "斜体😀"));
    }

    #[test]
    fn raster_faux_italic_advance_uses_drawn_image_width() {
        let rendered = RasterizedFauxItalicText {
            jpeg_data: Vec::new(),
            width_pt: 42.5,
            height_pt: 12.0,
            baseline_offset_pt: 9.0,
        };

        assert_eq!(raster_faux_italic_advance_width(&rendered), 42.5);
    }

    #[test]
    fn raster_faux_italic_baseline_offset_is_measured_from_bottom() {
        let offset = raster_faux_italic_baseline_offset(12.0, 8.0);

        assert_eq!(offset, 4.0);
    }

    #[test]
    fn mixed_emoji_text_still_requests_fake_italic_for_japanese_graphemes() {
        let style = TextStyle {
            italic: true,
            ..TextStyle::default()
        };

        assert!(
            should_fake_italic_grapheme(
                &PreviewFace {
                    body: Font::Helvetica,
                    body_bold: Font::HelveticaBold,
                    body_italic: Font::HelveticaOblique,
                    body_bold_italic: Font::HelveticaBoldOblique,
                    body_path: None,
                    body_bold_path: None,
                    emoji: None,
                    emoji_path: None,
                    has_custom_bold_italic: false,
                    has_custom_bold: false,
                },
                style,
                "日本語"
            ),
            "Japanese graphemes in a mixed emoji span should still be slanted"
        );
        assert!(
            !should_fake_italic_grapheme(
                &PreviewFace {
                    body: Font::Helvetica,
                    body_bold: Font::HelveticaBold,
                    body_italic: Font::HelveticaOblique,
                    body_bold_italic: Font::HelveticaBoldOblique,
                    body_path: None,
                    body_bold_path: None,
                    emoji: None,
                    emoji_path: None,
                    has_custom_bold_italic: false,
                    has_custom_bold: false,
                },
                style,
                "😀"
            ),
            "emoji graphemes should be rasterized instead of slanted"
        );
        assert!(
            !should_fake_italic_grapheme(
                &PreviewFace {
                    body: Font::Helvetica,
                    body_bold: Font::HelveticaBold,
                    body_italic: Font::HelveticaOblique,
                    body_bold_italic: Font::HelveticaBoldOblique,
                    body_path: None,
                    body_bold_path: None,
                    emoji: None,
                    emoji_path: None,
                    has_custom_bold_italic: false,
                    has_custom_bold: false,
                },
                style,
                "ABC"
            ),
            "ASCII text should keep the normal italic font path"
        );
    }

    #[test]
    fn math_lines_request_more_vertical_room_than_plain_text_lines() {
        let plain = vec![InlineSpan {
            text: "plain text".to_string(),
            style: TextStyle::default(),
        }];
        let math = vec![InlineSpan {
            text: "x = y + z".to_string(),
            style: TextStyle {
                math: true,
                ..TextStyle::default()
            },
        }];

        let plain_height = estimate_rich_line_height(&plain, 11.5);
        let math_height = estimate_rich_line_height(&math, 11.5);
        assert!(
            math_height > plain_height,
            "plain_height={plain_height}, math_height={math_height}"
        );
    }

    #[test]
    fn norm_math_lines_request_tall_enough_room() {
        let norm = vec![InlineSpan {
            text: "\\|\\mathbf{x}\\|_2".to_string(),
            style: TextStyle {
                math: true,
                ..TextStyle::default()
            },
        }];
        let height = estimate_rich_line_height(&norm, 11.5);
        assert!(
            height >= 20.0,
            "expected norm notation to reserve enough room, got {height}"
        );
    }

    #[test]
    fn parenthesized_math_lines_are_normalized_to_display_math() {
        let md = "これは `\\(...\\)` 形式です。\n\n\\( E = mc^2 \\)\n";
        let blocks = parse_markdown_blocks(md);
        assert_eq!(blocks.len(), 3, "{blocks:?}");

        match &blocks[0] {
            MarkdownBlock::Paragraph { spans, .. } => {
                let text = plain_text(spans);
                assert!(text.contains("これは"), "{text}");
                assert!(spans.iter().any(|s| s.style.code), "{spans:?}");
            }
            other => panic!("unexpected block: {other:?}"),
        }

        assert!(
            matches!(&blocks[1], MarkdownBlock::Spacer { lines: 1, .. }),
            "{blocks:?}"
        );

        match &blocks[2] {
            MarkdownBlock::DisplayMath { expr, .. } => {
                assert!(expr.contains("E = mc^2"), "{expr}");
            }
            other => panic!("unexpected block: {other:?}"),
        }
    }

    #[tokio::test]
    async fn display_math_blocks_render_without_dollars() {
        let blocks = parse_markdown_blocks("$$\nE = mc^2\n$$\n");
        assert_eq!(blocks.len(), 1, "{blocks:?}");
        assert!(
            matches!(&blocks[0], MarkdownBlock::DisplayMath { .. }),
            "{blocks:?}"
        );

        let bytes = preview_pdf_bytes(&blocks);
        let pdf_text = String::from_utf8_lossy(&bytes);
        assert!(
            pdf_text.contains("/DCTDecode"),
            "expected JPEG-encoded math image in PDF"
        );
        assert!(
            !pdf_text.contains("/SMask"),
            "expected no soft mask for math image"
        );
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let pdf_path = temp_dir.path().join("display-math.pdf");
        std::fs::write(&pdf_path, bytes).expect("preview PDF should be written");
        let loaded = crate::commands::pdf_loader::load_pdf(pdf_path.to_string_lossy().to_string())
            .await
            .expect("generated preview PDF should be readable");
        assert!(
            loaded.pages.iter().any(|page| !page.images.is_empty()),
            "expected rendered math to appear as an embedded image"
        );
    }

    #[tokio::test]
    async fn bracket_display_math_blocks_render_without_backslashes() {
        let blocks = parse_markdown_blocks("\\[\nE = mc^2\n\\]\n");
        assert_eq!(blocks.len(), 1, "{blocks:?}");
        assert!(
            matches!(&blocks[0], MarkdownBlock::DisplayMath { expr, .. } if expr.contains("E = mc^2")),
            "{blocks:?}"
        );

        let bytes = preview_pdf_bytes(&blocks);
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let pdf_path = temp_dir.path().join("bracket-display-math.pdf");
        std::fs::write(&pdf_path, bytes).expect("preview PDF should be written");
        let loaded = crate::commands::pdf_loader::load_pdf(pdf_path.to_string_lossy().to_string())
            .await
            .expect("generated preview PDF should be readable");
        let loaded_text = loaded
            .pages
            .iter()
            .flat_map(|page| page.text_blocks.iter().map(|block| block.text.as_str()))
            .collect::<String>();
        assert!(
            !loaded_text.contains("\\["),
            "expected display math delimiters to be stripped from extracted page text"
        );
        assert!(
            loaded.pages.iter().any(|page| !page.images.is_empty()),
            "expected rendered bracket math to appear as an embedded image"
        );
    }

    #[tokio::test]
    async fn ordered_list_prefixes_are_emitted_in_pdf() {
        let blocks = parse_markdown_blocks("1. first\n2. second\n");
        assert_eq!(blocks.len(), 2, "{blocks:?}");

        let bytes = preview_pdf_bytes(&blocks);
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let pdf_path = temp_dir.path().join("ordered-list.pdf");
        std::fs::write(&pdf_path, bytes).expect("preview PDF should be written");
        let loaded = crate::commands::pdf_loader::load_pdf(pdf_path.to_string_lossy().to_string())
            .await
            .expect("generated preview PDF should be readable");
        let all_text = loaded
            .pages
            .iter()
            .flat_map(|page| page.text_blocks.iter().map(|block| block.text.as_str()))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(all_text.contains("1."), "{all_text}");
        assert!(all_text.contains("2."), "{all_text}");
        assert!(all_text.contains("first"), "{all_text}");
        assert!(all_text.contains("second"), "{all_text}");
    }

    #[test]
    fn wrap_code_lines_preserves_internal_blank_lines() {
        let wrapped = wrap_code_lines("alpha\n\nbeta\n", 80);
        assert_eq!(
            wrapped,
            vec!["alpha".to_string(), "".to_string(), "beta".to_string()]
        );
    }

    #[test]
    fn blockquote_body_width_uses_available_width() {
        let face = PreviewFace {
            body: Font::Helvetica,
            body_bold: Font::HelveticaBold,
            body_italic: Font::HelveticaOblique,
            body_bold_italic: Font::HelveticaBoldOblique,
            body_path: None,
            body_bold_path: None,
            emoji: None,
            emoji_path: None,
            has_custom_bold_italic: false,
            has_custom_bold: false,
        };
        let short = vec![BlockquoteLine::Text(vec![InlineSpan {
            text: "short quote".to_string(),
            style: TextStyle::default(),
        }])];

        let short_width = blockquote_body_width(&short, &face, 11.0);

        assert!(
            (short_width - (PAGE_WIDTH - 2.0 * MARGIN - 12.0)).abs() < 0.001,
            "short={short_width}"
        );
        assert!(short_width > 0.0, "short={short_width}");
    }

    #[test]
    fn blockquote_long_text_wraps_into_multiple_lines() {
        let md = format!("> {}\n", "this is a long blockquote line that should wrap because it would otherwise overflow the frame by a large margin");
        let blocks = parse_markdown_blocks(&md);
        assert_eq!(blocks.len(), 1, "{blocks:?}");

        match &blocks[0] {
            MarkdownBlock::Blockquote { lines, .. } => {
                assert!(
                    lines.len() > 1,
                    "expected long blockquote text to wrap into multiple lines: {lines:?}"
                );
            }
            other => panic!("unexpected block: {other:?}"),
        }
    }

    #[test]
    fn blockquote_paragraphs_collapse_into_one_block() {
        let md = "> one\n>\n> two\n> > nested\n";
        let blocks = parse_markdown_blocks(md);
        assert_eq!(blocks.len(), 1, "{blocks:?}");

        match &blocks[0] {
            MarkdownBlock::Blockquote { lines, .. } => {
                let text = blockquote_plain_text(lines);
                assert!(text.contains("one"), "{text}");
                assert!(text.contains("two"), "{text}");
                assert!(text.contains('>'), "{text}");
            }
            other => panic!("unexpected block: {other:?}"),
        }
    }

    #[test]
    fn nested_blockquote_levels_insert_line_breaks() {
        let md = "> Level 1\n>\n> > Level 2\n> > > Level 3\n";
        let blocks = parse_markdown_blocks(md);
        assert_eq!(blocks.len(), 1, "{blocks:?}");

        match &blocks[0] {
            MarkdownBlock::Blockquote { lines, .. } => {
                let text = blockquote_plain_text(lines);
                assert!(text.contains("Level 1"), "{text}");
                assert!(text.contains("Level 2"), "{text}");
                assert!(text.contains("Level 3"), "{text}");
                assert!(text.contains('\n'), "{text:?}");
            }
            other => panic!("unexpected block: {other:?}"),
        }
    }

    #[tokio::test]
    async fn long_blockquote_spans_multiple_pages_in_pdf_and_html() {
        let md = (1..=120)
            .map(|i| format!("> quoted line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let blocks = parse_markdown_blocks(&md);
        assert_eq!(blocks.len(), 1, "{blocks:?}");
        assert!(
            matches!(&blocks[0], MarkdownBlock::Blockquote { .. }),
            "{blocks:?}"
        );

        let bytes = preview_pdf_bytes(&blocks);
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let pdf_path = temp_dir.path().join("long-blockquote.pdf");
        std::fs::write(&pdf_path, bytes).expect("preview PDF should be written");
        let loaded = crate::commands::pdf_loader::load_pdf(pdf_path.to_string_lossy().to_string())
            .await
            .expect("generated preview PDF should be readable");

        assert!(
            loaded.pages.len() > 1,
            "expected long blockquote to span multiple pages"
        );
        let first_page_text = loaded.pages[0]
            .text_blocks
            .iter()
            .map(|block| block.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let last_page_text = loaded
            .pages
            .last()
            .unwrap()
            .text_blocks
            .iter()
            .map(|block| block.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            first_page_text.contains("quoted line 1"),
            "{first_page_text}"
        );
        assert!(
            last_page_text.contains("quoted line 120"),
            "{last_page_text}"
        );

        let html = build_preview_html(&blocks);
        assert!(
            html.matches("<blockquote>").count() > 1,
            "expected blockquote chunks to be split across HTML pages: {html}"
        );
        assert!(
            html.matches("class=\"page\"").count() > 1,
            "expected HTML preview to span multiple pages for long blockquote: {html}"
        );
        assert!(
            html.contains("overflow-wrap: anywhere"),
            "expected blockquote CSS to keep long text inside the frame: {html}"
        );
    }

    #[tokio::test]
    async fn blockquote_code_block_is_preserved() {
        let md = "> before\n>\n> ```js\n> console.log('inside quote');\n> ```\n> after\n";
        let blocks = parse_markdown_blocks(md);
        assert_eq!(blocks.len(), 1, "{blocks:?}");
        match &blocks[0] {
            MarkdownBlock::Blockquote { lines, .. } => {
                assert!(
                    lines.iter().any(|line| matches!(
                        line,
                        BlockquoteLine::CodeBlock { code } if code.contains("inside quote")
                    )),
                    "{lines:?}"
                );
            }
            other => panic!("unexpected block: {other:?}"),
        }

        let bytes = preview_pdf_bytes(&blocks);
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let pdf_path = temp_dir.path().join("blockquote-code-block.pdf");
        std::fs::write(&pdf_path, bytes).expect("preview PDF should be written");
        let loaded = crate::commands::pdf_loader::load_pdf(pdf_path.to_string_lossy().to_string())
            .await
            .expect("generated preview PDF should be readable");
        let text = loaded
            .pages
            .iter()
            .flat_map(|page| page.text_blocks.iter().map(|block| block.text.as_str()))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("inside quote"), "{text}");
        assert!(text.contains("before"), "{text}");
        assert!(text.contains("after"), "{text}");
    }

    #[tokio::test]
    async fn emoji_list_items_survive_pdf_and_html_rendering() {
        let md = [
            "## 絵文字",
            "",
            "- 😀",
            "- 🚀",
            "- ✅",
            "- ⚠️",
            "- 🧪",
            "",
            "日本語 English 123 🚀",
        ]
        .join("\n");
        let blocks = parse_markdown_blocks(&md);
        assert!(!blocks.is_empty(), "{blocks:?}");

        let bytes = preview_pdf_bytes(&blocks);
        let pdf_path = std::env::temp_dir().join("minipdf-emoji-preview.pdf");
        std::fs::write(&pdf_path, bytes).expect("preview PDF should be written");
        let loaded = crate::commands::pdf_loader::load_pdf(pdf_path.to_string_lossy().to_string())
            .await
            .expect("generated emoji preview PDF should be readable");

        let has_images = loaded.pages.iter().any(|page| !page.images.is_empty());
        let extracted = loaded
            .pages
            .iter()
            .flat_map(|page| page.text_blocks.iter().map(|block| block.text.as_str()))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            has_images
                || extracted.contains('😀')
                || extracted.contains('🚀')
                || extracted.contains('✅')
                || extracted.contains('⚠')
                || extracted.contains('🧪'),
            "expected emoji preview PDF to keep emoji visible in either image or text form"
        );

        assert!(extracted.contains("絵文字"), "{extracted}");

        let html = build_preview_html(&blocks);
        assert!(html.contains("😀"), "{html}");
        assert!(html.contains("🚀"), "{html}");
        assert!(html.contains("✅"), "{html}");
    }

    #[test]
    fn code_block_prefers_single_page_when_it_fits() {
        assert!(
            code_block_fits_on_single_page(20.0 * 11.0 + 18.0),
            "expected short code block to fit on one page"
        );
        assert!(
            !code_block_fits_on_single_page(PAGE_HEIGHT),
            "expected oversized code block to fall back to chunking"
        );
    }

    #[test]
    fn thematic_break_variants_are_all_parsed_as_breaks() {
        let blocks = parse_markdown_blocks("---\n***\n___\n");
        assert_eq!(blocks.len(), 3, "{blocks:?}");
        assert!(
            blocks
                .iter()
                .all(|block| matches!(block, MarkdownBlock::ThematicBreak { .. })),
            "{blocks:?}"
        );

        let html = build_preview_html(&blocks);
        assert_eq!(html.matches("class=\"hr-hyphen\"").count(), 1, "{html}");
        assert_eq!(html.matches("class=\"hr-asterisk\"").count(), 1, "{html}");
        assert_eq!(html.matches("class=\"hr-underscore\"").count(), 1, "{html}");
    }

    #[test]
    fn blockquote_display_math_is_preserved_as_a_separate_line() {
        let md = "> intro\n>\n> $$\n> x = y + z\n> $$\n>\n> outro\n";
        let blocks = parse_markdown_blocks(md);
        assert_eq!(blocks.len(), 1, "{blocks:?}");

        match &blocks[0] {
            MarkdownBlock::Blockquote { lines, .. } => {
                assert!(
                    lines.iter().any(|line| matches!(line, BlockquoteLine::DisplayMath(expr) if expr.contains("x = y + z"))),
                    "{lines:?}"
                );
                let text = blockquote_plain_text(lines);
                assert!(text.contains("intro"), "{text}");
                assert!(text.contains("outro"), "{text}");
            }
            other => panic!("unexpected block: {other:?}"),
        }
    }

    #[test]
    fn resolve_mmdc_in_ancestor_cwd_node_modules_bin() {
        let temp_root = std::env::temp_dir().join(format!("minipdf-mmdc-test-{}", Uuid::new_v4()));
        let nested = temp_root.join("nested").join("child");
        let bin = temp_root.join("node_modules").join(".bin");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::create_dir_all(&nested).unwrap();

        #[cfg(windows)]
        let found_path = {
            let target = bin.join("mmdc.cmd");
            std::fs::write(&target, "@echo off\r\nexit /b 0\r\n").unwrap();
            let original = std::env::current_dir().unwrap();
            std::env::set_current_dir(&nested).unwrap();
            let found = resolve_mmdc_cwd_node_modules_bin();
            std::env::set_current_dir(original).unwrap();
            found
        };

        #[cfg(not(windows))]
        let found_path = {
            let target = bin.join("mmdc");
            std::fs::write(&target, "#!/bin/sh\nexit 0\n").unwrap();
            let original = std::env::current_dir().unwrap();
            std::env::set_current_dir(&nested).unwrap();
            let found = resolve_mmdc_cwd_node_modules_bin();
            std::env::set_current_dir(original).unwrap();
            found
        };

        assert!(found_path.is_some(), "expected mmdc under ancestor cwd");

        let _ = std::fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn mermaid_cli_can_render_minimal_diagram() {
        let result = render_mermaid_png("flowchart LR\n  A-->B\n")
            .expect("mermaid render call should not error");
        assert!(
            result.is_some(),
            "expected mermaid png when mmdc is available"
        );
    }

    #[test]
    fn html_preview_includes_print_page_rules() {
        let html = build_preview_html(&[MarkdownBlock::Paragraph {
            spans: vec![InlineSpan {
                text: "hello".to_string(),
                style: TextStyle::default(),
            }],
            source: BlockSource::single(1),
        }]);
        assert!(html.contains("@page { size: A4; margin: 0; }"), "{html}");
        assert!(html.contains("break-after: page"), "{html}");
        assert!(html.contains("page-break-after: always"), "{html}");
    }

    #[test]
    fn html_preview_constrains_mermaid_images_and_fallbacks() {
        let html = build_preview_html(&[MarkdownBlock::Paragraph {
            spans: vec![InlineSpan {
                text: "hello".to_string(),
                style: TextStyle::default(),
            }],
            source: BlockSource::single(1),
        }]);

        assert!(html.contains(".mermaid-frame"), "{html}");
        assert!(html.contains("max-height: 260pt"), "{html}");
        assert!(html.contains("page-break-inside: avoid"), "{html}");
        assert!(html.contains("break-inside: avoid"), "{html}");

        let image_html = mermaid_preview_image_html("Zm9vYmFy");
        assert!(image_html.contains("mermaid-frame"), "{image_html}");
        assert!(
            image_html.contains("data:image/png;base64,Zm9vYmFy"),
            "{image_html}"
        );

        let fallback_html = mermaid_preview_fallback_html();
        assert!(
            fallback_html.contains("Mermaid diagram preview unavailable"),
            "{fallback_html}"
        );
        assert!(!fallback_html.contains("<pre><code>"), "{fallback_html}");
    }

    #[test]
    fn mermaid_fallback_is_short_and_not_a_code_block() {
        let html = mermaid_preview_fallback_html();
        assert!(
            html.contains("Mermaid diagram preview unavailable"),
            "{html}"
        );
        assert!(html.contains("Mermaid CLI を確認してください。"), "{html}");
        assert!(!html.contains("<pre><code>"), "{html}");
    }

    #[test]
    fn normalize_mermaid_jpeg_bytes_flattens_transparency() {
        let mut png_bytes = Vec::new();
        let image = ImageBuffer::from_pixel(2, 2, image::Rgba([10, 20, 30, 128]));
        DynamicImage::ImageRgba8(image)
            .write_to(
                &mut std::io::Cursor::new(&mut png_bytes),
                ImageOutputFormat::Png,
            )
            .expect("test png should encode");

        let normalized =
            normalize_mermaid_jpeg_bytes(&png_bytes).expect("normalization should succeed");
        let decoded = image::load_from_memory(&normalized).expect("normalized png should decode");
        assert!(!decoded.color().has_alpha(), "{:?}", decoded.color());
        assert!(
            normalized.starts_with(&[0xFF, 0xD8]),
            "expected JPEG output"
        );
    }

    #[tokio::test]
    async fn mermaid_preview_pdf_does_not_emit_smask() {
        let bytes = preview_pdf_bytes(&[MarkdownBlock::CodeBlock {
            lang: "mermaid".to_string(),
            code: "flowchart LR\n  A-->B\n".to_string(),
            source: BlockSource::single(1),
        }]);

        assert!(
            !bytes.windows(7).any(|w| w == b"/SMask "),
            "expected Mermaid PDF to avoid soft masks"
        );
        assert!(
            bytes.windows(10).any(|w| w == b"/DCTDecode"),
            "expected Mermaid PDF image to be JPEG-compressed"
        );

        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let pdf_path = temp_dir.path().join("mermaid-preview.pdf");
        std::fs::write(&pdf_path, &bytes).expect("preview PDF should be written");
        let loaded = crate::commands::pdf_loader::load_pdf(pdf_path.to_string_lossy().to_string())
            .await
            .expect("generated preview PDF should be readable");
        assert_eq!(loaded.pages.len(), 1, "expected a single preview page");
    }

    #[test]
    fn fit_mermaid_image_to_page_centers_smaller_diagrams() {
        let (x, width, height) = fit_mermaid_image_to_page(400.0, 200.0);
        assert_eq!(width, 300.0);
        assert_eq!(height, 150.0);
        assert!((x - 147.5).abs() < 0.01, "x={x}");
    }

    #[test]
    fn fit_mermaid_image_to_page_caps_large_diagrams() {
        let (x, width, height) = fit_mermaid_image_to_page(1200.0, 800.0);
        assert!((width - 390.0).abs() < 0.01, "width={width}");
        assert!((x - 102.5).abs() < 0.01, "x={x}");
        assert!(
            height <= MERMAID_MAX_DISPLAY_HEIGHT + 0.01,
            "height={height}"
        );
    }

    #[test]
    fn list_item_content_x_increases_with_prefix_width() {
        let font = Font::Helvetica;
        let shallow = list_item_content_x(0, "•", &font, 11.0);
        let nested = list_item_content_x(1, "•", &font, 11.0);
        let ordered = list_item_content_x(0, "12.", &font, 11.0);

        assert!(shallow > MARGIN + 4.0, "shallow={shallow}");
        assert!(nested > shallow, "nested={nested}, shallow={shallow}");
        assert!(ordered >= shallow, "ordered={ordered}, shallow={shallow}");
    }

    #[test]
    fn nested_list_indent_uses_fixed_step_instead_of_literal_spaces() {
        let font = Font::Helvetica;
        let top_marker = list_marker_x(0);
        let nested_marker = list_marker_x(1);
        let deep_marker = list_marker_x(2);
        let top_text = list_item_content_x(0, "•", &font, 11.0);
        let nested_text = list_item_content_x(1, "•", &font, 11.0);

        assert!(
            (nested_marker - top_marker - LIST_INDENT_STEP_PT).abs() < f64::EPSILON,
            "top={top_marker}, nested={nested_marker}"
        );
        assert!(
            (deep_marker - nested_marker - LIST_INDENT_STEP_PT).abs() < f64::EPSILON,
            "nested={nested_marker}, deep={deep_marker}"
        );
        assert!(
            (nested_text - top_text - LIST_INDENT_STEP_PT).abs() < f64::EPSILON,
            "top_text={top_text}, nested_text={nested_text}"
        );
    }

    #[tokio::test]
    async fn oversized_first_code_block_does_not_create_a_leading_blank_page() {
        let code = (0..120)
            .map(|i| format!("line-{i:03}"))
            .collect::<Vec<_>>()
            .join("\n");
        let bytes = preview_pdf_bytes(&[MarkdownBlock::CodeBlock {
            lang: "text".to_string(),
            code,
            source: BlockSource { start_line: 1, end_line: 120 },
        }]);

        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let pdf_path = temp_dir.path().join("oversized-code-block.pdf");
        std::fs::write(&pdf_path, bytes).expect("preview PDF should be written");

        let loaded = crate::commands::pdf_loader::load_pdf(pdf_path.to_string_lossy().to_string())
            .await
            .expect("generated preview PDF should be readable");

        assert!(!loaded.pages.is_empty(), "expected at least one page");
        assert!(
            !loaded.pages[0].text_blocks.is_empty(),
            "expected the first page to contain code block content instead of being blank"
        );
    }

    #[tokio::test]
    async fn long_code_block_spans_multiple_pages() {
        let code = (0..260)
            .map(|i| format!("line-{i:03}"))
            .collect::<Vec<_>>()
            .join("\n");
        let bytes = preview_pdf_bytes(&[MarkdownBlock::CodeBlock {
            lang: "text".to_string(),
            code,
            source: BlockSource {
                start_line: 1,
                end_line: 260,
            },
        }]);

        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let pdf_path = temp_dir.path().join("long-code-block.pdf");
        std::fs::write(&pdf_path, bytes).expect("preview PDF should be written");

        let loaded = crate::commands::pdf_loader::load_pdf(pdf_path.to_string_lossy().to_string())
            .await
            .expect("generated preview PDF should be readable");

        assert!(
            loaded.pages.len() >= 2,
            "expected a long code block to flow onto multiple pages"
        );
        assert!(
            loaded.pages.iter().all(|page| !page.text_blocks.is_empty()),
            "expected every rendered page to contain some code text"
        );
    }
}
