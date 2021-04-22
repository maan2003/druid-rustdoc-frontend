use std::fs;
use std::path::Path;


use druid::{AppDelegate, Selector};
use rdoc::{ItemEnum, ItemKind};
use rustdoc_types as rdoc;
pub const OPEN_LINK: Selector<String> = Selector::new("druid-rustdoc.open-link");

use crate::md::markdown_to_text;
use crate::{data, GOTO_ITEM};

pub struct Delegate {
    krate: rdoc::Crate,
    current: rdoc::Id,
}

impl Delegate {
    pub fn new(path: &Path) -> Self {
        let data = fs::read_to_string(path).unwrap();
        let krate = rustdoc_types::parse(&data);
        Self {
            current: krate.root.clone(),
            krate,
        }
    }

    pub fn data(&self) -> data::Screen {
        let item = &self.krate.index[&self.current];
        let s = self.krate.paths.get(&self.current);
        match &item.inner {
            ItemEnum::ModuleItem(m) => {
                let item = item_to_data(&item, s);
                let item_of_kind = |k| {
                    m.items
                        .iter()
                        .map(|id| (&self.krate.index[id], self.krate.paths.get(id)))
                        .filter(|(i, _)| i.kind == k)
                        .map(|(i, sum)| item_to_data(i, sum))
                        .collect()
                };
                let mod_ = data::Mod {
                    item,
                    structs: item_of_kind(ItemKind::Struct),
                    enums: item_of_kind(ItemKind::Enum),
                    traits: item_of_kind(ItemKind::Trait),
                    mods: item_of_kind(ItemKind::Module),
                    fns: item_of_kind(ItemKind::Function),
                };
                data::Screen::Mod(mod_)
            }
            ItemEnum::StructItem(_) => {
                let struct_ = self.item_to_struct(item, s);
                data::Screen::Struct(struct_)
            }
            ItemEnum::EnumItem(e) => {
                let item = item_to_data(&item, s);
                let enum_ = data::Enum {
                    item,
                    generics: e.generics.clone(),
                    variants: e
                        .variants
                        .iter()
                        .map(|id| (&self.krate.index[id], self.krate.paths.get(id)))
                        .map(|(item, s)| self.item_to_variant(item, s))
                        .collect(),
                    impls: e
                        .impls
                        .iter()
                        .map(|id| self.item_to_impl(id))
                        .filter(|i| i.trait_.is_none())
                        .collect(),
                    trait_impls: e
                        .impls
                        .iter()
                        .map(|id| self.item_to_impl(id))
                        .filter(|i| i.trait_.is_some() && !i.synthetic)
                        .collect(),
                    auto_impls: e
                        .impls
                        .iter()
                        .map(|id| self.item_to_impl(id))
                        .filter(|i| i.synthetic)
                        .collect(),
                };
                data::Screen::Enum(enum_)
            }
            ItemEnum::FunctionItem(f) => {
                let f = item_to_fn(item, s, f);
                data::Screen::Fn(f)
            }
            _ => todo!("handle other Items"),
        }
    }

    fn item_to_variant(&self, item: &rdoc::Item, s: Option<&rdoc::ItemSummary>) -> data::Variant {
        match &item.inner {
            ItemEnum::VariantItem(v) => data::Variant {
                item: item_to_data(item, s),
                inner: match v {
                    rdoc::Variant::Plain => data::VariantInner::Plain,
                    rdoc::Variant::Tuple(v) => data::VariantInner::Tuple(v.clone()),
                    rdoc::Variant::Struct(i) => data::VariantInner::Struct(
                        i.iter()
                            .map(|id| {
                                let item = &self.krate.index[id];
                                let s = &self.krate.paths[id];
                                let ty = match &item.inner {
                                    ItemEnum::StructFieldItem(ty) => ty.clone(),
                                    _ => unreachable!(),
                                };
                                let item = item_to_data(item, Some(s));
                                data::Field { item, ty }
                            })
                            .collect(),
                    ),
                },
            },
            _ => unreachable!(),
        }
    }

    fn item_to_struct(&self, item: &rdoc::Item, sum: Option<&rdoc::ItemSummary>) -> data::Struct {
        let s = match &item.inner {
            ItemEnum::StructItem(s) => s,
            _ => unreachable!(),
        };
        let item = item_to_data(&item, sum);
        let struct_ = data::Struct {
            item,
            generics: s.generics.clone(),
            fields: s
                .fields
                .iter()
                .map(|id| {
                    let item = &self.krate.index[id];
                    let s = self.krate.paths.get(id);
                    let ty = match &item.inner {
                        ItemEnum::StructFieldItem(ty) => ty.clone(),
                        _ => unreachable!(),
                    };
                    let item = item_to_data(item, s);
                    data::Field { item, ty }
                })
                .collect(),
            impls: s
                .impls
                .iter()
                .map(|id| self.item_to_impl(id))
                .filter(|i| i.trait_.is_none())
                .collect(),
            trait_impls: s
                .impls
                .iter()
                .map(|id| self.item_to_impl(id))
                .filter(|i| i.trait_.is_some() && !i.synthetic)
                .collect(),
            auto_impls: s
                .impls
                .iter()
                .map(|id| self.item_to_impl(id))
                .filter(|i| i.synthetic)
                .collect(),
        };
        struct_
    }

    fn item_to_impl(&self, id: &rdoc::Id) -> data::Impl {
        let item = &self.krate.index[id];
        let s = self.krate.paths.get(id);
        match &item.inner {
            ItemEnum::ImplItem(i) => {
                let item = item_to_data(item, s);
                data::Impl {
                    item,
                    is_unsafe: i.is_unsafe,
                    generics: i.generics.clone(),
                    provided_trait_methods: i.provided_trait_methods.clone(),
                    trait_: i.trait_.clone(),
                    for_: i.for_.clone(),
                    items: i
                        .items
                        .iter()
                        .filter_map(|id| {
                            let item = &self.krate.index[id];
                            let s = self.krate.paths.get(id);
                            match &item.inner {
                                ItemEnum::FunctionItem(f) => {
                                    Some(data::ImplItem::Fn(item_to_fn(item, s, f)))
                                }
                                _ => None,
                            }
                        })
                        .collect(),
                    negative: i.negative,
                    synthetic: i.synthetic,
                    blanket_impl: i.blanket_impl.clone(),
                }
            }
            _ => unreachable!(),
        }
    }
}

fn item_to_fn(item: &rdoc::Item, s: Option<&rdoc::ItemSummary>, m: &rdoc::Function) -> data::Fn {
    let item = item_to_data(item, s);
    data::Fn {
        item,
        decl: m.decl.clone(),
        generics: m.generics.clone(),
        header: m.header.clone(),
        abi: m.abi.clone(),
    }
}

fn item_to_data(item: &rdoc::Item, s: Option<&rdoc::ItemSummary>) -> data::Item {
    data::Item {
        name: item.name.clone().unwrap_or("_".into()),
        id: item.id.clone(),
        parents: s.map(|s| s.path.iter().take(s.path.len() - 1).cloned().collect()).unwrap_or_default(),
        short_doc: item
            .docs
            .as_deref()
            .and_then(|doc| doc.lines().next())
            .map(Into::into),
        doc: item.docs.clone().as_deref().map(markdown_to_text),
    }
}

impl AppDelegate<data::Screen> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut druid::DelegateCtx,
        _target: druid::Target,
        cmd: &druid::Command,
        data: &mut data::Screen,
        _env: &druid::Env,
    ) -> druid::Handled {
        if let Some(link) = cmd.get(OPEN_LINK) {
            open::that_in_background(link);
        }
        if let Some(id) = cmd.get(GOTO_ITEM) {
            self.current = id.clone();
            *data = self.data();
        }
        druid::Handled::No
    }
}
