use druid::im::Vector;
use druid::text::RichTextBuilder;
use rustdoc_types::{
    FnDecl, GenericArg, GenericArgs, GenericBound, GenericParamDef, GenericParamDefKind, Generics,
    Qualifiers, TraitBoundModifier, Type, TypeBindingKind, WherePredicate,
};

use crate::theme;

pub fn format_ty(ty: &Type, hint_trait: bool, r: &mut RichTextBuilder) {
    match ty {
        Type::ResolvedPath {
            name,
            id,
            args,
            param_names: _,
        } => {
            let name = name.rsplit("::").next().unwrap();
            r.push(name)
                .text_color(if hint_trait {
                    theme::TRAIT_COLOR
                } else {
                    theme::STRUCT_COLOR
                })
                .link(super::GOTO_ITEM.with(id.clone()));

            if let Some(args) = args {
                format_generic_args(args, r);
            }
        }
        Type::Generic(g) => {
            r.push(g).text_color(theme::TYPE_COLOR);
        }
        Type::Primitive(p) => {
            r.push(p).text_color(theme::PRIMITIVE_COLOR);
        }
        Type::FunctionPointer(f) => {
            r.push("(");
            format_seperated(f.decl.inputs.iter(), ", ", r, |(name, ty), r| {
                r.push(name);
                r.push(": ");
                format_ty(ty, false, r);
            });
            r.push(")");
            if let Some(ty) = &f.decl.output {
                r.push(" -> ");
                format_ty(ty, false, r);
            }
        }
        Type::Tuple(t) => {
            r.push("(");
            format_seperated(t.iter(), ", ", r, |ty, r| format_ty(ty, false, r));
            r.push(")");
        }
        Type::Slice(s) => {
            r.push("&[");
            format_ty(&s, false, r);
            r.push("]");
        }
        Type::Array { type_, len } => {
            r.push("[");
            format_ty(type_, false, r);
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
            format_ty(type_, false, r);
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
            format_ty(type_, false, r);
        }
        Type::QualifiedPath {
            name,
            self_type,
            trait_,
        } => {
            match trait_ as &Type {
                Type::ResolvedPath { name, .. } if name == "" => {
                    format_ty(self_type, false, r);
                    r.push("::");
                }
                _ => {
                    r.push("<");
                    format_ty(self_type, false, r);
                    r.push(" as ");
                    format_ty(trait_, true, r);
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
    no_bounds: bool,
    r: &'a mut RichTextBuilder,
) {
    format_seperated(g.into_iter(), ", ", r, |g, r| match &g.kind {
        GenericParamDefKind::Lifetime => {
            r.push(&g.name);
        }
        GenericParamDefKind::Type { bounds, default } => {
            r.push(&g.name).text_color(theme::TYPE_COLOR);
            if !bounds.is_empty() && !no_bounds {
                r.push(": ");
                format_generic_bound(bounds, r);
            }
            if let Some(def) = default {
                r.push(" = ");
                format_ty(def, false, r);
            }
        }
        GenericParamDefKind::Const(c) => {
            r.push("const ");
            r.push(&g.name).text_color(theme::TYPE_COLOR);
            r.push(": ");
            format_ty(c, false, r);
        }
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
                    format_generics_def(generic_params, false, r);
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
                format_ty(trait_, true, r);
            }
            GenericBound::Outlives(a) => {
                r.push(a);
            }
        };
    });
}

pub fn format_generic_args(g: &GenericArgs, r: &mut RichTextBuilder) {
    match g {
        GenericArgs::AngleBracketed { args, bindings }
            if !args.is_empty() || !bindings.is_empty() =>
        {
            r.push("<");
            format_seperated(args.iter(), ", ", r, |a, r| match a {
                GenericArg::Lifetime(lf) => {
                    r.push(lf);
                }
                GenericArg::Type(ty) => {
                    format_ty(ty, false, r);
                }
                GenericArg::Const(c) => {
                    r.push("{");
                    r.push(c.value.as_ref().unwrap());
                    r.push("}");
                }
            });
            if !args.is_empty() && !bindings.is_empty() {
                r.push(", ");
            }
            format_seperated(bindings.iter(), ", ", r, |a, r| {
                r.push(&a.name).text_color(theme::TYPE_COLOR);
                match &a.binding {
                    TypeBindingKind::Equality(ty) => {
                        r.push(" = ");
                        format_ty(ty, false, r);
                    }
                    TypeBindingKind::Constraint(c) => {
                        r.push(": ");
                        format_generic_bound(c, r);
                    }
                }
            });
            r.push(">");
        }
        GenericArgs::Parenthesized { inputs, output } => {
            r.push("(");
            format_seperated(inputs.iter(), ", ", r, |ty, r| format_ty(ty, false, r));
            r.push(")");
            if let Some(ty) = output {
                r.push(" -> ");
                format_ty(ty, false, r);
            }
        }
        _ => {}
    }
}

pub fn format_wheres<'a>(
    others: impl IntoIterator<Item = &'a GenericParamDef>,
    wheres: impl IntoIterator<Item = &'a WherePredicate>,
    r: &mut RichTextBuilder,
) {
    r.push("where");
    for i in others {
        match &i.kind {
            GenericParamDefKind::Type { bounds, default: _ } if !bounds.is_empty() => {
                r.push("\n    ");
                r.push(&i.name).text_color(theme::TYPE_COLOR);
                r.push(": ");
                format_generic_bound(bounds, r);
                r.push(",");
            }
            _ => {}
        }
    }
    for i in wheres {
        match i {
            WherePredicate::BoundPredicate { ty, bounds } => {
                r.push("\n    ");
                format_ty(ty, false, r);
                r.push(": ");
                format_generic_bound(bounds, r);
                r.push(",");
            }
            WherePredicate::RegionPredicate { lifetime, bounds } => {
                r.push("\n    ");
                r.push(lifetime);
                r.push(": ");
                format_generic_bound(bounds, r);
                r.push(", ");
            }
            WherePredicate::EqPredicate { lhs: _, rhs: _ } => {}
        }
    }
}
pub fn format_fn(
    name: &str,
    header: &Vector<Qualifiers>,
    gens: &Generics,
    decl: &FnDecl,
    r: &mut RichTextBuilder,
) {
    r.push("pub fn ");
    for h in header {
        match h {
            Qualifiers::Const => {
                r.push("const ");
            }
            Qualifiers::Unsafe => {
                r.push("unsafe ");
            }
            Qualifiers::Async => {
                r.push("async ");
            }
            _ => {}
        };
    }

    r.push(name).text_color(theme::FN_COLOR);

    if !gens.params.iter().all(|x| x.name.starts_with("impl ")) {
        r.push("<");
        format_generics_def(
            gens.params.iter().filter(|x| !x.name.starts_with("impl ")),
            false,
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
                match ty {
                    Type::BorrowedRef {
                        lifetime,
                        mutable,
                        type_: _,
                    } => {
                        r.push("&");
                        if let Some(lf) = lifetime {
                            r.push(lf);
                            r.push(" ");
                        }
                        if *mutable {
                            r.push("mut ");
                        }
                        r.push("self");
                        continue;
                    }
                    Type::Generic(s) if s == "Self" => {
                        r.push("self");
                        continue;
                    }
                    _ => {}
                }
            }
        }

        r.push(name);
        r.push(": ");
        format_ty(ty, false, r);
    }
    r.push(")");
    if let Some(ty) = &decl.output {
        r.push(" -> ");
        format_ty(ty, false, r);
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
