use druid::im::Vector;
use druid::text::RichText;
use druid::{lens, Data};

#[derive(Data, Clone, Debug)]
pub enum Screen {
    Mod(Mod),
}

#[derive(Data, Clone, Debug)]
pub struct Mod {
    pub item: Item,
    pub structs: Vector<Item>,
    pub enums: Vector<Item>,
    pub traits: Vector<Item>,
    pub mods: Vector<Item>,
}

#[derive(Data, Clone, Debug)]
pub struct Item {
    pub name: String,
    pub id: rustdoc_types::Id,
    pub short_doc: Option<String>,
    pub doc: Option<RichText>,
}
