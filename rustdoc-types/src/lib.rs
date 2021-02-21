//! This is my crate.

use std::{fmt::Display, path::PathBuf, rc::Rc, sync::Arc};

use druid::{Data, Lens};
use im::{HashMap, Vector};
use serde::{Deserialize, Serialize};

pub fn parse(s: &str) -> Crate {
    serde_json::from_str(s).unwrap()
}
/// A `Crate` is the root of the emitted JSON blob. It contains all type/documentation information
/// about the language items in the local crate, as well as info about external items to allow
/// tools to find or link to them.
#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct Crate {
    /// The id of the root [`Module`] item of the local crate.
    pub root: Id,
    /// The version string given to `--crate-version`, if any.
    pub crate_version: Option<String>,
    /// Whether or not the output includes private items.
    pub includes_private: bool,
    /// A collection of all items in the local crate as well as some external traits and their
    /// items that are referenced locally.
    pub index: HashMap<Id, Item>,

    /// Maps IDs to fully qualified paths and other info helpful for generating links.
    pub paths: HashMap<Id, ItemSummary>,
    /// Maps `crate_id` of items to a crate name and html_root_url if it exists.
    pub external_crates: HashMap<u32, ExternalCrate>,
    /// A single version number to be used in the future when making backwards incompatible changes
    /// to the JSON output.
    pub format_version: u32,
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct ExternalCrate {
    pub name: String,
    pub html_root_url: Option<String>,
}

/// For external (not defined in the local crate) items, you don't get the same level of
/// information. This struct should contain enough to generate a link/reference to the item in
/// question, or can be used by a tool that takes the json output of multiple crates to find
/// the actual item definition with all the relevant info.
#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct ItemSummary {
    /// Can be used to look up the name and html_root_url of the crate this item came from in the
    /// `external_crates` map.
    pub crate_id: u32,
    /// The list of path components for the fully qualified path of this item (e.g.
    /// `["std", "io", "lazy", "Lazy"]` for `std::io::lazy::Lazy`).
    pub path: Vector<String>,
    /// Whether this item is a struct, trait, macro, etc.
    pub kind: ItemKind,
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct Item {
    /// The unique identifier of this item. Can be used to find this item in various mappings.
    pub id: Id,
    /// This can be used as a key to the `external_crates` map of [`Crate`] to see which crate
    /// this item came from.
    pub crate_id: u32,
    /// Some items such as impls don't have names.
    pub name: Option<String>,
    /// The source location of this item (absent if it came from a macro expansion or inline
    /// assembly).
    pub source: Option<Span>,
    /// By default all documented items are public, but you can tell rustdoc to output private items
    /// so this field is needed to differentiate.
    pub visibility: Visibility,
    /// The full markdown docstring of this item.
    #[serde(default)]
    pub docs: Option<Arc<str>>,
    /// This mapping resolves [intra-doc links](https://github.com/rust-lang/rfcs/blob/master/text/1946-intra-rustdoc-links.md) from the docstring to their IDs
    pub links: HashMap<String, Id>,
    /// Stringified versions of the attributes on this item (e.g. `"#[inline]", PartialEq`)
    pub attrs: Vector<String>,
    pub deprecation: Option<Deprecation>,
    pub kind: ItemKind,
    pub inner: ItemEnum,
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct Span {
    /// The path to the source file for this span relative to the path `rustdoc` was invoked with.
    #[data(same_fn = "PartialEq::eq")]
    pub filename: PathBuf,
    /// Zero indexed Line and Column of the first character of the `Span`
    pub begin: (usize, usize),
    /// Zero indexed Line and Column of the last character of the `Span`
    pub end: (usize, usize),
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct Deprecation {
    pub since: Option<String>,
    pub note: Option<String>,
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Data, Serialize, Deserialize, PartialEq)]
pub enum Visibility {
    Public,
    /// For the most part items are private by default. The exceptions are associated items of
    /// public traits and variants of public enums.
    Default,
    Crate,
    /// For `pub(in path)` visibility. `parent` is the module it's restricted to and `path` is how
    /// that module was referenced (like `"super::super"` or `"crate::foo::bar"`).
    Restricted {
        parent: Id,
        path: String,
    },
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Data, Serialize, Deserialize, PartialEq)]
pub enum GenericArgs {
    /// <'a, 32, B: Copy, C = u32>
    AngleBracketed {
        args: Vector<GenericArg>,
        bindings: Vector<TypeBinding>,
    },
    /// Fn(A, B) -> C
    Parenthesized {
        inputs: Vector<Type>,
        output: Option<Type>,
    },
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Data, Serialize, Deserialize, PartialEq)]
pub enum GenericArg {
    Lifetime(String),
    Type(Type),
    Const(Constant),
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct Constant {
    #[serde(rename = "type")]
    pub type_: Type,
    pub expr: String,
    pub value: Option<String>,
    pub is_literal: bool,
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct TypeBinding {
    pub name: String,
    pub binding: TypeBindingKind,
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Data, Serialize, Deserialize, PartialEq)]
pub enum TypeBindingKind {
    Equality(Type),
    Constraint(Vector<GenericBound>),
}

#[derive(Clone, Debug, Data, Eq, Hash, Serialize, Deserialize, PartialEq)]
pub struct Id(pub String);

#[repr(u8)]
#[derive(Clone, Debug, Copy, Data, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    Module,
    ExternCrate,
    Import,
    Struct,
    StructField,
    Union,
    Enum,
    Variant,
    Function,
    Typedef,
    OpaqueTy,
    Constant,
    Trait,
    TraitAlias,
    Method,
    Impl,
    Static,
    ForeignType,
    Macro,
    ProcAttribute,
    ProcDerive,
    AssocConst,
    AssocType,
    Primitive,
    Keyword,
}

#[serde(untagged)]
#[derive(Clone, Debug, Data, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ItemEnum {
    ModuleItem(Module),
    ExternCrateItem {
        name: String,
        rename: Option<String>,
    },
    ImportItem(Import),

    UnionItem(Union),
    StructItem(Struct),
    StructFieldItem(Type),
    EnumItem(Enum),
    VariantItem(Variant),

    FunctionItem(Function),

    TraitItem(Trait),
    TraitAliasItem(TraitAlias),
    MethodItem(Method),
    ImplItem(Impl),

    TypedefItem(Typedef),
    OpaqueTyItem(OpaqueTy),
    ConstantItem(Constant),

    StaticItem(Static),

    /// `type`s from an extern block
    ForeignTypeItem,

    /// Declarative macro_rules! macro
    MacroItem(String),
    ProcMacroItem(ProcMacro),

    AssocConstItem {
        #[serde(rename = "type")]
        type_: Type,
        /// e.g. `const X: usize = 5;`
        default: Option<String>,
    },
    AssocTypeItem {
        bounds: Vector<GenericBound>,
        /// e.g. `type X = usize;`
        default: Option<Type>,
    },
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct Module {
    pub is_crate: bool,
    pub items: Vector<Id>,
}

#[derive(Clone, Lens, Data, Debug, Serialize, Deserialize, PartialEq)]
pub struct Union {
    pub generics: Generics,
    pub fields_stripped: bool,
    pub fields: Vector<Id>,
    pub impls: Vector<Id>,
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct Struct {
    pub struct_type: StructType,
    pub generics: Generics,
    pub fields_stripped: bool,
    pub fields: Vector<Id>,
    pub impls: Vector<Id>,
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct Enum {
    pub generics: Generics,
    pub variants_stripped: bool,
    pub variants: Vector<Id>,
    pub impls: Vector<Id>,
}

#[serde(rename_all = "snake_case")]
#[serde(tag = "variant_kind", content = "variant_inner")]
#[derive(Clone, Debug, Data, Serialize, Deserialize, PartialEq)]
pub enum Variant {
    Plain,
    Tuple(Vector<Type>),
    Struct(Vector<Id>),
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Data, Serialize, Deserialize, PartialEq)]
pub enum StructType {
    Plain,
    Tuple,
    Unit,
    Union,
}

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Data, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Qualifiers {
    Const,
    Unsafe,
    Async,
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct Function {
    pub decl: FnDecl,
    pub generics: Generics,
    pub header: Vector<Qualifiers>,
    pub abi: String,
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct Method {
    pub decl: FnDecl,
    pub generics: Generics,
    pub header: Vector<Qualifiers>,
    pub abi: String,
    pub has_body: bool,
}

#[derive(Clone, Debug, Data, Lens, Default, Serialize, Deserialize, PartialEq)]
pub struct Generics {
    pub params: Vector<GenericParamDef>,
    pub where_predicates: Vector<WherePredicate>,
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct GenericParamDef {
    pub name: String,
    pub kind: GenericParamDefKind,
}

impl Display for GenericParamDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            GenericParamDefKind::Lifetime => write!(f, "{}", self.name),
            GenericParamDefKind::Type { bounds, default } => {
                write!(f, "{}", self.name)?;
                if !bounds.is_empty() {
                    let bounds = bounds
                        .iter()
                        .map(|x| match x {
                            GenericBound::TraitBound {
                                trait_,
                                generic_params,
                                modifier,
                            } => {
                                format!(
                                    "{}{}",
                                    match modifier {
                                        TraitBoundModifier::Maybe => "?",
                                        TraitBoundModifier::MaybeConst => "?const ",
                                        TraitBoundModifier::None => "",
                                    },
                                    trait_
                                )
                            }
                            GenericBound::Outlives(l) => l.clone(),
                        })
                        .collect::<Vec<_>>()
                        .join(" + ");
                    write!(f, ": {}", bounds)?;
                }
                Ok(())
                // format!("{}: {} = {}", x.name, bounds)
            }
            GenericParamDefKind::Const(c) => write!(f, "const {}: {}", self.name, c),
        }
    }
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Data, Serialize, Deserialize, PartialEq)]
pub enum GenericParamDefKind {
    Lifetime,
    Type {
        bounds: Vector<GenericBound>,
        default: Option<Type>,
    },
    Const(Type),
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Data, Serialize, Deserialize, PartialEq)]
pub enum WherePredicate {
    BoundPredicate {
        ty: Type,
        bounds: Vector<GenericBound>,
    },
    RegionPredicate {
        lifetime: String,
        bounds: Vector<GenericBound>,
    },
    EqPredicate {
        lhs: Type,
        rhs: Type,
    },
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Data, Serialize, Deserialize, PartialEq)]
pub enum GenericBound {
    TraitBound {
        #[serde(rename = "trait")]
        trait_: Type,
        /// Used for HRTBs
        generic_params: Vector<GenericParamDef>,
        modifier: TraitBoundModifier,
    },
    Outlives(String),
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Data, Serialize, Deserialize, PartialEq)]
pub enum TraitBoundModifier {
    None,
    Maybe,
    MaybeConst,
}

#[serde(rename_all = "snake_case")]
#[serde(tag = "kind", content = "inner")]
#[derive(Clone, Debug, Data, Serialize, Deserialize, PartialEq)]
pub enum Type {
    /// Structs, enums, and traits
    ResolvedPath {
        name: String,
        id: Id,
        args: Option<Rc<GenericArgs>>,
        param_names: Vector<GenericBound>,
    },
    /// Parameterized types
    Generic(String),
    /// Fixed-size numeric types (plus int/usize/float), char, arrays, slices, and tuples
    Primitive(String),
    /// `extern "ABI" fn`
    FunctionPointer(Rc<FunctionPointer>),
    /// `(String, u32, Rc<usize>)`
    Tuple(Vector<Type>),
    /// `[u32]`
    Slice(Rc<Type>),
    /// [u32; 15]
    Array {
        #[serde(rename = "type")]
        type_: Rc<Type>,
        len: String,
    },
    /// `impl TraitA + TraitB + ...`
    ImplTrait(Vector<GenericBound>),
    /// `!`
    Never,
    /// `_`
    Infer,
    /// `*mut u32`, `*u8`, etc.
    RawPointer {
        mutable: bool,
        #[serde(rename = "type")]
        type_: Rc<Type>,
    },
    /// `&'a mut String`, `&str`, etc.
    BorrowedRef {
        lifetime: Option<String>,
        mutable: bool,
        #[serde(rename = "type")]
        type_: Rc<Type>,
    },
    /// `<Type as Trait>::Name` or associated types like `T::Item` where `T: Iterator`
    QualifiedPath {
        name: String,
        self_type: Rc<Type>,
        #[serde(rename = "trait")]
        trait_: Rc<Type>,
    },
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::ResolvedPath {
                name,
                id,
                args,
                param_names,
            } => {
                write!(f, "{}", name)
            }
            Type::Generic(g) => write!(f, "{}", g),
            Type::Primitive(p) => write!(f, "{}", p),
            Type::FunctionPointer(f) => panic!("unknown"),
            Type::Tuple(t) => write!(
                f,
                "({})",
                t.iter()
                    .map(|x| format!("{}", x))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Type::Slice(s) => write!(f, "[{}]", s),
            Type::Array { type_, len } => write!(f, "[{}; {}]", type_, len),
            Type::Never => write!(f, "!"),
            Type::Infer => write!(f, "_"),
            Type::RawPointer { mutable: mu, type_ } => {
                write!(f, "*{} {}", if *mu { "mut" } else { "const" }, type_)
            }
            Type::BorrowedRef {
                lifetime: Some(lf),
                mutable: mu,
                type_,
            } => {
                write!(f, "&'{} {}{}", lf, if *mu { "mut " } else { "" }, type_)
            }
            Type::BorrowedRef {
                lifetime: None,
                mutable: mu,
                type_,
            } => {
                write!(f, "&{}{}", if *mu { "mut " } else { "" }, type_)
            }
            Type::QualifiedPath {
                name,
                self_type,
                trait_,
            } => write!(f, "<{} as {}>::{}", self_type, trait_, name),
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct FunctionPointer {
    pub decl: FnDecl,
    pub generic_params: Vector<GenericParamDef>,
    pub header: Vector<Qualifiers>,
    pub abi: String,
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct FnDecl {
    pub inputs: Vector<(String, Type)>,
    pub output: Option<Type>,
    pub c_variadic: bool,
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct Trait {
    pub is_auto: bool,
    pub is_unsafe: bool,
    pub items: Vector<Id>,
    pub generics: Generics,
    pub bounds: Vector<GenericBound>,
    pub implementors: Vector<Id>,
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct TraitAlias {
    pub generics: Generics,
    pub params: Vector<GenericBound>,
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct Impl {
    pub is_unsafe: bool,
    pub generics: Generics,
    pub provided_trait_methods: Vector<String>,
    #[serde(rename = "trait")]
    pub trait_: Option<Type>,
    #[serde(rename = "for")]
    pub for_: Type,
    pub items: Vector<Id>,
    pub negative: bool,
    pub synthetic: bool,
    pub blanket_impl: Option<Type>,
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct Import {
    /// The full path being imported.
    pub span: String,
    /// May be different from the last segment of `source` when renaming imports:
    /// `use source as name;`
    pub name: String,
    /// The ID of the item being imported.
    pub id: Option<Id>, // FIXME is this actually ever None?
    /// Whether this import uses a glob: `use source::*;`
    pub glob: bool,
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct ProcMacro {
    pub kind: MacroKind,
    pub helpers: Vector<String>,
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Data, Serialize, Deserialize, PartialEq)]
pub enum MacroKind {
    /// A bang macro `foo!()`.
    Bang,
    /// An attribute macro `#[foo]`.
    Attr,
    /// A derive macro `#[derive(Foo)]`
    Derive,
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct Typedef {
    #[serde(rename = "type")]
    pub type_: Type,
    pub generics: Generics,
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct OpaqueTy {
    pub bounds: Vector<GenericBound>,
    pub generics: Generics,
}

#[derive(Clone, Debug, Data, Lens, Serialize, Deserialize, PartialEq)]
pub struct Static {
    #[serde(rename = "type")]
    pub type_: Type,
    pub mutable: bool,
    pub expr: String,
}
