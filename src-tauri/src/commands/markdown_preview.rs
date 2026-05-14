use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use base64::{engine::general_purpose, Engine as _};
use comrak::nodes::{AstNode, ListType, NodeCode, NodeCodeBlock, NodeValue};
use comrak::{parse_document, Arena, Options};
use image::GenericImageView;
use oxidize_pdf::{Color, Document, Font, Image, Page as OxidizePage};
use tauri::command;
use uuid::Uuid;

use crate::utils::font_manager;

const PAGE_WIDTH: f64 = 595.0;
const PAGE_HEIGHT: f64 = 842.0;
const MARGIN: f64 = 40.0;
const BASE_LINE_HEIGHT: f64 = 16.0;
const JP_SANS_FONT: &str = "NotoSansJP";
const JP_SANS_BOLD: &str = "NotoSansJP-Bold";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct TextStyle {
    bold: bool,
    italic: bool,
    code: bool,
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

#[derive(Debug, Clone)]
enum MarkdownBlock {
    Heading {
        level: usize,
        spans: Vec<InlineSpan>,
    },
    Paragraph(Vec<InlineSpan>),
    /// ネストした箇条書きは `indent` を増やしてフラット化する。
    ListItem {
        indent: u8,
        spans: Vec<InlineSpan>,
    },
    OrderedListItem {
        n: u32,
        indent: u8,
        spans: Vec<InlineSpan>,
    },
    /// `---` / `***` / `___` などの区切り線。
    ThematicBreak,
    /// 各行はセルごとのインライン列（列数は行ごとに揃う想定）
    Table(Vec<Vec<Vec<InlineSpan>>>),
    CodeBlock {
        lang: String,
        code: String,
    },
    Blockquote(Vec<InlineSpan>),
}

#[command]
pub async fn render_markdown_to_pdf_preview(markdown: String) -> Result<String, String> {
    let blocks = parse_markdown_blocks(&markdown);
    let output_path = build_preview_path();
    let bytes = build_preview_pdf(&blocks)?;

    fs::write(&output_path, &bytes)
        .map_err(|e| format!("プレビューPDFの書き込みに失敗しました: {e}"))?;

    Ok(output_path.to_string_lossy().to_string())
}

fn build_preview_path() -> PathBuf {
    std::env::temp_dir().join(format!("miniPDF-preview-{}.pdf", Uuid::new_v4()))
}

fn comrak_options() -> Options<'static> {
    let mut opts = Options::default();
    opts.extension.strikethrough = true;
    opts.extension.table = true;
    opts.extension.tagfilter = true;
    opts.extension.autolink = true;
    opts.extension.tasklist = true;
    opts
}

fn parse_markdown_blocks(markdown: &str) -> Vec<MarkdownBlock> {
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &comrak_options());
    let mut blocks = Vec::new();
    push_blocks_from_children(root, &mut blocks);
    blocks
}

fn push_blocks_from_children<'a>(node: &'a AstNode<'a>, blocks: &mut Vec<MarkdownBlock>) {
    for child in node.children() {
        push_block(child, blocks);
    }
}

/// `List` ノードをフラットな `ListItem` / `OrderedListItem` に展開する（子 `List` は `indent+1`）。
fn flatten_list_blocks<'a>(list_node: &'a AstNode<'a>, indent: u8, blocks: &mut Vec<MarkdownBlock>) {
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
                                blocks.push(MarkdownBlock::ListItem { indent, spans });
                            }
                        }
                        NodeValue::List(_) => {
                            flatten_list_blocks(child, indent.saturating_add(1), blocks);
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
                            });
                        }
                        NodeValue::List(_) => {
                            flatten_list_blocks(child, indent.saturating_add(1), blocks);
                        }
                        _ => {}
                    }
                }
                n = n.saturating_add(1);
            }
        }
    }
}

fn push_block<'a>(node: &'a AstNode<'a>, blocks: &mut Vec<MarkdownBlock>) {
    match &node.data.borrow().value {
        NodeValue::Document => push_blocks_from_children(node, blocks),
        NodeValue::FrontMatter(_) => {}
        NodeValue::BlockQuote => {
            let mut paras: Vec<Vec<InlineSpan>> = Vec::new();
            collect_blockquote_content(node, &mut paras);
            for p in paras {
                if !p.is_empty() {
                    blocks.push(MarkdownBlock::Blockquote(p));
                }
            }
        }
        NodeValue::MultilineBlockQuote(_) => {
            let mut paras: Vec<Vec<InlineSpan>> = Vec::new();
            collect_blockquote_content(node, &mut paras);
            for p in paras {
                if !p.is_empty() {
                    blocks.push(MarkdownBlock::Blockquote(p));
                }
            }
        }
        NodeValue::List(_) => {
            flatten_list_blocks(node, 0, blocks);
        }
        NodeValue::Paragraph => {
            let spans = paragraph_spans(node);
            if !spans.is_empty() {
                blocks.push(MarkdownBlock::Paragraph(spans));
            }
        }
        NodeValue::Heading(h) => {
            let spans = paragraph_spans(node);
            if !spans.is_empty() {
                blocks.push(MarkdownBlock::Heading {
                    level: h.level as usize,
                    spans,
                });
            }
        }
        NodeValue::CodeBlock(nb) => {
            blocks.push(code_block_from_node(nb));
        }
        NodeValue::Table(_) => {
            if let Some(t) = table_from_node(node) {
                blocks.push(t);
            }
        }
        NodeValue::ThematicBreak => {
            blocks.push(MarkdownBlock::ThematicBreak);
        }
        NodeValue::HtmlBlock(nb) => {
            let t = nb.literal.trim();
            if !t.is_empty() {
                let mut v = Vec::new();
                InlineSpan::push(&mut v, t, TextStyle::default());
                blocks.push(MarkdownBlock::Paragraph(v));
            }
        }
        NodeValue::Item(_) => {
            let spans = item_first_paragraph_spans(node);
            if !spans.is_empty() {
                blocks.push(MarkdownBlock::ListItem {
                    indent: 0,
                    spans,
                });
            }
        }
        _ => {
            for c in node.children() {
                push_block(c, blocks);
            }
        }
    }
}

fn code_block_from_node(nb: &NodeCodeBlock) -> MarkdownBlock {
    MarkdownBlock::CodeBlock {
        lang: nb.info.trim().to_ascii_lowercase(),
        code: nb.literal.clone(),
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
        Some(MarkdownBlock::Table(rows))
    }
}

fn collect_blockquote_content<'a>(node: &'a AstNode<'a>, paras: &mut Vec<Vec<InlineSpan>>) {
    for c in node.children() {
        match &c.data.borrow().value {
            NodeValue::Paragraph => paras.push(paragraph_spans(c)),
            NodeValue::BlockQuote | NodeValue::MultilineBlockQuote(_) => {
                collect_blockquote_content(c, paras);
            }
            NodeValue::List(_) => {
                let mut flat = Vec::new();
                flatten_list_blocks(c, 0, &mut flat);
                for b in flat {
                    match b {
                        MarkdownBlock::ListItem { indent, mut spans } => {
                            if spans.is_empty() {
                                continue;
                            }
                            let mut prefix = Vec::new();
                            let pad = "  ".repeat(1 + indent as usize);
                            InlineSpan::push(&mut prefix, &format!("{pad}• "), TextStyle::default());
                            prefix.append(&mut spans);
                            paras.push(prefix);
                        }
                        MarkdownBlock::OrderedListItem { n, indent, mut spans } => {
                            let mut prefix = Vec::new();
                            let pad = "  ".repeat(1 + indent as usize);
                            InlineSpan::push(
                                &mut prefix,
                                &format!("{pad}{n}. "),
                                TextStyle::default(),
                            );
                            prefix.append(&mut spans);
                            paras.push(prefix);
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                let mut tmp = Vec::new();
                push_block(c, &mut tmp);
                for b in tmp {
                    match b {
                        MarkdownBlock::Paragraph(s) | MarkdownBlock::Blockquote(s) => {
                            if !s.is_empty() {
                                paras.push(s);
                            }
                        }
                        MarkdownBlock::Heading { spans, .. }
                            if !spans.is_empty() =>
                        {
                            paras.push(spans);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
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
        NodeValue::Text(t) => InlineSpan::push(out, t, inherited),
        NodeValue::SoftBreak => InlineSpan::push(out, "\n", inherited),
        NodeValue::LineBreak => InlineSpan::push(out, "\n", inherited),
        NodeValue::Code(NodeCode { literal, .. }) => {
            InlineSpan::push(
                out,
                literal,
                TextStyle {
                    bold: false,
                    italic: false,
                    code: true,
                },
            );
        }
        NodeValue::Strong => {
            let st = TextStyle {
                bold: true,
                italic: inherited.italic,
                code: false,
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
            for c in node.children() {
                collect_inline_spans(c, inherited, out);
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
            InlineSpan::push(
                out,
                &format!("[画像: {label}]"),
                inherited,
            );
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
        let non_ascii = !ascii_only(text);
        match (style.bold, style.italic) {
            (true, _) if non_ascii => {
                if self.has_custom_bold {
                    if style.italic {
                        self.body_bold_italic.clone()
                    } else {
                        self.body_bold.clone()
                    }
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
                if self.has_custom_bold {
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

fn ascii_only(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii())
}

fn make_preview_face(doc: &Document, body: Font) -> PreviewFace {
    let has_bold = doc.has_custom_font(JP_SANS_BOLD);
    PreviewFace {
        body: body.clone(),
        body_bold: if has_bold {
            Font::custom(JP_SANS_BOLD)
        } else {
            body.clone()
        },
        body_italic: Font::HelveticaOblique,
        body_bold_italic: if has_bold {
            Font::custom(JP_SANS_BOLD)
        } else {
            body.clone()
        },
        has_custom_bold: has_bold,
    }
}

fn build_preview_pdf(blocks: &[MarkdownBlock]) -> Result<Vec<u8>, String> {
    let mut doc = Document::new();
    font_manager::register_fonts_on_document(&mut doc)?;

    let body_font = if doc.has_custom_font(JP_SANS_FONT) {
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
            MarkdownBlock::Heading { level, spans } => {
                let font_size = match level {
                    1 => 24.0,
                    2 => 20.0,
                    3 => 18.0,
                    _ => 16.0,
                };
                let est = estimate_rich_block_height(spans, max_chars_for_font(font_size), font_size);
                cursor_y = ensure_room(cursor_y, est + 8.0, &mut page, &mut doc);
                draw_rich_block(
                    &mut page,
                    spans,
                    &face,
                    font_size,
                    MARGIN,
                    &mut cursor_y,
                    (font_size * 1.2).max(BASE_LINE_HEIGHT),
                    max_chars_for_font(font_size),
                )?;
                cursor_y -= 8.0;
            }
            MarkdownBlock::Paragraph(spans) => {
                let est = estimate_rich_block_height(spans, 90, 11.5);
                cursor_y = ensure_room(cursor_y, est + 6.0, &mut page, &mut doc);
                draw_rich_block(
                    &mut page,
                    spans,
                    &face,
                    11.5,
                    MARGIN,
                    &mut cursor_y,
                    BASE_LINE_HEIGHT,
                    90,
                )?;
                cursor_y -= 6.0;
            }
            MarkdownBlock::ListItem { indent, spans } => {
                let ind = *indent;
                let bullet = if ind > 0 {
                    format!("{}• ", "  ".repeat(ind as usize))
                } else {
                    "• ".to_string()
                };
                let prefix_cols = unicode_display_width(bullet.as_str());
                let budget = 90usize.saturating_sub(prefix_cols + 4).max(20);
                let est = estimate_rich_block_height(spans, budget, 11.0) + BASE_LINE_HEIGHT;
                cursor_y = ensure_room(cursor_y, est, &mut page, &mut doc);
                let lines = layout_spans_lines(spans, budget);
                let mut first = true;
                for line in lines {
                    let mut prefixed = Vec::new();
                    if first {
                        first = false;
                        InlineSpan::push(&mut prefixed, &bullet, TextStyle::default());
                    } else {
                        InlineSpan::push(
                            &mut prefixed,
                            &" ".repeat(prefix_cols.max(1)),
                            TextStyle::default(),
                        );
                    }
                    prefixed.extend(line);
                    draw_rich_line_segments(
                        &mut page,
                        &prefixed,
                        &face,
                        11.0,
                        MARGIN + 4.0,
                        &mut cursor_y,
                        BASE_LINE_HEIGHT,
                    )?;
                }
                cursor_y -= 2.0;
            }
            MarkdownBlock::OrderedListItem { n, indent, spans } => {
                let ind = *indent;
                let prefix = if ind > 0 {
                    format!("{}{}. ", "  ".repeat(ind as usize), n)
                } else {
                    format!("{n}. ")
                };
                let prefix_cols = unicode_display_width(prefix.as_str());
                let budget = 90usize.saturating_sub(prefix_cols + 4).max(20);
                let est = estimate_rich_block_height(spans, budget, 11.0) + BASE_LINE_HEIGHT;
                cursor_y = ensure_room(cursor_y, est, &mut page, &mut doc);
                let lines = layout_spans_lines(spans, budget);
                let mut idx = 0;
                for line in lines {
                    let mut row = Vec::new();
                    if idx == 0 {
                        InlineSpan::push(
                            &mut row,
                            &prefix,
                            TextStyle::default(),
                        );
                        idx += 1;
                    } else {
                        InlineSpan::push(
                            &mut row,
                            &" ".repeat(prefix_cols.max(1)),
                            TextStyle::default(),
                        );
                    }
                    row.extend(line);
                    draw_rich_line_segments(
                        &mut page,
                        &row,
                        &face,
                        11.0,
                        MARGIN + 4.0,
                        &mut cursor_y,
                        BASE_LINE_HEIGHT,
                    )?;
                }
                cursor_y -= 2.0;
            }
            MarkdownBlock::ThematicBreak => {
                let h = 22.0_f64;
                cursor_y = ensure_room(cursor_y, h, &mut page, &mut doc);
                let y = cursor_y - h * 0.45;
                page.graphics()
                    .set_stroke_color(Color::gray(0.72))
                    .set_line_width(0.55)
                    .move_to(MARGIN + 6.0, y)
                    .line_to(PAGE_WIDTH - MARGIN - 6.0, y)
                    .stroke();
                cursor_y -= h;
            }
            MarkdownBlock::Table(rows) => {
                let row_h = 16.0_f64;
                let n = rows.len().max(1);
                let est = row_h * n as f64 + 16.0;
                cursor_y = ensure_room(cursor_y, est, &mut page, &mut doc);
                draw_table(&mut page, rows, &face, &mut cursor_y, row_h)?;
                cursor_y -= 6.0;
            }
            MarkdownBlock::CodeBlock { lang, code } => {
                if lang == "mermaid" {
                    if let Some((png_data, width, height)) = render_mermaid_png(code)? {
                        let draw_width = PAGE_WIDTH - (MARGIN * 2.0);
                        let aspect = if width > 0.0 { height / width } else { 1.0 };
                        let draw_height = (draw_width * aspect).min(260.0).max(80.0);
                        cursor_y = ensure_room(cursor_y, draw_height + 18.0, &mut page, &mut doc);
                        draw_image_on_page(
                            &mut page,
                            &png_data,
                            "png",
                            MARGIN,
                            cursor_y - draw_height,
                            draw_width,
                            draw_height,
                        )?;
                        cursor_y -= draw_height + 10.0;
                        continue;
                    }
                }

                let code_lines = wrap_code_lines(code, 88);
                let lh = BASE_LINE_HEIGHT - 1.0;
                let pad = 6.0;
                let fs = 9.5;
                let block_h = code_lines.len() as f64 * lh + pad * 2.0;
                cursor_y = ensure_room(cursor_y, block_h + 10.0, &mut page, &mut doc);

                let y_top = cursor_y;
                let y_bottom = y_top - block_h;
                page.graphics()
                    .set_fill_color(Color::rgb(0.88, 0.89, 0.92))
                    .rectangle(MARGIN, y_bottom, PAGE_WIDTH - 2.0 * MARGIN, block_h)
                    .fill();

                if lang == "mermaid" {
                    let hint = "(Mermaid: MINIPDF_MERMAID_CLI、リポジトリの node_modules/.bin/mmdc、実行ファイル隣の mmdc、または PATH の mmdc が必要です)";
                    let mut hy = y_top - pad - fs * 0.85;
                    page.text()
                        .set_fill_color(Color::rgb(0.05, 0.05, 0.07))
                        .set_font(face.body.clone(), 8.5)
                        .at(MARGIN + pad, hy)
                        .write(hint)
                        .map_err(|e| format!("テキスト描画に失敗しました: {e}"))?;
                    hy -= lh;
                    for line in &code_lines {
                        let line_font = if ascii_only(line) {
                            Font::Courier
                        } else {
                            face.body.clone()
                        };
                        page.text()
                            .set_fill_color(Color::rgb(0.05, 0.05, 0.07))
                            .set_font(line_font, fs)
                            .at(MARGIN + pad, hy)
                            .write(line)
                            .map_err(|e| format!("テキスト描画に失敗しました: {e}"))?;
                        hy -= lh;
                    }
                    cursor_y = y_bottom - 6.0;
                } else {
                    let mut hy = y_top - pad - fs * 0.85;
                    for line in &code_lines {
                        let line_font = if ascii_only(line) {
                            Font::Courier
                        } else {
                            face.body.clone()
                        };
                        page.text()
                            .set_fill_color(Color::rgb(0.05, 0.05, 0.07))
                            .set_font(line_font, fs)
                            .at(MARGIN + pad, hy)
                            .write(line)
                            .map_err(|e| format!("テキスト描画に失敗しました: {e}"))?;
                        hy -= lh;
                    }
                    cursor_y = y_bottom - 6.0;
                }
            }
            MarkdownBlock::Blockquote(spans) => {
                let lines = layout_spans_lines(spans, 84);
                let h = lines.len().max(1) as f64 * BASE_LINE_HEIGHT + 6.0;
                cursor_y = ensure_room(cursor_y, h + 4.0, &mut page, &mut doc);
                let y_top = cursor_y;
                let y_bottom = y_top - h;
                let body_w = PAGE_WIDTH - 2.0 * MARGIN - 14.0;
                page.graphics()
                    .set_fill_color(Color::rgb(0.96, 0.97, 0.99))
                    .rectangle(MARGIN + 6.0, y_bottom, body_w, h)
                    .fill();
                page.graphics()
                    .set_fill_color(Color::rgb(0.45, 0.55, 0.78))
                    .rectangle(MARGIN + 2.0, y_bottom, 4.5, h)
                    .fill();
                draw_rich_block(
                    &mut page,
                    spans,
                    &face,
                    11.0,
                    MARGIN + 10.0,
                    &mut cursor_y,
                    BASE_LINE_HEIGHT,
                    84,
                )?;
                cursor_y -= 4.0;
            }
        }
    }

    doc.add_page(page);
    doc.to_bytes()
        .map_err(|e| format!("プレビューPDFの生成に失敗しました: {e}"))
}

fn estimate_rich_block_height(spans: &[InlineSpan], max_cols: usize, font_size: f64) -> f64 {
    let lines = layout_spans_lines(spans, max_cols);
    let lh = if font_size >= 18.0 {
        (font_size * 1.2).max(BASE_LINE_HEIGHT)
    } else {
        BASE_LINE_HEIGHT
    };
    lines.len().max(1) as f64 * lh
}

fn draw_rich_block(
    page: &mut OxidizePage,
    spans: &[InlineSpan],
    face: &PreviewFace,
    default_size: f64,
    x0: f64,
    cursor_y: &mut f64,
    line_height: f64,
    max_cols: usize,
) -> Result<(), String> {
    let lines = layout_spans_lines(spans, max_cols);
    for line in lines {
        draw_rich_line_segments(page, &line, face, default_size, x0, cursor_y, line_height)?;
    }
    Ok(())
}

fn layout_spans_lines(spans: &[InlineSpan], max_cols: usize) -> Vec<Vec<InlineSpan>> {
    let flat = flatten_spans(spans);
    if flat.is_empty() {
        return vec![vec![]];
    }
    let mut lines: Vec<Vec<InlineSpan>> = Vec::new();
    let mut cur_line: Vec<(String, TextStyle)> = Vec::new();
    let mut cur_width = 0usize;

    let flush_line = |cur_line: &mut Vec<(String, TextStyle)>, lines: &mut Vec<Vec<InlineSpan>>| {
        if cur_line.is_empty() {
            return;
        }
        let mut out = Vec::new();
        for (t, st) in cur_line.drain(..) {
            InlineSpan::push(&mut out, &t, st);
        }
        merge_adjacent_spans(&mut out);
        lines.push(out);
    };

    for (ch, st) in flat {
        if ch == '\n' {
            flush_line(&mut cur_line, &mut lines);
            cur_width = 0;
            continue;
        }
        let w = char_display_width(ch);
        if cur_width + w > max_cols && cur_width > 0 {
            flush_line(&mut cur_line, &mut lines);
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
    flush_line(&mut cur_line, &mut lines);
    if lines.is_empty() {
        lines.push(vec![]);
    }
    lines
}

fn flatten_spans(spans: &[InlineSpan]) -> Vec<(char, TextStyle)> {
    let mut v = Vec::new();
    for s in spans {
        for ch in s.text.chars() {
            v.push((ch, s.style));
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
) -> Result<(), String> {
    let y = *cursor_y;
    let mut x = x0;
    for seg in line {
        let size = if seg.style.code {
            (default_size - 1.0).max(8.5)
        } else {
            default_size
        };
        let font = face.pick(seg.style, &seg.text);
        if seg.text.is_empty() {
            continue;
        }
        if seg.style.code {
            let w = approx_width_pt(&seg.text, &font, size).max(size * 0.6);
            let pad_y = size * 0.25;
            let box_h = size * 1.2;
            let box_bottom = y - pad_y;
            page.graphics()
                .set_fill_color(Color::rgb(0.91, 0.93, 0.96))
                .rectangle(x - 1.5, box_bottom - 1.0, w + 3.0, box_h + 2.0)
                .fill();
        }
        let fill = if seg.style.bold && !ascii_only(&seg.text) && !face.has_custom_bold {
            Color::rgb(0.04, 0.04, 0.07)
        } else if seg.style.code {
            Color::rgb(0.08, 0.1, 0.14)
        } else if seg.style.italic {
            Color::rgb(0.15, 0.16, 0.2)
        } else {
            Color::rgb(0.02, 0.02, 0.04)
        };
        page.text()
            .set_fill_color(fill)
            .set_font(font.clone(), size)
            .at(x, y)
            .write(&seg.text)
            .map_err(|e| format!("テキスト描画に失敗しました: {e}"))?;
        x += approx_width_pt(&seg.text, &font, size);
    }
    *cursor_y -= line_height;
    Ok(())
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
        | Font::CourierBoldOblique => oxidize_pdf::text::measure_text(text, font.clone(), font_size),
        Font::TimesRoman
        | Font::TimesBold
        | Font::TimesItalic
        | Font::TimesBoldItalic => oxidize_pdf::text::measure_text(text, font.clone(), font_size),
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

fn cell_plain_text(row: &[Vec<InlineSpan>], ci: usize) -> String {
    row.get(ci)
        .map(|c| c.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join(""))
        .unwrap_or_default()
}

/// 表全体を一度に罫線で囲み、行ごとの縦線の重ね描きを避ける（ずれ・破線の主因だった）。
fn draw_table(
    page: &mut OxidizePage,
    rows: &[Vec<Vec<InlineSpan>>],
    face: &PreviewFace,
    cursor_y: &mut f64,
    row_h: f64,
) -> Result<(), String> {
    if rows.is_empty() {
        return Ok(());
    }
    let fs = 9.5;
    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(1).max(1);
    let nrows = rows.len();
    let table_left = MARGIN + 4.0;
    let table_width = PAGE_WIDTH - 2.0 * MARGIN - 12.0;
    let col_w = table_width / ncols as f64;
    let max_cell_cols = ((col_w - 8.0) / (fs * 0.52)).max(3.0).floor() as usize;

    let table_top = *cursor_y;
    let table_bottom = table_top - nrows as f64 * row_h;

    for ri in 0..=nrows {
        let y = table_top - ri as f64 * row_h;
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

    for (ri, row) in rows.iter().enumerate() {
        let baseline = table_top - ri as f64 * row_h - row_h * 0.72;
        for ci in 0..ncols {
            let flat = cell_plain_text(row, ci);
            let first_line = wrap_text(&flat, max_cell_cols)
                .into_iter()
                .next()
                .unwrap_or_default();
            page.text()
                .set_fill_color(Color::rgb(0.02, 0.02, 0.04))
                .set_font(face.body.clone(), fs)
                .at(table_left + col_w * ci as f64 + 4.0, baseline)
                .write(&first_line)
                .map_err(|e| format!("テキスト描画に失敗しました: {e}"))?;
        }
    }

    *cursor_y = table_bottom - 6.0;
    Ok(())
}

fn ensure_room(
    cursor_y: f64,
    required_height: f64,
    page: &mut OxidizePage,
    doc: &mut Document,
) -> f64 {
    if cursor_y - required_height > MARGIN {
        return cursor_y;
    }
    let mut next_page = OxidizePage::new(PAGE_WIDTH, PAGE_HEIGHT);
    std::mem::swap(page, &mut next_page);
    doc.add_page(next_page);
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
    for raw in code.lines() {
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
        if !buf.is_empty() {
            out.push(buf);
        }
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

/// `npm install` 済みの開発用リポジトリでは `node_modules/.bin/mmdc` をそのまま使う。
fn resolve_mmdc_workspace_node_modules_bin() -> Option<PathBuf> {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent()?;
    let bin = workspace_root.join("node_modules").join(".bin");

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

fn resolve_mmdc_executable() -> PathBuf {
    if let Some(p) = std::env::var_os("MINIPDF_MERMAID_CLI") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }

    if let Some(p) = resolve_mmdc_workspace_node_modules_bin() {
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

    let png_data = fs::read(&output_path)
        .map_err(|e| format!("Mermaid画像の読み込みに失敗しました: {e}"))?;
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
    page.graphics()
        .draw_image(&image_name, x, y, width, height);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
                },
                MarkdownBlock::OrderedListItem {
                    n: 2,
                    indent: 0,
                    spans: b,
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
            MarkdownBlock::Table(rows) => assert!(rows.len() >= 2, "rows={:?}", rows),
            _ => panic!("expected table"),
        }
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
            MarkdownBlock::Paragraph(spans) => {
                assert!(
                    spans.iter().any(|s| s.style.bold && s.text.contains("world")),
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
            MarkdownBlock::Paragraph(spans) => {
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
    fn nested_bullet_list_is_flattened_with_indent() {
        let md = "- a\n- b\n  - nested\n";
        let b = parse_markdown_blocks(md);
        assert_eq!(b.len(), 3, "{b:?}");
        match (&b[0], &b[1], &b[2]) {
            (
                MarkdownBlock::ListItem {
                    indent: 0,
                    spans: a,
                },
                MarkdownBlock::ListItem {
                    indent: 0,
                    spans: b,
                },
                MarkdownBlock::ListItem {
                    indent: 1,
                    spans: n,
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
            b.iter().any(|x| matches!(x, MarkdownBlock::ThematicBreak)),
            "{b:?}"
        );
    }

    fn plain_text(spans: &[InlineSpan]) -> String {
        spans.iter().map(|s| s.text.as_str()).collect()
    }
}
