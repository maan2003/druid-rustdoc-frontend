use super::*;

pub fn module(item: &Item, summary: Option<&ItemSummary>) -> impl Widget<()> {
    let (type_, color) = item_kind_str(item);

    let mut name = RichTextBuilder::new();
    name.push(type_);
    name.push(" ");
    name.push(item.name.as_ref().unwrap())
        .text_color(color)
        .weight(FontWeight::MEDIUM);
    let name = RawLabel::new()
        .with_font(theme::CODE_FONT)
        .with_text_size(24.0)
        .lens(Constant(name.build()));

    let docs = markdown_to_text(&item.docs.as_deref().unwrap_or(""));
    let docs = RawLabel::new()
        .with_line_break_mode(LineBreaking::WordWrap)
        .lens(Constant(docs));
    let m = item.inner.as_mod().unwrap();

    let items = ItemsWidget::new(m.items.iter().cloned().collect(), |mut items, _env| {
        let mut prev_kind = ItemKind::AssocConst;
        let mut current_names = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);
        let mut all_names = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);
        let mut sum = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);
        items.sort_by(|(a, _), (b, _)| {
            (a.kind as u8).cmp(&(b.kind as u8)).then_with(|| {
                a.name
                    .as_ref()
                    .unwrap_or(&"_".into())
                    .cmp(b.name.as_ref().unwrap_or(&"_".into()))
            })
        });
        let mut is_first = true;
        let mut heading = if items.is_empty() {
            String::new()
        } else {
            format!("{}s", item_kind_str(&items[0].0).0)
        };
        for (item, summary) in &items {
            if item.kind != prev_kind {
                if !is_first {
                    let heading =
                        std::mem::replace(&mut heading, format!("{}s", item_kind_str(&item).0));
                    let heading = Label::new(heading).with_text_size(22.4);
                    let items = Flex::row()
                        .cross_axis_alignment(CrossAxisAlignment::Start)
                        .with_child(current_names)
                        .with_flex_child(sum, 1.0);

                    let this_group = Flex::column()
                        .cross_axis_alignment(CrossAxisAlignment::Start)
                        .with_default_spacer()
                        .with_child(heading)
                        .with_spacer(3.0)
                        .with_child(
                            Seperator::new()
                                .with_color(Color::Rgba32(0x5c6773ff))
                                .with_size(1.0),
                        )
                        .with_default_spacer()
                        .with_child(items)
                        .boxed();
                    current_names = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);
                    sum = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);
                    all_names.add_child(this_group);
                } else {
                    is_first = false;
                }
                prev_kind = item.kind;
            }
            let color = item_kind_str(item).1;
            let id = item.id.clone();
            let mut name = Label::new(item.name.as_ref().unwrap_or(&"_".into()).clone())
                .with_font(theme::CODE_FONT)
                .with_text_color(color)
                .on_click(move |ctx, _data, _env| {
                    ctx.submit_command(GOTO_ITEM.with((id.clone())).to(Target::Global));
                })
                .fix_height(22.0)
                .boxed();

            let docs = item
                .docs
                .as_deref()
                .map(|x| x.split("\n\n").next().unwrap())
                .unwrap_or("");
            let docs = markdown_to_text(docs);
            let docs = RawLabel::new()
                .with_line_break_mode(LineBreaking::Clip)
                .with_text_size(16.0)
                .padding((10.0, 2.))
                .fix_height(22.0)
                .lens(Constant(docs));
            sum.add_child(docs);
            sum.add_spacer(2.0);

            current_names.add_child(name);
            current_names.add_spacer(2.0);
        }
        all_names.boxed()
    });
    Flex::column()
        .with_child(name)
        .with_default_spacer()
        .with_child(
            Seperator::new()
                .with_color(Color::Rgba32(0x5c6773ff))
                .with_size(1.0)
                .with_stroke_style(StrokeStyle::new().dash(vec![2.0], 0.0)),
        )
        .with_default_spacer()
        .with_flex_child(
            Scroll::new(
                Flex::column()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .with_child(docs)
                    .with_child(items.expand_width()),
            )
            .vertical(),
            1.0,
        )
        .cross_axis_alignment(CrossAxisAlignment::Start)
}
