use std::{ops::Range, time::Instant};

use druid::{
    text::{Attribute, AttributeSpans, RichText},
    Color, FontFamily, FontStyle, FontWeight,
};
use pulldown_cmark::{Event as ParseEvent, Parser, Tag};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::theme;

// Parse a markdown string and generate a `RichText` object with
/// the appropriate attributes.
pub fn markdown_to_text(text: &str) -> RichText {
    let mut current_pos = 0;
    let mut buffer = String::new();
    let mut attrs = AttributeSpans::new();
    let mut tag_stack = Vec::new();

    let parser = Parser::new(text);
    for event in parser {
        match event {
            ParseEvent::Start(tag) => {
                tag_stack.push((current_pos, tag));
            }
            ParseEvent::Text(txt) => {
                buffer.push_str(&txt);
                buffer.push_str(" ");
                current_pos += txt.len() + 1;
            }
            ParseEvent::End(end_tag) => {
                let (start_off, tag) = tag_stack
                    .pop()
                    .expect("parser does not return unbalanced tags");
                assert_eq!(end_tag, tag, "mismatched tags?");
                match tag {
                    Tag::CodeBlock(_) => highlighting_code(&buffer, start_off, &mut attrs),
                    _ => {}
                }
                add_attribute_for_tag(&tag, start_off..current_pos, &mut attrs);
                if add_newline_after_tag(&tag) {
                    buffer.push_str("\n\n");
                    current_pos += 2;
                }
            }
            ParseEvent::Code(txt) => {
                buffer.push_str(&txt);
                let range = current_pos..current_pos + txt.len();
                attrs.add(range, Attribute::font_family(theme::CODE_FONT));
                current_pos += txt.len();
            }
            ParseEvent::Html(txt) => {
                // buffer.push_str(&txt);
                // current_pos += txt.len();
                // let range = current_pos..current_pos + txt.len();
                // attrs.add(range.clone(), Attribute::font_family(FontFamily::MONOSPACE));
                // attrs.add(range, Attribute::text_color(BLOCKQUOTE_COLOR));
            }
            ParseEvent::HardBreak => {
                // buffer.push_str("\n\n");
                // current_pos += 1;
            }

            ParseEvent::FootnoteReference(_) => {}
            ParseEvent::SoftBreak => {}
            ParseEvent::Rule => {}
            ParseEvent::TaskListMarker(_) => {}
        }
    }
    let buffer = buffer.trim_end();
    RichText::new_with_attributes(buffer.into(), attrs)
}

fn add_newline_after_tag(tag: &Tag) -> bool {
    !matches!(
        tag,
        Tag::Emphasis | Tag::Strong | Tag::Strikethrough | Tag::Link(..)
    )
}

fn add_attribute_for_tag(tag: &Tag, range: Range<usize>, attrs: &mut AttributeSpans) {
    match tag {
        Tag::Heading(lvl) => {
            let font_size = match lvl {
                1 => 20.,
                2 => 18.0,
                3 => 17.0,
                _ => 16.0,
            };
            attrs.add(range.clone(), Attribute::size(font_size));
            attrs.add(range, Attribute::weight(FontWeight::MEDIUM));
        }
        Tag::BlockQuote => {
            // attrs.add(range.clone(), Attribute::style(FontStyle::Italic));
            // attrs.add(range, Attribute::text_color(BLOCKQUOTE_COLOR));
        }
        Tag::CodeBlock(_) => {
            attrs.add(range, Attribute::font_descriptor(theme::CODE_FONT));
        }
        Tag::Emphasis => attrs.add(range, Attribute::style(FontStyle::Italic)),
        Tag::Strong => attrs.add(range, Attribute::weight(FontWeight::BOLD)),
        Tag::Link(..) => {
            attrs.add(range.clone(), Attribute::underline(true));
            // attrs.add(range, Attribute::text_color(LINK_COLOR));
        }
        // ignore other tags for now
        _ => (),
    }
}

lazy_static::lazy_static! {
    pub static ref PS: SyntaxSet = SyntaxSet::load_defaults_newlines();
    pub static ref TS: ThemeSet = ThemeSet::load_defaults();
}

fn highlighting_code(txt: &str, range: usize, attrs: &mut AttributeSpans) {
    let txt = &txt[range..];

    let syntax = PS.find_syntax_by_extension("rs").unwrap();
    let mut h = HighlightLines::new(syntax, &TS.themes["Solarized (dark)"]);
    let mut current_pos = range;
    for line in LinesWithEndings::from(txt) {
        // LinesWithEndings enables use of newlines mode
        for (style, string) in h.highlight(line, &PS) {
            let range = current_pos..current_pos + string.len();
            let color = Color::rgba8(
                style.foreground.r,
                style.foreground.g,
                style.foreground.b,
                style.foreground.a,
            );
            attrs.add(range, Attribute::TextColor(color.into()));
            current_pos += string.len();
        }
    }
}
