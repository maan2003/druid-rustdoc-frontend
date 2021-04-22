use druid::im::Vector;
use druid::text::RichTextBuilder;
use druid::widget::{CrossAxisAlignment, Flex, Label, List, RawLabel};
use druid::{lens, Color, Data, Key, Widget, WidgetExt};
use druid_simple_table::Table;
use rustdoc_types::GenericParamDefKind;

use crate::data;
use crate::format::{format_fn, format_generics_def, format_seperated, format_ty, format_wheres};
use crate::widgets::*;
use crate::{theme, GOTO_ITEM};

pub fn ui_builder() -> impl Widget<data::Screen> {
    data::Screen::matcher()
        .mod_(mod_())
        .struct_(struct_())
        .enum_(enum_())
        .fn_(fn_())
}

fn mod_() -> impl Widget<data::Mod> {
    let name = RawLabel::code()
        .with_text_size(24.)
        .computed(|m: &data::Mod| {
            let mut r = RichTextBuilder::new();
            r.push("mod ");
            for i in &m.item.parents {
                r.push(&i).text_color(theme::MOD_COLOR);
                r.push("::");
            }
            r.push(&m.item.name).text_color(theme::MOD_COLOR);
            r.build()
        });

    let docs = RawLabel::new()
        .wrap_text()
        .padding((0., 0., 0., 10.))
        .or_empty()
        .lens(lens!(data::Mod, item.doc));

    let mods = item_list("Modules", theme::MOD_COLOR).lens(lens!(data::Mod, mods));
    let structs = item_list("Structs", theme::STRUCT_COLOR).lens(lens!(data::Mod, structs));
    let enums = item_list("Enums", theme::ENUM_COLOR).lens(lens!(data::Mod, enums));
    let fns = item_list("Functions", theme::FN_COLOR).lens(lens!(data::Mod, fns));

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(name)
        .with_default_spacer()
        .with_child(docs)
        .with_default_spacer()
        .with_child(mods)
        .with_child(structs)
        .with_child(enums)
        .with_child(fns)
        .padding(10.)
        .scroll()
        .vertical()
}

fn item_list(head: &'static str, color: Key<Color>) -> impl Widget<Vector<data::Item>> {
    let mods = Table::new()
        .seperator(0., 0.)
        .col(move || {
            RawLabel::code()
                .color(color)
                .lens(lens!(data::Item, name))
                .on_click(|ctx, it, _env| {
                    ctx.submit_command(GOTO_ITEM.with(it.id.clone()));
                })
        })
        .col(|| {
            RawLabel::code()
                .padding((20., 0., 0., 0.))
                .or_empty()
                .lens(lens!(data::Item, short_doc))
        });

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(h2(head))
        .seperator(2)
        .with_child(mods)
        .with_spacer(20.)
        .empty_if(|m, _| m.is_empty())
}

fn h2<T: Data>(txt: &str) -> impl Widget<T> {
    Label::new(txt).with_text_size(21.)
}

fn struct_() -> impl Widget<data::Struct> {
    let name = RawLabel::code()
        .with_text_size(24.)
        .computed(|t: &data::Struct| {
            let mut r = RichTextBuilder::new();
            r.push("struct ");
            for i in &t.item.parents {
                r.push(&i).text_color(theme::MOD_COLOR);
                r.push("::");
            }
            r.push(&t.item.name).text_color(theme::STRUCT_COLOR);
            if !t.generics.params.is_empty() {
                r.push("<");
                format_generics_def(&t.generics.params, true, &mut r);
                r.push(">");
            }
            r.build()
        });

    let generics = RawLabel::code()
        .computed(|t: &data::Struct| {
            let mut r = RichTextBuilder::new();
            format_wheres(&t.generics.params, &t.generics.where_predicates, &mut r);
            r.build()
        })
        .empty_if(|t, _| {
            t.generics.where_predicates.is_empty()
                && !t.generics.params.iter().any(|i| {
                    matches!( &i.kind,
                               GenericParamDefKind::Type { bounds, .. } if !bounds.is_empty())
                })
        });

    let docs = RawLabel::new()
        .wrap_text()
        .or_empty()
        .lens(lens!(data::Struct, item.doc));

    let feilds = List::new(|| {
        let docs = RawLabel::new()
            .wrap_text()
            .padding((10., 5., 0., 10.))
            .or_empty()
            .lens(lens!(data::Field, item.doc));

        Flex::column()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(RawLabel::code().computed(|t: &data::Field| {
                let mut r = RichTextBuilder::new();
                r.push(&t.item.name);
                r.push(": ");
                format_ty(&t.ty, false, &mut r);
                r.build()
            }))
            .with_child(docs)
    });

    let fields = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(h2("Fields"))
        .seperator(2)
        .with_child(feilds)
        .with_default_spacer()
        .empty_if(|f: &Vector<_>, _| f.is_empty())
        .lens(lens!(data::Struct, fields));

    let impls = List::new(impl_).with_spacing(20.);
    let impls = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(h2("Implementations"))
        .seperator(2)
        .with_child(impls)
        .with_spacer(20.)
        .empty_if(|i: &Vector<_>, _| i.is_empty())
        .lens(lens!(data::Struct, impls));

    let trait_impls = List::new(impl_).with_spacing(20.);
    let trait_impls = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(h2("Trait Implementations"))
        .seperator(2)
        .with_child(trait_impls)
        .with_spacer(20.)
        .empty_if(|i: &Vector<_>, _| i.is_empty())
        .lens(lens!(data::Struct, trait_impls));

    let auto_impls = List::new(impl_).with_spacing(20.);
    let auto_impls = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(h2("Auto Implementations"))
        .seperator(2)
        .with_child(auto_impls)
        .with_spacer(20.)
        .empty_if(|i: &Vector<_>, _| i.is_empty())
        .lens(lens!(data::Struct, auto_impls));
    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(name)
        .with_child(generics)
        .with_default_spacer()
        .with_child(docs)
        .with_spacer(20.)
        .with_child(fields)
        .with_child(impls)
        .with_child(trait_impls)
        .with_default_spacer()
        .with_default_spacer()
        .with_child(auto_impls)
        .padding(10.)
        .scroll()
        .vertical()
}

fn enum_() -> impl Widget<data::Enum> {
    let name = RawLabel::code()
        .with_text_size(24.)
        .computed(|t: &data::Enum| {
            let mut r = RichTextBuilder::new();
            r.push("enum ");
            for i in &t.item.parents {
                r.push(&i).text_color(theme::MOD_COLOR);
                r.push("::");
            }
            r.push(&t.item.name).text_color(theme::STRUCT_COLOR);
            if !t.generics.params.is_empty() {
                r.push("<");
                format_generics_def(&t.generics.params, true, &mut r);
                r.push(">");
            }
            r.build()
        });

    let generics = RawLabel::code()
        .computed(|t: &data::Enum| {
            let mut r = RichTextBuilder::new();
            format_wheres(&t.generics.params, &t.generics.where_predicates, &mut r);
            r.build()
        })
        .empty_if(|t, _| {
            t.generics.where_predicates.is_empty()
                && !t.generics.params.iter().any(|i| {
                    matches!( &i.kind,
                               GenericParamDefKind::Type { bounds, .. } if !bounds.is_empty())
                })
        });

    let variants = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(h2("Variants"))
        .seperator(2)
        .with_child(List::new(variant))
        .with_default_spacer()
        .empty_if(|f: &Vector<_>, _| f.is_empty())
        .lens(lens!(data::Enum, variants));

    let docs = RawLabel::new()
        .wrap_text()
        .or_empty()
        .lens(lens!(data::Enum, item.doc));

    let impls = List::new(impl_).with_spacing(20.);
    let impls = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(h2("Implementations"))
        .seperator(2)
        .with_child(impls)
        .with_spacer(20.)
        .empty_if(|i: &Vector<_>, _| i.is_empty())
        .lens(lens!(data::Enum, impls));

    let trait_impls = List::new(impl_).with_spacing(20.);
    let trait_impls = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(h2("Trait Implementations"))
        .seperator(2)
        .with_child(trait_impls)
        .with_spacer(20.)
        .empty_if(|i: &Vector<_>, _| i.is_empty())
        .lens(lens!(data::Enum, trait_impls));

    let auto_impls = List::new(impl_).with_spacing(20.);
    let auto_impls = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(h2("Auto Implementations"))
        .seperator(2)
        .with_child(auto_impls)
        .with_spacer(20.)
        .empty_if(|i: &Vector<_>, _| i.is_empty())
        .lens(lens!(data::Enum, auto_impls));
    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(name)
        .with_child(generics)
        .with_default_spacer()
        .with_child(docs)
        .with_default_spacer()
        .with_child(variants)
        .with_child(impls)
        .with_child(trait_impls)
        .with_default_spacer()
        .with_default_spacer()
        .with_child(auto_impls)
        .padding(10.)
        .scroll()
        .vertical()
}

fn variant() -> impl Widget<data::Variant> {
    let docs = RawLabel::new()
        .wrap_text()
        .padding((10., 5., 0., 10.))
        .or_empty()
        .lens(lens!(data::Variant, item.doc));

    let label = RawLabel::code().computed(|v: &data::Variant| {
        let mut r = RichTextBuilder::new();
        r.push(&v.item.name).text_color(theme::ENUM_COLOR);
        match &v.inner {
            data::VariantInner::Plain => {}
            data::VariantInner::Tuple(ts) => {
                r.push("(");
                format_seperated(ts.iter(), ", ", &mut r, |ty, r| {
                    format_ty(ty, false, r);
                });
                r.push(")");
            }
            data::VariantInner::Struct(fs) => {
                if !fs.is_empty() {
                    r.push(" {");
                    r.push("\n");
                } else {
                    r.push("{ ");
                }
                for f in fs.iter() {
                    r.push("    ");
                    r.push(&f.item.name);
                    r.push(": ");
                    format_ty(&f.ty, false, &mut r);
                    r.push(",");
                    r.push("\n");
                }
                r.push("}");
            }
        }
        r.build()
    });
    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(label)
        .with_child(docs)
}

fn impl_() -> impl Widget<data::Impl> {
    let head = RawLabel::code().computed(|i: &data::Impl| {
        let mut r = RichTextBuilder::new();
        r.push("impl");
        if !i.generics.params.is_empty() {
            r.push("<");
            format_generics_def(&i.generics.params, true, &mut r);
            r.push(">");
        }

        r.push(" ");
        if let Some(tr) = &i.trait_ {
            format_ty(tr, true, &mut r);
            r.push(" for ");
        }
        format_ty(&i.for_, false, &mut r);
        if !i.generics.where_predicates.is_empty()
            || i.generics.params.iter().any(|i| {
                matches!( &i.kind,
                GenericParamDefKind::Type { bounds, .. } if !bounds.is_empty())
            })
        {
            r.push("\n");
            format_wheres(&i.generics.params, &i.generics.where_predicates, &mut r);
        }
        r.build()
    });
    let items = List::new(|| data::ImplItem::matcher().fn_(impl_fn()))
        .with_spacing(10.)
        .padding((0., 10., 0., 0.))
        .empty_if(|d: &Vector<_>, _| d.is_empty())
        .lens(lens!(data::Impl, items));

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(head)
        .with_child(items)
}

fn impl_fn() -> impl Widget<data::Fn> {
    let docs = RawLabel::new()
        .wrap_text()
        .padding((10., 10., 0., 10.))
        .or_empty()
        .lens(lens!(data::Fn, item.doc));

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(RawLabel::code().computed(|f: &data::Fn| {
            let mut r = RichTextBuilder::new();
            format_fn(&f.item.name, &f.header, &f.generics, &f.decl, &mut r);
            r.build()
        }))
        .with_child(docs)
        .padding((20., 0., 0., 0.))
}

fn fn_() -> impl Widget<data::Fn> {
    let name = RawLabel::code()
        .with_text_size(21.)
        .computed(|f: &data::Fn| {
            let mut r = RichTextBuilder::new();
            r.push("fn ");
            for i in &f.item.parents {
                r.push(&i).text_color(theme::MOD_COLOR);
                r.push("::");
            }
            r.push(&f.item.name).text_color(theme::FN_COLOR);

            if !f
                .generics
                .params
                .iter()
                .all(|x| x.name.starts_with("impl "))
            {
                r.push("<");
                format_generics_def(
                    f.generics
                        .params
                        .iter()
                        .filter(|x| !x.name.starts_with("impl ")),
                    true,
                    &mut r,
                );
                r.push(">");
            }

            r.push("(\n");
            for (name, ty) in &f.decl.inputs {
                r.push("    ");
                r.push(&name);
                r.push(": ");
                format_ty(ty, false, &mut r);
                r.push(", ");
                r.push("\n");
            }
            r.push(")");
            if let Some(ty) = &f.decl.output {
                r.push(" -> ");
                format_ty(ty, false, &mut r);
            }

            r.build()
        });

    let generics = RawLabel::code()
        .computed(|t: &data::Fn| {
            let mut r = RichTextBuilder::new();
            format_wheres(&t.generics.params, &t.generics.where_predicates, &mut r);
            r.build()
        })
        .empty_if(|t, _| {
            t.generics.where_predicates.is_empty()
                && !t.generics.params.iter().any(|f| {
                    !f.name.starts_with("impl ")
                        && matches!( &f.kind,
                               GenericParamDefKind::Type { bounds, .. } if !bounds.is_empty())
                })
        });

    let docs = RawLabel::new()
        .wrap_text()
        .or_empty()
        .lens(lens!(data::Fn, item.doc));

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(name)
        .with_child(generics)
        .with_default_spacer()
        .with_child(docs)
        .with_default_spacer()
        .padding(10.)
        .scroll()
}
