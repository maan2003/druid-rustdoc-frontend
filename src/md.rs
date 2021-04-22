use druid::text::{AttributesAdder, RichTextBuilder};
use druid::Selector;
use druid::{
    text::{Attribute, AttributeSpans, RichText},
    Color, FontStyle, FontWeight,
};
use pulldown_cmark::{Event as ParseEvent, Parser, Tag};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::delegate::OPEN_LINK;
use crate::theme;
const BLOCKQUOTE_COLOR: Color = Color::grey8(0x88);
const LINK_COLOR: Color = Color::from_rgba32_u32(0x39AFD7FF);

// Parse a markdown string and generate a `RichText` object with
/// the appropriate attributes.
pub fn markdown_to_text(text: &str) -> RichText {
    let mut current_pos = 0;
    let mut builder = RichTextBuilder::new();
    let mut tag_stack = Vec::new();
    let mut is_code = false;

    let parser = Parser::new(text);
    for event in parser {
        match event {
            ParseEvent::Start(tag) => {
                if let Tag::CodeBlock(..) = tag {
                    is_code = true;
                } else if let Tag::Item = tag {
                    builder.push("• ");
                    current_pos += 2;
                }
                tag_stack.push((current_pos, tag));
            }
            ParseEvent::Text(mut txt) => {
                if is_code {
                    txt = txt
                        .split_inclusive('\n')
                        .filter(|l| !l.starts_with("# "))
                        .collect::<String>()
                        .into();
                }
                builder.push(&txt);
                builder.push(" ");
                current_pos += txt.len() + 1;
            }
            ParseEvent::End(end_tag) => {
                let (start_off, tag) = tag_stack
                    .pop()
                    .expect("parser does not return unbalanced tags");
                assert_eq!(end_tag, tag, "mismatched tags?");
                match tag {
                    Tag::CodeBlock(_) => {
                        let (buffer, attrs) = builder.raw_parts();
                        highlighting_code(buffer, start_off, attrs);
                        is_code = false;
                    }
                    _ => {}
                }
                add_attribute_for_tag(
                    &tag,
                    builder.add_attributes_for_range(start_off..current_pos),
                );
                for _ in 0..newlines_after_tag(&tag) {
                    builder.push("\n");
                    current_pos += 1;
                }
            }
            ParseEvent::Code(txt) => {
                builder
                    .push(&txt)
                    .font_descriptor(theme::CODE_FONT)
                    .text_color(theme::CODE_COLOR);
                current_pos += txt.len();
            }
            ParseEvent::Html(txt) => {
                builder
                    .push(&txt)
                    .font_descriptor(theme::CODE_FONT)
                    .text_color(BLOCKQUOTE_COLOR);
                current_pos += txt.len();
            }
            ParseEvent::HardBreak => {
                builder.push("\n\n");
                current_pos += 2;
            }
            _ => (),
        }
    }
    let (string, _) = builder.raw_parts();
    if string.ends_with("\n\n") {
        string.truncate(string.len() - 2);
    }
    builder.build()
}

fn newlines_after_tag(tag: &Tag) -> usize {
    match tag {
        Tag::Emphasis | Tag::Strong | Tag::Strikethrough | Tag::Link(..) => 0,
        Tag::Item => 1,
        Tag::List(..) => 1,
        Tag::Paragraph => 2,
        _ => 2,
    }
}

fn add_attribute_for_tag(tag: &Tag, mut attrs: AttributesAdder) {
    match tag {
        Tag::Heading(lvl) => {
            let font_size = match lvl {
                1 => 20.,
                2 => 18.0,
                3 => 17.0,
                _ => 16.0,
            };
            attrs.size(font_size).weight(FontWeight::BOLD);
        }
        Tag::BlockQuote => {
            attrs.style(FontStyle::Italic).text_color(BLOCKQUOTE_COLOR);
        }
        Tag::CodeBlock(_) => {
            attrs.font_descriptor(theme::CODE_FONT);
        }
        Tag::Emphasis => {
            attrs.style(FontStyle::Italic);
        }
        Tag::Strong => {
            attrs.weight(FontWeight::BOLD);
        }
        Tag::Link(_link_ty, target, _title) => {
            attrs
                .underline(true)
                .text_color(LINK_COLOR)
                .link(OPEN_LINK.with(target.to_string()));
        }
        // ignore other tags for now
        _ => {}
    }
}

lazy_static::lazy_static! {
    pub static ref PS: SyntaxSet = SyntaxSet::load_defaults_newlines();
    pub static ref TS: ThemeSet = ThemeSet::load_from_folder("themes").unwrap();
}

fn highlighting_code(txt: &str, range: usize, attrs: &mut AttributeSpans) {
    let txt = &txt[range..];

    let syntax = PS.find_syntax_by_extension("rs").unwrap();
    let mut h = HighlightLines::new(syntax, &TS.themes["Ultimate Dark"]);
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
