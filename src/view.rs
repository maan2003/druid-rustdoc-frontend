use druid::{
    lens::{Constant, Unit},
    piet::StrokeStyle,
    text::{Attribute, RichText, RichTextBuilder},
    widget::{CrossAxisAlignment, Flex, Label, LineBreaking, RawLabel, Scroll, ViewSwitcher},
    Color, FontFamily, FontWeight, Key, Target, Widget, WidgetExt,
};
use rustdoc_types::{
    Function, GenericBound, GenericParamDef, GenericParamDefKind, Generics, Impl, Item, ItemEnum,
    ItemKind, ItemSummary, Method, Module, Struct, TraitBoundModifier, Type, Union,
};

use crate::{
    format::{format_fn, format_generics_def, format_ty},
    lens::IdLens,
    md::markdown_to_text,
    theme, title_bar,
    widgets::{ItemsWidget, Seperator},
    AppData, GOTO_ITEM,
};
mod mod_;
mod r#fn;
pub fn ui_builder() -> impl Widget<AppData> {
    #[cfg(target_os = "windows")]
    {
        Flex::column()
            .with_child(title_bar::title_bar().lens(Unit))
            .with_flex_child(item().padding(10.), 1.)
            .lens(IdLens)
    }

    #[cfg(not(target_os = "windows"))]
    {
        item().padding(10.).lens(IdLens)
    }
}

fn item() -> impl Widget<(Item, Option<ItemSummary>)> {
    ViewSwitcher::new(
        |data: &(Item, Option<ItemSummary>), _env| data.clone(),
        |_, (item, summary), _env| {
            let time = std::time::Instant::now();
            let x = match item.kind {
                ItemKind::Module => mod_::module(item, summary.as_ref()).lens(Unit).boxed(),
                ItemKind::Struct => struct_view(item, summary.as_ref()).lens(Unit).boxed(),
                ItemKind::Function => function(item).lens(Unit).boxed(),
                _ => panic!("unknown {:?}", item.kind),
                // ItemKind::ExternCrate => {}
                // ItemKind::Import => {}
                // ItemKind::StructField => {}
                // ItemKind::Union => {}
                // ItemKind::Enum => {}
                // ItemKind::Variant => {}
                // ItemKind::Function => {}
                // ItemKind::Typedef => {}
                // ItemKind::OpaqueTy => {}
                // ItemKind::Constant => {}
                // ItemKind::Trait => {}
                // ItemKind::TraitAlias => {}
                // ItemKind::Method => {}
                // ItemKind::Impl => {}
                // ItemKind::Static => {}
                // ItemKind::ForeignType => {}
                // ItemKind::Macro => {}
                // ItemKind::ProcAttribute => {}
                // ItemKind::ProcDerive => {}
                // ItemKind::AssocConst => {}
                // ItemKind::AssocType => {}
                // ItemKind::Primitive => {}
                // ItemKind::Keyword => {}
            };
            dbg!(time.elapsed());
            x
        },
    )
}

fn struct_view(item: &Item, summary: Option<&ItemSummary>) -> impl Widget<()> {
    fn struct_(item: &Item) -> &Union {
        match &item.inner {
            ItemEnum::UnionItem(s) => s,
            _ => panic!("{:?}", item.inner),
        }
    }

    fn field(item: &Item) -> &Type {
        match &item.inner {
            ItemEnum::StructFieldItem(s) => s,
            _ => unreachable!(),
        }
    }

    fn impl_(item: &Item) -> &Impl {
        match &item.inner {
            ItemEnum::ImplItem(s) => s,
            _ => unreachable!(),
        }
    }

    let (type_, color) = item_kind_str(item);

    let st = struct_(item);

    let mut name = RichTextBuilder::new();
    name.push("Struct ");
    for modu in summary
        .into_iter()
        .flat_map(|x| x.path.iter().take(x.path.len() - 1))
    {
        name.push(modu).text_color(theme::MOD_COLOR);
        name.push("::");
    }
    name.push(item.name.as_ref().unwrap())
        .text_color(theme::STRUCT_COLOR);

    if !st.generics.params.is_empty() {
        name.push("<");
        format_generics_def(&st.generics.params, &mut name);
        name.push(">");
    }

    let name = name
        .build()
        .with_attribute(.., Attribute::Weight(FontWeight::MEDIUM))
        .with_attribute(
            ..,
            Attribute::font_family(FontFamily::new_unchecked("Fira Code")),
        );

    let name = RawLabel::new().with_text_size(22.5).lens(Constant(name));

    let mut body = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);

    if let Some(docs) = &item.docs {
        let docs = markdown_to_text(docs);
        let docs = RawLabel::new()
            .with_line_break_mode(LineBreaking::WordWrap)
            .lens(Constant(docs));
        body.add_child(docs);
        body.add_default_spacer();
    }

    // fields
    if !st.fields.is_empty() {
        let fields = ItemsWidget::new(
            st.fields.iter().cloned().collect(),
            |mut field_items, _env| {
                let mut fields = Flex::column()
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .with_child(Label::new("Fields").with_text_size(22.4))
                    .with_spacer(5.0)
                    .with_child(Seperator::new().with_size(1.0))
                    .with_spacer(5.0);

                for (f, summary) in field_items {
                    let mut r = RichTextBuilder::new();
                    r.push(f.name.as_ref().unwrap());
                    r.push(": ");
                    format_ty(field(&f), &mut r);

                    fields.add_child(
                        RawLabel::new()
                            .with_font(theme::CODE_FONT)
                            .lens(Constant(r.build()))
                            .padding((5., 0., 0., 0.)),
                    );

                    if let Some(docs) = &f.docs {
                        let docs = markdown_to_text(docs);
                        let docs = RawLabel::new()
                            .with_line_break_mode(LineBreaking::WordWrap)
                            .padding((10., 0., 0., 0.))
                            .lens(Constant(docs));
                        fields.add_child(docs);
                        fields.add_default_spacer();
                    } else {
                        fields.add_spacer(5.0);
                    }
                }
                fields.boxed()
            },
        );

        // impls
        let impls = ItemsWidget::new(st.impls.iter().cloned().collect(), |mut impl_items, _| {
            let mut impls = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);
            #[derive(Ord, PartialOrd, Eq, PartialEq)]
            enum ImplType {
                Simple,
                Trait,
                Auto,
                Blanket,
            }
            let classify = |a: &Impl| {
                if dbg!(&a.trait_).is_none() {
                    ImplType::Simple
                } else if a.synthetic {
                    ImplType::Auto
                } else if a.blanket_impl.is_some() {
                    dbg!("blanket");
                    ImplType::Blanket
                } else {
                    ImplType::Trait
                }
            };
            impl_items.sort_by(|(a, _), (b, _)| {
                let ai = impl_(&a);
                let bi = impl_(&b);
                classify(ai).cmp(&classify(bi))
            });
            let mut start = ImplType::Blanket;
            for (item, summary) in impl_items {
                let i = impl_(&item);
                let kind = classify(i);
                if kind != start {
                    let heading = match kind {
                        ImplType::Simple => "Implementations",
                        ImplType::Trait => "Trait Implementations",
                        ImplType::Auto => "Auto Trait Implementations",
                        ImplType::Blanket => "Blanket Implementations",
                    };
                    impls.add_child(Label::new(heading).with_text_size(22.4));
                    impls.add_spacer(3.0);
                    impls.add_child(Seperator::new().with_size(1.0));
                    impls.add_default_spacer();
                    start = kind;
                }
                impls.add_child(impl_block(&item).padding((0., 0.0, 0.0, 0.0)));
            }
            impls.boxed()
        });

        body.add_child(fields);
        body.add_child(impls);
    }

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
        .with_flex_child(Scroll::new(body).vertical(), 1.0)
}

fn impl_block(item: &Item) -> impl Widget<()> {
    fn impl_(item: &Item) -> &Impl {
        match &item.inner {
            ItemEnum::ImplItem(s) => s,
            _ => unreachable!(),
        }
    }

    let i = impl_(item);
    let mut r = RichTextBuilder::new();
    r.push("impl ");
    if let Some(tr) = &i.trait_ {
        format_ty(tr, &mut r);
        r.push(" for ");
    }
    format_ty(&i.for_, &mut r);

    let name = RawLabel::new()
        .with_font(theme::CODE_FONT)
        .lens(Constant(r.build()));

    let items = ItemsWidget::new(i.items.iter().cloned().collect(), |items, _| {
        let mut flex = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);
        for (item, summary) in items {
            let mut r = RichTextBuilder::new();
            match &item.inner {
                ItemEnum::FunctionItem(m) => {
                    format_fn(&item, &mut r);
                }
                ItemEnum::AssocConstItem { type_, default } => {
                    r.push("pub const ");
                    r.push(&item.name.as_ref().unwrap())
                        .text_color(theme::CONST_COLOR);

                    if let Some(def) = default {
                        r.push(" = ");
                        r.push(def);
                    }
                }
                ItemEnum::AssocTypeItem { bounds, default } => {
                    r.push("pub type ");
                    r.push(&item.name.as_ref().unwrap())
                        .text_color(theme::TYPE_COLOR);
                }
                ItemEnum::TypedefItem(t) => {
                    r.push("pub type ");
                    r.push(&item.name.as_ref().unwrap())
                        .text_color(theme::TYPE_COLOR);
                    r.push(" = ");
                    format_ty(&t.type_, &mut r);
                }
                _ => {
                    eprintln!("Unknown thing {:?}", item)
                }
            };

            flex.add_child(
                RawLabel::new()
                    .with_font(theme::CODE_FONT)
                    .lens(Constant(r.build())),
            );
            if let Some(docs) = &item.docs {
                let docs = markdown_to_text(&docs);
                let docs = RawLabel::new()
                    .with_line_break_mode(LineBreaking::WordWrap)
                    .padding((20., 0., 0., 0.))
                    .lens(Constant(docs));

                flex.add_child(docs);
                flex.add_spacer(10.);
            }
            flex.add_spacer(20.);
        }
        flex.boxed()
    });
    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(name)
        .with_spacer(5.)
        .with_child(items.padding((10., 0., 0., 0.)))
}

fn function(item: &Item) -> impl Widget<()> {
    fn func(item: &Item) -> &Function {
        match &item.inner {
            ItemEnum::FunctionItem(f) => f,
            _ => unreachable!(),
        }
    }
    let (type_, color) = item_kind_str(item);

    let mut name = RichTextBuilder::new();
    name.push("fn ");
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
        .with_flex_child(Scroll::new(docs).vertical(), 1.)
}


fn item_kind_str(i: &Item) -> (&'static str, Key<Color>) {
    match &i.inner {
        ItemEnum::ModuleItem(m) if m.is_crate => ("crate", theme::MOD_COLOR),
        ItemEnum::ModuleItem(_) => ("mod", theme::MOD_COLOR),
        // ItemEnum::ImportItem(_) => {}
        ItemEnum::StructItem(_) | ItemEnum::UnionItem(_)=> ("Struct", theme::STRUCT_COLOR),
        // ItemEnum::StructFieldItem(_) => {}
        ItemEnum::EnumItem(_) => ("Enum", theme::ENUM_COLOR),
        ItemEnum::FunctionItem(_) => ("Function", theme::FUNCTION_COLOR),
        ItemEnum::TraitItem(_) => ("Trait", theme::TRAIT_COLOR),
        // ItemEnum::VariantItem(_) => {}
        // ItemEnum::FunctionItem(_) => {}
        // ItemEnum::TraitAliasItem(_) => {}
        // ItemEnum::MethodItem(_) => {}
        // ItemEnum::ImplItem(_) => {}
        // ItemEnum::TypedefItem(_) => {}
        // ItemEnum::OpaqueTyItem(_) => {}
        ItemEnum::ConstantItem(_) => ("const", theme::FUNCTION_COLOR),
        // ItemEnum::StaticItem(_) => {}
        // ItemEnum::ForeignTypeItem => {}
        // ItemEnum::MacroItem(_) => {}
        // ItemEnum::ProcMacroItem(_) => {}
        _ => ("Unknown Item", theme::MOD_COLOR),
        // ItemEnum::ExternCrateItem { name, rename } => {}
        // ItemEnum::AssocConstItem { type_, default } => {}
        // ItemEnum::AssocTypeItem { bounds, default } => {}
    }
}
