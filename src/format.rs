use druid::text::RichTextBuilder;
use rustdoc_types::{
    GenericArg, GenericArgs, GenericBound, GenericParamDef, GenericParamDefKind, Item, ItemEnum,
    TraitBoundModifier, Type,
};

use crate::theme;

pub fn format_ty(ty: &Type, r: &mut RichTextBuilder) {
    match ty {
        Type::ResolvedPath {
            name,
            id,
            args,
            param_names,
        } => {
            r.push(name).text_color(theme::STRUCT_COLOR);
            if let Some(args) = args {
                format_generic_args(args, r);
            }
        }
        Type::Generic(g) => {
            r.push(g).text_color(theme::TYPE_COLOR);
        }
        Type::Primitive(p) => {
            r.push(p).text_color(theme::STRUCT_COLOR);
        }
        Type::FunctionPointer(f) => {
            r.push("(");
            let mut is_first = true;
            format_seperated(f.decl.inputs.iter(), ", ", r, |(name, ty), r| {
                r.push(name);
                r.push(": ");
                format_ty(ty, r);
            });
            r.push(")");
            if let Some(ty) = &f.decl.output {
                r.push(" -> ");
                format_ty(ty, r);
            }
        }
        Type::Tuple(t) => {
            r.push("(");
            format_seperated(t.iter(), ", ", r, format_ty);
            r.push(")");
        }
        Type::Slice(s) => {
            r.push("&[");
            format_ty(&s, r);
            r.push("]");
        }
        Type::Array { type_, len } => {
            r.push("[");
            format_ty(type_, r);
            r.push("; ");
            r.push(len);
            r.push("]");
        }
        Type::Never => {
            r.push("!");
        }
        Type::Infer => {
            r.push("_");
        }
        Type::RawPointer { mutable, type_ } => {
            r.push("*");
            if *mutable {
                r.push("mut ");
            } else {
                r.push("const ");
            }
            format_ty(type_, r);
        }
        Type::BorrowedRef {
            lifetime,
            mutable,
            type_,
        } => {
            r.push("&");
            if let Some(lf) = lifetime {
                r.push(lf);
                r.push(" ");
            }
            if *mutable {
                r.push("mut ");
            }
            format_ty(type_, r);
        }
        Type::QualifiedPath {
            name,
            self_type,
            trait_,
        } => {
            match (trait_ as &Type) {
                Type::ResolvedPath { name, .. } if name == "" => {
                    format_ty(self_type, r);
                    r.push("::");
                }
                _ => {
                    r.push("<");
                    format_ty(self_type, r);
                    r.push(" as ");
                    format_ty(trait_, r);
                    r.push(">::");
                }
            }
            r.push(name).text_color(theme::TYPE_COLOR);
        }
        Type::ImplTrait(tr) => {
            r.push("impl ");
            format_generic_bound(tr, r);
        }
    }
}

pub fn format_generics_def<'a>(
    g: impl IntoIterator<Item = &'a GenericParamDef>,
    r: &'a mut RichTextBuilder,
) {
    format_seperated(g.into_iter(), ", ", r, |g, r| match &g.kind {
        GenericParamDefKind::Lifetime => {
            r.push(&g.name);
        }
        GenericParamDefKind::Type { bounds, default } => {
            r.push(&g.name).text_color(theme::TYPE_COLOR);
            if !bounds.is_empty() {
                r.push(": ");
                format_generic_bound(bounds, r);
            }
            if let Some(def) = default {
                r.push(" = ");
                format_ty(def, r);
            }
        }
        GenericParamDefKind::Const(c) => {}
    });
}

pub fn format_generic_bound<'a>(
    g: impl IntoIterator<Item = &'a GenericBound>,
    r: &'a mut RichTextBuilder,
) {
    format_seperated(g.into_iter(), " + ", r, |g, r| {
        match g {
            GenericBound::TraitBound {
                trait_,
                generic_params,
                modifier,
            } => {
                if !generic_params.is_empty() {
                    r.push("for<");
                    format_generics_def(generic_params, r);
                    r.push("> ");
                }
                match modifier {
                    TraitBoundModifier::None => {}
                    TraitBoundModifier::Maybe => {
                        r.push("?");
                    }
                    TraitBoundModifier::MaybeConst => {
                        r.push("?const ");
                    }
                }
                format_ty(trait_, r);
            }
            GenericBound::Outlives(a) => {
                r.push(a);
            }
        };
    });
}

pub fn format_generic_args(g: &GenericArgs, r: &mut RichTextBuilder) {
    match g {
        GenericArgs::AngleBracketed { args, bindings } if !args.is_empty() => {
            r.push("<");
            format_seperated(args.iter(), ", ", r, |a, r| match a {
                GenericArg::Lifetime(lf) => {
                    r.push(lf);
                }
                GenericArg::Type(ty) => {
                    format_ty(ty, r);
                }
                GenericArg::Const(c) => {
                    r.push("{");
                    r.push(c.value.as_ref().unwrap());
                    r.push("}");
                }
            });
            r.push(">");
        }
        GenericArgs::Parenthesized { inputs, output } => {
            r.push("(");
            format_seperated(inputs.iter(), ", ", r, format_ty);
            r.push(")");
            if let Some(ty) = output {
                r.push(" -> ");
                format_ty(ty, r);
            }
        }
        _ => {}
    }
}

pub fn format_fn(f: &Item, r: &mut RichTextBuilder) {
    r.push("pub fn ");
    let (decl, gens) = match &f.inner {
        ItemEnum::FunctionItem(f) => (&f.decl, &f.generics),
        ItemEnum::MethodItem(m) => (&m.decl, &m.generics),
        _ => unreachable!(),
    };

    r.push(f.name.as_ref().unwrap())
        .text_color(theme::FUNCTION_COLOR);

    if !gens.params.iter().all(|x| x.name.starts_with("impl ")) {
        r.push("<");
        format_generics_def(
            gens.params.iter().filter(|x| !x.name.starts_with("impl ")),
            r,
        );
        r.push(">");
    }

    r.push("(");
    let mut is_first = true;
    for (name, ty) in &decl.inputs {
        if !is_first {
            r.push(", ");
        } else {
            is_first = false;
            if name == "self" {
                r.push(&ty.to_string().replace("Self", "self"));
                continue;
            }
        }

        r.push(name);
        r.push(": ");
        format_ty(ty, r);
    }
    r.push(")");
    if let Some(ty) = &decl.output {
        r.push(" -> ");
        format_ty(ty, r);
    }
}

pub fn format_seperated<'a, T>(
    items: impl Iterator<Item = T>,
    sep: &'a str,
    r: &'a mut RichTextBuilder,
    mut func: impl FnMut(T, &mut RichTextBuilder),
) {
    let mut is_first = true;
    items.for_each(move |item| {
        if !is_first {
            r.push(sep);
        } else {
            is_first = false;
        }
        func(item, r);
    })
}
