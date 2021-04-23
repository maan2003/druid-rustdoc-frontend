use druid::im::Vector;
use druid::text::RichText;
use druid::Data;
use druid_enums::Matcher;
use rustdoc_types::{FnDecl, Generics, Qualifiers, Type};

#[derive(Data, Clone, Debug, Matcher)]
pub enum Screen {
    #[matcher(builder_name = mod_)]
    Mod(Mod),
    #[matcher(builder_name = struct_)]
    Struct(Struct),
    #[matcher(builder_name = enum_)]
    Enum(Enum),
    #[matcher(builder_name = fn_)]
    Fn(Fn),
}

#[derive(Data, Clone, Debug)]
pub struct Item {
    pub name: String,
    pub parents: Vector<String>,
    pub id: rustdoc_types::Id,
    pub short_doc: Option<String>,
    pub doc: Option<RichText>,
}

#[derive(Data, Clone, Debug)]
pub struct Mod {
    pub item: Item,
    pub structs: Vector<Item>,
    pub enums: Vector<Item>,
    pub traits: Vector<Item>,
    pub mods: Vector<Item>,
    pub fns: Vector<Item>,
}

#[derive(Data, Clone, Debug)]
pub struct Struct {
    pub item: Item,
    pub generics: Generics,
    pub fields: Vector<Field>,
    pub impls: Vector<Impl>,
    pub trait_impls: Vector<Impl>,
    pub auto_impls: Vector<Impl>,
}

#[derive(Data, Clone, Debug)]
pub struct Enum {
    pub item: Item,
    pub generics: Generics,
    pub variants: Vector<Variant>,
    pub impls: Vector<Impl>,
    pub trait_impls: Vector<Impl>,
    pub auto_impls: Vector<Impl>,
}

#[derive(Data, Clone, Debug)]
pub struct Field {
    pub item: Item,
    pub ty: Type,
}

#[derive(Data, Clone, Debug)]
pub struct Variant {
    pub item: Item,
    pub inner: VariantInner,
}

#[derive(Data, Clone, Debug)]
pub enum VariantInner {
    Plain,
    Tuple(Vector<Type>),
    Struct(Vector<Field>),
}

#[derive(Data, Clone, Debug)]
pub struct Impl {
    pub item: Item,
    pub is_unsafe: bool,
    pub generics: Generics,
    pub provided_trait_methods: Vector<String>,
    pub trait_: Option<Type>,
    pub for_: Type,
    pub fns: Vector<Fn>,
    pub negative: bool,
    pub synthetic: bool,
    pub blanket_impl: Option<Type>,
}


#[derive(Data, Clone, Debug)]
pub struct Fn {
    pub item: Item,
    pub decl: FnDecl,
    pub generics: Generics,
    pub header: Vector<Qualifiers>,
    pub abi: String,
}
