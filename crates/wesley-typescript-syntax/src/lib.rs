//! TypeScript declarations built as an swc syntax tree and printed by swc.
//!
//! A [`Module`] describes exported interfaces and type aliases. [`print_module`]
//! checks it, builds the matching `swc_ecma_ast` module, attaches each doc as a
//! JSDoc block comment through swc's comment store, and prints the tree with
//! `swc_ecma_codegen`. No TypeScript source is written as a string: keywords,
//! punctuation, quoting and escaping are all the printer's.

mod jsdoc;
mod tree;
mod writer;

use swc_common::sync::Lrc;
use swc_common::SourceMap;
use swc_ecma_ast::Ident;
use swc_ecma_codegen::text_writer::{JsWriter, WriteJs};
use swc_ecma_codegen::{Config, Emitter};

/// A TypeScript type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TsType {
    /// `string`
    String,
    /// `number`
    Number,
    /// `boolean`
    Boolean,
    /// `unknown`
    Unknown,
    /// `null`
    Null,
    /// A string literal type; the printer quotes and escapes it.
    StringLiteral(String),
    /// A number literal type. It must be a safe integer (at most 2^53 - 1 in
    /// magnitude), which TypeScript holds exactly.
    NumberLiteral(i64),
    /// `true` or `false`
    BooleanLiteral(bool),
    /// A named type. The name must be an identifier that can name a type.
    Ref(String),
    /// `T[]`; a union element is parenthesised, `(A | B)[]`.
    Array(Box<TsType>),
    /// `A | B`; it must have at least one member.
    Union(Vec<TsType>),
    /// An inline object type literal.
    Object(Vec<Property>),
    /// `Record<string, T>`
    Record(Box<TsType>),
}

/// A property of an interface or an inline object type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Property {
    /// The key. A name that is not an identifier name is printed as a
    /// string-literal key, e.g. `"what-ran"`.
    pub name: String,
    /// `name?:`
    pub optional: bool,
    /// `readonly name:`
    pub readonly: bool,
    pub ty: TsType,
    /// Printed as a JSDoc comment before the property.
    pub doc: Option<String>,
}

/// An exported declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Declaration {
    /// `export interface Name { ... }`
    Interface {
        name: String,
        doc: Option<String>,
        properties: Vec<Property>,
    },
    /// `export type Name = ...;`
    TypeAlias {
        name: String,
        doc: Option<String>,
        ty: TsType,
    },
}

/// A module of exported declarations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Module {
    /// Printed as a JSDoc comment at the top of the module, as a leading
    /// comment of the first declaration (before that declaration's own doc).
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
    #[error("reference {0:?} is not an identifier that names a declaration")]
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
    #[error("the module has more documented nodes than synthetic positions")]
    TooManyDocs,
    #[error("the printer failed: {0}")]
    Print(String),
}

/// Checks `module`, builds it as an swc syntax tree and prints it with swc's
/// code generator.
///
/// The output is byte-deterministic and ends with exactly one newline, except
/// that a module with no declarations prints as the empty string.
///
/// # Errors
///
/// Returns an [`Error`] for an empty name, a declaration or reference name
/// that is not a usable identifier, a property name repeated in one object, a
/// declaration name repeated in the module, an empty union, a number literal
/// that is not a safe integer, or a module doc with nothing to attach to.
pub fn print_module(module: &Module) -> Result<String, Error> {
    let mut builder = tree::Builder::new();
    let program = builder.module(module)?;

    let cm: Lrc<SourceMap> = Lrc::default();
    let mut buf = Vec::new();
    {
        let mut emitter = Emitter {
            cfg: Config::default(),
            cm: cm.clone(),
            comments: Some(&builder.comments),
            wr: writer::DocLines::new(JsWriter::new(cm, "\n", &mut buf, None)),
        };
        emitter
            .emit_module(&program)
            .map_err(|err| Error::Print(err.to_string()))?;
        emitter
            .wr
            .write_line()
            .map_err(|err| Error::Print(err.to_string()))?;
    }
    String::from_utf8(buf).map_err(|err| Error::Print(err.to_string()))
}

/// Whether `name` is an ECMAScript `IdentifierName`: an identifier start
/// (`ID_Start`, `$` or `_`) followed by identifier parts (`ID_Continue`, `$`,
/// ZWNJ or ZWJ). Reserved words are identifier names, so they are valid
/// property keys.
pub fn is_identifier_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(Ident::is_valid_start)
        && chars.all(|c| Ident::is_valid_continue(c) || matches!(c, '\u{200C}' | '\u{200D}'))
}
