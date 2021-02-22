use super::*;

pub(crate) fn function(item: &Item, summary: Option<&ItemSummary>) -> impl Widget<()> {
    let (type_, color) = item_kind_str(item);

    let mut name = RichTextBuilder::new();
    name.push("fn ");
    dbg!(&item.links);
    for modu in summary
        .into_iter()
        .flat_map(|x| x.path.iter().take(x.path.len() - 1))
    {
        name.push(modu).text_color(theme::MOD_COLOR);
        name.push("::");
    }

    name.push(item.name.as_ref().unwrap())
        .text_color(theme::FUNCTION_COLOR)
        .weight(FontWeight::SEMI_BOLD);

    let name = RawLabel::new()
        .with_font(theme::CODE_FONT)
        .with_text_size(24.0)
        .lens(Constant(name.build()));

    let mut syntax = RichTextBuilder::new();
    if item.inner.as_fn().unwrap().decl.inputs.len() > 2 {
        format_fn_multiline(item, &mut syntax);
    } else {
        format_fn(item, &mut syntax);
    }
    let syntax = RawLabel::new()
        .with_font(theme::CODE_FONT)
        .lens(Constant(syntax.build()));

    let docs = markdown_to_text(&item.docs.as_deref().unwrap_or(""));
    let docs = RawLabel::new()
        .with_line_break_mode(LineBreaking::WordWrap)
        .lens(Constant(docs));

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(name)
        .with_default_spacer()
        .with_child(
            Seperator::new()
                .with_color(Color::Rgba32(0x5c6773ff))
                .with_size(1.0)
                .with_stroke_style(StrokeStyle::new().dash(vec![2.0], 0.0)),
        )
        .with_default_spacer()
        .with_child(syntax)
        .with_default_spacer()
        .with_flex_child(Scroll::new(docs).vertical(), 1.)
}
