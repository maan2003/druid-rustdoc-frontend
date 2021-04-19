use std::fs;
use std::path::Path;

use druid::im::Vector;
use druid::text::{RichText, RichTextBuilder};
use druid::AppDelegate;
use rdoc::ItemEnum;
use rustdoc_types as rdoc;

use crate::data;
use crate::md::markdown_to_text;

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
        match &item.inner {
            ItemEnum::ModuleItem(m) => {
                let item = item_to_data(&item);
                let mod_ = data::Mod {
                    item,
                    structs: Vector::new(),
                    enums: Vector::new(),
                    traits: Vector::new(),
                    mods: Vector::new(),
                };
                data::Screen::Mod(mod_)
            }
            _ => todo!("handle other Items"),
        }
    }
}

fn item_to_data(item: &rdoc::Item) -> data::Item {
    data::Item {
        name: item.name.clone().unwrap_or("_".into()),
        id: item.id.clone(),
        short_doc: Some("Short docs are TODO".into()),
        doc: item.docs.clone().as_deref().map(markdown_to_text),
    }
}

impl AppDelegate<data::Screen> for Delegate {
    fn command(
        &mut self,
        ctx: &mut druid::DelegateCtx,
        target: druid::Target,
        cmd: &druid::Command,
        data: &mut data::Screen,
        env: &druid::Env,
    ) -> druid::Handled {
        druid::Handled::No
    }
}
