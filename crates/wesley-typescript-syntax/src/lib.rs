//! TypeScript declarations built as an swc syntax tree and printed by swc.

/// A TypeScript type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TsType {
    String,
    Number,
    Boolean,
    Unknown,
    Null,
    StringLiteral(String),
    NumberLiteral(i64),
    BooleanLiteral(bool),
    Ref(String),
    Array(Box<TsType>),
    Union(Vec<TsType>),
    Object(Vec<Property>),
    Record(Box<TsType>),
}

/// A property of an interface or an inline object type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Property {
    pub name: String,
    pub optional: bool,
    pub readonly: bool,
    pub ty: TsType,
    pub doc: Option<String>,
}

/// An exported declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Declaration {
    Interface {
        name: String,
        doc: Option<String>,
        properties: Vec<Property>,
    },
    TypeAlias {
        name: String,
        doc: Option<String>,
        ty: TsType,
    },
}

/// A module of exported declarations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Module {
    pub doc: Option<String>,
    pub declarations: Vec<Declaration>,
}

/// Why a module could not be printed.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("a name is empty")]
    EmptyName,
    #[error("declaration name {0:?} is not an identifier that names a declaration")]
    InvalidDeclarationName(String),
    #[error("reference {0:?} is not an identifier")]
    InvalidTypeReference(String),
    #[error("property {0:?} appears twice in one object")]
    DuplicateProperty(String),
    #[error("declaration {0:?} appears twice in the module")]
    DuplicateDeclaration(String),
    #[error("a union has no members")]
    EmptyUnion,
    #[error("number literal {0} is not a safe integer")]
    UnsafeInteger(i64),
    #[error("the module has a doc but no declaration to carry it")]
    ModuleDocWithoutDeclarations,
    #[error("the printer failed: {0}")]
    Print(String),
}

/// Prints a module.
pub fn print_module(_module: &Module) -> Result<String, Error> {
    Ok(String::new())
}

/// Whether `name` is an ECMAScript IdentifierName.
pub fn is_identifier_name(_name: &str) -> bool {
    false
}
