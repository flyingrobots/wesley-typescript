//! Builds an swc module from a [`Module`], checking it on the way.
//!
//! Nodes that carry JSDoc get a synthetic span whose `lo` is a fresh
//! [`BytePos`]; the comment is stored in a [`SingleThreadedComments`] as a
//! leading comment at that position, which is where swc's printer looks for
//! it. Every other node has a dummy span, for which the printer looks for no
//! comment at all.

use std::collections::HashSet;

use swc_atoms::{Atom, Wtf8Atom};
use swc_common::comments::{Comment, CommentKind, Comments, SingleThreadedComments};
use swc_common::{BytePos, Span, DUMMY_SP};
use swc_ecma_ast as ast;

use crate::jsdoc::comment_text;
use crate::{Declaration, Error, Module, Property, TsType};

/// The largest integer a TypeScript number literal type holds exactly.
const MAX_SAFE_INTEGER: i64 = (1 << 53) - 1;

/// The type that `TsType::Record` names: `Record<string, T>`.
const RECORD: &str = "Record";

/// Names TypeScript reserves for its own types, which no declaration may take.
const PREDEFINED_TYPE_NAMES: &[&str] = &[
    "any",
    "bigint",
    "boolean",
    "never",
    "number",
    "object",
    "string",
    "symbol",
    "undefined",
    "unknown",
];

pub(crate) struct Builder {
    pub(crate) comments: SingleThreadedComments,
    next_pos: u32,
}

impl Builder {
    pub(crate) fn new() -> Self {
        Self {
            comments: SingleThreadedComments::default(),
            next_pos: 1,
        }
    }

    /// A span that carries `docs` as leading JSDoc comments, in order, or a
    /// dummy span when there are none.
    ///
    /// Positions advance by two: the printer also looks for comments at the
    /// position just before a list item's `hi`, and an odd-only allocation
    /// keeps that position free.
    fn doc_span(&mut self, docs: &[&str]) -> Result<Span, Error> {
        if docs.is_empty() {
            return Ok(DUMMY_SP);
        }
        let pos = BytePos(self.next_pos);
        self.next_pos = self.next_pos.checked_add(2).ok_or(Error::TooManyDocs)?;
        for doc in docs {
            self.comments.add_leading(
                pos,
                Comment {
                    kind: CommentKind::Block,
                    span: DUMMY_SP,
                    text: Atom::from(comment_text(doc)),
                },
            );
        }
        Ok(Span::new(pos, pos))
    }

    pub(crate) fn module(&mut self, module: &Module) -> Result<ast::Module, Error> {
        if module.declarations.is_empty() && module.doc.is_some() {
            return Err(Error::ModuleDocWithoutDeclarations);
        }
        let mut seen = HashSet::new();
        let mut body = Vec::with_capacity(module.declarations.len());
        for (index, declaration) in module.declarations.iter().enumerate() {
            let name = declaration_name(declaration);
            check_declaration_name(name)?;
            if !seen.insert(name) {
                return Err(Error::DuplicateDeclaration(name.to_owned()));
            }
            let mut docs = Vec::new();
            if index == 0 {
                docs.extend(module.doc.as_deref());
            }
            docs.extend(declaration_doc(declaration));
            body.push(self.declaration(declaration, &docs)?);
        }
        Ok(ast::Module {
            span: DUMMY_SP,
            body,
            shebang: None,
        })
    }

    fn declaration(
        &mut self,
        declaration: &Declaration,
        docs: &[&str],
    ) -> Result<ast::ModuleItem, Error> {
        let span = self.doc_span(docs)?;
        let decl = match declaration {
            Declaration::Interface {
                name, properties, ..
            } => ast::Decl::TsInterface(Box::new(ast::TsInterfaceDecl {
                span: DUMMY_SP,
                id: ident(name),
                declare: false,
                type_params: None,
                extends: Vec::new(),
                body: ast::TsInterfaceBody {
                    span: DUMMY_SP,
                    body: self.members(properties)?,
                },
            })),
            Declaration::TypeAlias { name, ty, .. } => {
                ast::Decl::TsTypeAlias(Box::new(ast::TsTypeAliasDecl {
                    span: DUMMY_SP,
                    declare: false,
                    id: ident(name),
                    type_params: None,
                    type_ann: Box::new(self.ty(ty)?),
                }))
            }
        };
        Ok(ast::ModuleItem::ModuleDecl(ast::ModuleDecl::ExportDecl(
            ast::ExportDecl { span, decl },
        )))
    }

    fn members(&mut self, properties: &[Property]) -> Result<Vec<ast::TsTypeElement>, Error> {
        let mut seen = HashSet::new();
        let mut members = Vec::with_capacity(properties.len());
        for property in properties {
            if property.name.is_empty() {
                return Err(Error::EmptyName);
            }
            if !seen.insert(property.name.as_str()) {
                return Err(Error::DuplicateProperty(property.name.clone()));
            }
            let docs: Vec<&str> = property.doc.as_deref().into_iter().collect();
            let span = self.doc_span(&docs)?;
            let ty = self.ty(&property.ty)?;
            members.push(ast::TsTypeElement::TsPropertySignature(
                ast::TsPropertySignature {
                    span,
                    readonly: property.readonly,
                    key: Box::new(property_key(&property.name)),
                    computed: false,
                    optional: property.optional,
                    type_ann: Some(Box::new(ast::TsTypeAnn {
                        span: DUMMY_SP,
                        type_ann: Box::new(ty),
                    })),
                },
            ));
        }
        Ok(members)
    }

    fn ty(&mut self, ty: &TsType) -> Result<ast::TsType, Error> {
        Ok(match ty {
            TsType::String => keyword(ast::TsKeywordTypeKind::TsStringKeyword),
            TsType::Number => keyword(ast::TsKeywordTypeKind::TsNumberKeyword),
            TsType::Boolean => keyword(ast::TsKeywordTypeKind::TsBooleanKeyword),
            TsType::Unknown => keyword(ast::TsKeywordTypeKind::TsUnknownKeyword),
            TsType::Null => keyword(ast::TsKeywordTypeKind::TsNullKeyword),
            TsType::StringLiteral(value) => literal(ast::TsLit::Str(ast::Str {
                span: DUMMY_SP,
                value: Wtf8Atom::from(value.as_str()),
                raw: None,
            })),
            TsType::NumberLiteral(value) => {
                if !(-MAX_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(value) {
                    return Err(Error::UnsafeInteger(*value));
                }
                // Every safe integer converts to f64 exactly.
                #[allow(clippy::cast_precision_loss)]
                let value = *value as f64;
                literal(ast::TsLit::Number(ast::Number {
                    span: DUMMY_SP,
                    value,
                    raw: None,
                }))
            }
            TsType::BooleanLiteral(value) => literal(ast::TsLit::Bool(ast::Bool {
                span: DUMMY_SP,
                value: *value,
            })),
            TsType::Ref(name) => {
                check_reference(name)?;
                reference(name, None)
            }
            TsType::Array(element) => {
                let element = self.ty(element)?;
                let element = match element {
                    ast::TsType::TsUnionOrIntersectionType(_) => {
                        ast::TsType::TsParenthesizedType(ast::TsParenthesizedType {
                            span: DUMMY_SP,
                            type_ann: Box::new(element),
                        })
                    }
                    other => other,
                };
                ast::TsType::TsArrayType(ast::TsArrayType {
                    span: DUMMY_SP,
                    elem_type: Box::new(element),
                })
            }
            TsType::Union(members) => {
                if members.is_empty() {
                    return Err(Error::EmptyUnion);
                }
                let types = members
                    .iter()
                    .map(|member| self.ty(member).map(Box::new))
                    .collect::<Result<Vec<_>, _>>()?;
                ast::TsType::TsUnionOrIntersectionType(ast::TsUnionOrIntersectionType::TsUnionType(
                    ast::TsUnionType {
                        span: DUMMY_SP,
                        types,
                    },
                ))
            }
            TsType::Object(properties) => ast::TsType::TsTypeLit(ast::TsTypeLit {
                span: DUMMY_SP,
                members: self.members(properties)?,
            }),
            TsType::Record(value) => {
                let value = self.ty(value)?;
                reference(
                    RECORD,
                    Some(vec![
                        keyword(ast::TsKeywordTypeKind::TsStringKeyword),
                        value,
                    ]),
                )
            }
        })
    }
}

fn declaration_name(declaration: &Declaration) -> &str {
    match declaration {
        Declaration::Interface { name, .. } | Declaration::TypeAlias { name, .. } => name,
    }
}

fn declaration_doc(declaration: &Declaration) -> Option<&str> {
    match declaration {
        Declaration::Interface { doc, .. } | Declaration::TypeAlias { doc, .. } => doc.as_deref(),
    }
}

/// Whether `name` can name a declared type: an identifier that is not a
/// reserved word and not one of TypeScript's predefined type names.
fn is_type_name(name: &str) -> bool {
    crate::is_identifier_name(name)
        && ast::Ident::verify_symbol(name).is_ok()
        && !PREDEFINED_TYPE_NAMES.contains(&name)
}

fn check_declaration_name(name: &str) -> Result<(), Error> {
    if name.is_empty() {
        Err(Error::EmptyName)
    } else if is_type_name(name) {
        Ok(())
    } else {
        Err(Error::InvalidDeclarationName(name.to_owned()))
    }
}

fn check_reference(name: &str) -> Result<(), Error> {
    if name.is_empty() {
        Err(Error::EmptyName)
    } else if is_type_name(name) {
        Ok(())
    } else {
        Err(Error::InvalidTypeReference(name.to_owned()))
    }
}

fn ident(name: &str) -> ast::Ident {
    ast::Ident::new_no_ctxt(Atom::from(name), DUMMY_SP)
}

/// An identifier key when the name is an identifier name, otherwise a string
/// literal key that the printer quotes and escapes.
fn property_key(name: &str) -> ast::Expr {
    if crate::is_identifier_name(name) {
        ast::Expr::Ident(ident(name))
    } else {
        ast::Expr::Lit(ast::Lit::Str(ast::Str {
            span: DUMMY_SP,
            value: Wtf8Atom::from(name),
            raw: None,
        }))
    }
}

fn keyword(kind: ast::TsKeywordTypeKind) -> ast::TsType {
    ast::TsType::TsKeywordType(ast::TsKeywordType {
        span: DUMMY_SP,
        kind,
    })
}

fn literal(lit: ast::TsLit) -> ast::TsType {
    ast::TsType::TsLitType(ast::TsLitType {
        span: DUMMY_SP,
        lit,
    })
}

fn reference(name: &str, params: Option<Vec<ast::TsType>>) -> ast::TsType {
    ast::TsType::TsTypeRef(ast::TsTypeRef {
        span: DUMMY_SP,
        type_name: ast::TsEntityName::Ident(ident(name)),
        type_params: params.map(|params| {
            Box::new(ast::TsTypeParamInstantiation {
                span: DUMMY_SP,
                params: params.into_iter().map(Box::new).collect(),
            })
        }),
    })
}
