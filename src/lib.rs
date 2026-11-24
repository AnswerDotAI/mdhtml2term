//! MDHTML Markdown rendered as terminal rows. The caller supplies the width after subtracting its gutter.
use comfy_table::{Cell, CellAlignment, ContentArrangement, LineStyle, Table, TableStyle};
use html2text::{config, render::RichAnnotation};
use mdhtml::{Block, Document, Inline, fast5ever::{DOCUMENT, Dom, NodeId, parse_fragment}, parse, plain,
    render::{EncodeNode, encode_document_parts_with, encode_document_with, escape_text}, resolve::{BakeOptions, RefsMode, bake_document}};
pub use mdhtml::resolve::CiteStyle;
use std::{borrow::Cow, fmt::Write, sync::LazyLock};
use syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

fn escaped(text: &str) -> String { let mut s = String::new(); escape_text(text, &mut s); s }

static SYNTAX: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEMES: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

fn theme() -> &'static syntect::highlighting::Theme {
    let name = match std::env::var("COLORFGBG").ok().as_deref().and_then(|s| s.rsplit(';').next()) {
        Some("7" | "15") => "base16-ocean.light", _ => "base16-ocean.dark",
    };
    &THEMES.themes[name]
}

fn code(text: &str, lang: &str) -> Option<String> {
    highlighted_html_for_string(text, &SYNTAX, SYNTAX.find_syntax_by_token(lang)?, theme()).ok()
}

/// Unwrapped syntax spans as (SGR prefix, original text). Text offsets include tabs and newlines.
pub fn highlight_spans<'a>(text: &'a str, lang: &str) -> Vec<(String, &'a str)> {
    use syntect::{easy::HighlightLines, util::LinesWithEndings};
    let Some(syntax) = SYNTAX.find_syntax_by_token(lang) else { return vec![(String::new(), text)] };
    let mut highlighter = HighlightLines::new(syntax, theme());
    let mut spans = Vec::new();
    for line in LinesWithEndings::from(text) {
        let Ok(tokens) = highlighter.highlight_line(line, &SYNTAX) else { return vec![(String::new(), text)] };
        for (style, token) in tokens {
            let c = style.foreground;
            spans.push((format!("\x1b[38;2;{};{};{}m", c.r, c.g, c.b), token));
        }
    }
    spans
}

fn children_html(dom: &Dom, id: NodeId) -> String {
    dom.children(id).iter().map(|&child| dom.to_html(child)).collect()
}

fn table(html: &str, width: u16) -> Option<String> {
    // Use the encoded cell contents: links, footnotes and citations already belong to the document.
    let dom = parse_fragment(html, "div");
    let root = *dom.element_children(DOCUMENT).first()?;
    let mut t = Table::new();
    t.load_style(TableStyle::new().header_separator(LineStyle::none().fill('─').junction('─'))).set_width(width).set_content_arrangement(ContentArrangement::Dynamic);
    let mut caption = String::new();
    for group in dom.element_children(root) {
        let header = match dom.tag(group) {
            Some("caption") => { caption = children_html(&dom, group); continue; }
            Some("thead") => true, Some("tbody" | "tfoot") => false, _ => continue,
        };
        let rows = dom.element_children(group);
        if header && rows.len() > 1 { return None; }
        for row in rows {
            let mut cells = Vec::new();
            for cell in dom.element_children(row) {
                if dom.attr(cell, "rowspan").is_some() || dom.attr(cell, "colspan").is_some() { return None; }
                let text = config::plain_no_decorate().min_wrap_width(1).string_from_read(children_html(&dom, cell).as_bytes(), width.into()).ok()?;
                cells.push(Cell::new(text.trim_end()).set_alignment(match dom.attr(cell, "align") {
                    Some("right") => CellAlignment::Right, Some("center") => CellAlignment::Center, _ => CellAlignment::Left,
                }));
            }
            if header { t.set_header(cells); } else { t.add_row(cells); }
        }
    }
    let text = t.to_string();
    if text.lines().any(|s| s.width() > usize::from(width)) { return None; }
    Some(format!("<div>{caption}<pre>{}</pre></div>", escaped(&text)))
}

fn checkboxes(html: &str) -> String {
    let mut dom = parse_fragment(html, "div");
    for id in dom.descendants(DOCUMENT) {
        if dom.tag(id) != Some("input") || dom.attr(id, "type") != Some("checkbox") { continue; }
        let marker = dom.create_text(if dom.attr(id, "checked").is_some() { "[x]" } else { "[ ]" });
        dom.replace_child(dom.parent(id).unwrap(), marker, id).unwrap();
    }
    dom.to_html(DOCUMENT)
}

fn bake(doc: &mut Document, cite_style: Option<CiteStyle>) -> Result<(), String> {
    bake_document(doc, &BakeOptions {refs: RefsMode::Lenient, links: false, cite_style, ..Default::default()}).map(|_| ())
}

fn decorate(node: EncodeNode<'_>, html: &str, width: u16) -> Option<String> {
    match node {
        EncodeNode::Block(Block::Heading {..}) => Some(format!("<strong style=\"color:magenta\">{html}</strong>")),
        EncodeNode::Block(Block::Table {..}) => table(html, width),
        EncodeNode::Block(Block::List {items, ..}) if items.iter().any(|i| i.checked.is_some()) => Some(checkboxes(html)),
        EncodeNode::Block(Block::CodeBlock {lang: Some(lang), text, ..}) => code(text, lang),
        EncodeNode::Inline(Inline::Link {url, children, ..}) if !url.starts_with('#') && plain(children) != *url =>
            Some(format!("{html} ({})", escaped(url))),
        _ => None,
    }
}

fn styled(out: &mut String, tags: &[RichAnnotation], text: &str) {
    use RichAnnotation::*;
    let start = out.len();
    for tag in tags { match tag {
        Strong => out.push_str("\x1b[1m"), Emphasis => out.push_str("\x1b[3m"), Strikeout => out.push_str("\x1b[9m"),
        Code => out.push_str("\x1b[36m"), Link(_) => out.push_str("\x1b[4;34m"), Image(_) => out.push_str("\x1b[2m"),
        Colour(rgb) => { let _ = write!(out, "\x1b[38;2;{};{};{}m", rgb.r, rgb.g, rgb.b); },
        _ => (),
    } }
    let reset = out.len() != start;
    out.extend(text.chars().filter(|c| !c.is_control()));
    if reset { out.push_str("\x1b[0m"); }
}

fn literal(text: &str, width: u16) -> Vec<String> {
    if width == 0 { return vec![]; }
    let clean: String = text.chars().filter(|&c| c == '\n' || !c.is_control())
        .map(|c| if c.width().unwrap_or(0) > usize::from(width) { '�' } else { c }).collect();
    textwrap::wrap(&clean, usize::from(width)).into_iter().map(|s| s.into_owned()).collect()
}

/// Render one complete reply. Each row has no trailing newline and resets its own SGR styles.
/// Resolve citations before layout, including references to a bibliography later in the reply.
pub fn render(source: &str, width: u16) -> Vec<String> {
    render_with_style(source, width, None)
}

/// Render a document using its citation style unless the caller supplies one.
pub fn render_with_style(source: &str, width: u16, cite_style: Option<CiteStyle>) -> Vec<String> {
    if source.is_empty() || width == 0 { return vec![]; }
    let mut doc = parse(source, &Default::default());
    let html = bake(&mut doc, cite_style).and_then(|_| encode_document_with(&doc, &|n,h| Ok(decorate(n,h,width)), true));
    html.map(|h| render_html(&h, width)).unwrap_or_else(|_| literal(source, width))
}

/// Render a notebook HTML fragment without interpreting it as Markdown.
pub fn render_html(html: &str, width: u16) -> Vec<String> {
    if width == 0 { return vec![]; }
    let config = config::rich().use_doc_css().unicode_strikeout(false).min_wrap_width(1).allow_width_overflow();
    let Ok(lines) = config.lines_from_read(html.as_bytes(), width.into()) else { return vec![] };
    let mut rows = Vec::new();
    for line in lines {
        // Nested list and quote prefixes can be wider than the terminal itself.
        if line.tagged_strings().map(|s| s.width()).sum::<usize>() > usize::from(width) {
            rows.extend(literal(&line.tagged_strings().map(|s| s.s.as_str()).collect::<String>(), width));
        } else {
            let mut row = String::new();
            for span in line.tagged_strings() { styled(&mut row, &span.tag, &span.s); }
            rows.push(row);
        }
    }
    rows
}

/// A selectable block's original Markdown, or the collected definitions for the endnotes.
pub struct Section<'a> { pub source: Cow<'a, str>, pub rows: Vec<String>, pub code: bool }

/// Resolve references across the document before laying out each top-level block.
pub fn sections(source: &str, width: u16) -> Vec<Section<'_>> {
    sections_with_style(source, width, None)
}

/// Render selectable sections with an optional override for the document's citation style.
pub fn sections_with_style(source: &str, width: u16, cite_style: Option<CiteStyle>) -> Vec<Section<'_>> {
    if source.is_empty() || width == 0 { return vec![]; }
    let (mut doc, sources) = mdhtml::parse_spanned(source, &Default::default());
    let result = bake(&mut doc, cite_style).and_then(|_| encode_document_parts_with(&doc, &|n,h| Ok(decorate(n,h,width)), true));
    let Ok((parts, endnotes)) = result else { return vec![Section { source: source.into(), rows: literal(source,width), code:false }]; };
    let mut sections: Vec<_> = sources.into_iter().zip(&doc.blocks).zip(parts).map(|((src,b),html)| Section {
        source: src.into(), rows: render_html(&html,width), code: matches!(b, Block::CodeBlock {..})
    }).collect();
    if !endnotes.is_empty() {
        let definitions = Document { footnotes: doc.footnotes, ..Default::default() };
        sections.push(Section { source: mdhtml::write_md::render_md(&definitions).into(), rows: render_html(&endnotes,width), code:false });
    }
    sections
}

/// Syntax-highlight literal source without interpreting any of it as Markdown.
pub fn highlight(text: &str, language: &str, width: u16) -> Vec<String> {
    if text.is_empty() || width == 0 { return vec![]; }
    let html = code(text,language).unwrap_or_else(|| format!("<pre>{}</pre>",escaped(text)));
    render_html(&html,width)
}
