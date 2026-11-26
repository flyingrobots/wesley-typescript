//! Invalid input is reported as an `Error`, never a panic.

use wesley_typescript_syntax::{
    is_identifier_name, print_module, Declaration, Error, Module, Property, TsType,
};

fn prop(name: &str, ty: TsType) -> Property {
    Property {
        name: name.to_owned(),
        optional: false,
        readonly: false,
        ty,
        doc: None,
    }
}

fn interface(name: &str, properties: Vec<Property>) -> Module {
    Module {
        doc: None,
        declarations: vec![Declaration::Interface {
            name: name.to_owned(),
            doc: None,
            properties,
        }],
    }
}

fn alias(name: &str, ty: TsType) -> Module {
    Module {
        doc: None,
        declarations: vec![Declaration::TypeAlias {
            name: name.to_owned(),
            doc: None,
            ty,
        }],
    }
}

#[test]
fn an_empty_declaration_name_is_an_error() {
    assert_eq!(
        print_module(&interface("", Vec::new())),
        Err(Error::EmptyName)
    );
}

#[test]
fn an_empty_property_name_is_an_error() {
    assert_eq!(
        print_module(&interface("Run", vec![prop("", TsType::String)])),
        Err(Error::EmptyName)
    );
}

#[test]
fn a_declaration_name_that_is_not_an_identifier_is_an_error() {
    for name in ["what-ran", "1st", "a b", "class", "string"] {
        assert_eq!(
            print_module(&alias(name, TsType::String)),
            Err(Error::InvalidDeclarationName(name.to_owned())),
            "{name}"
        );
    }
}

#[test]
fn a_type_reference_that_is_not_an_identifier_is_an_error() {
    assert_eq!(
        print_module(&alias("Run", TsType::Ref("not-a-name".to_owned()))),
        Err(Error::InvalidTypeReference("not-a-name".to_owned()))
    );
    assert_eq!(
        print_module(&alias("Run", TsType::Ref(String::new()))),
        Err(Error::EmptyName)
    );
}

#[test]
fn duplicate_property_names_in_an_interface_are_an_error() {
    let module = interface(
        "Run",
        vec![prop("id", TsType::String), prop("id", TsType::Number)],
    );
    assert_eq!(
        print_module(&module),
        Err(Error::DuplicateProperty("id".to_owned()))
    );
}

#[test]
fn duplicate_property_names_in_an_inline_object_are_an_error() {
    let module = alias(
        "Run",
        TsType::Array(Box::new(TsType::Object(vec![
            prop("what-ran", TsType::String),
            prop("what-ran", TsType::String),
        ]))),
    );
    assert_eq!(
        print_module(&module),
        Err(Error::DuplicateProperty("what-ran".to_owned()))
    );
}

#[test]
fn duplicate_declaration_names_are_an_error() {
    let module = Module {
        doc: None,
        declarations: vec![
            Declaration::TypeAlias {
                name: "Run".to_owned(),
                doc: None,
                ty: TsType::String,
            },
            Declaration::Interface {
                name: "Run".to_owned(),
                doc: None,
                properties: Vec::new(),
            },
        ],
    };
    assert_eq!(
        print_module(&module),
        Err(Error::DuplicateDeclaration("Run".to_owned()))
    );
}

#[test]
fn an_empty_union_is_an_error() {
    assert_eq!(
        print_module(&alias("Run", TsType::Union(Vec::new()))),
        Err(Error::EmptyUnion)
    );
}

#[test]
fn a_number_literal_beyond_the_safe_integers_is_an_error() {
    let too_big = 1_i64 << 53;
    assert_eq!(
        print_module(&alias("Run", TsType::NumberLiteral(too_big))),
        Err(Error::UnsafeInteger(too_big))
    );
    assert_eq!(
        print_module(&alias("Run", TsType::NumberLiteral(-too_big))),
        Err(Error::UnsafeInteger(-too_big))
    );
    assert!(print_module(&alias("Run", TsType::NumberLiteral(too_big - 1))).is_ok());
}

#[test]
fn identifier_names_follow_ecmascript() {
    for name in ["id", "_x", "$", "café", "class", "what_ran", "a1"] {
        assert!(is_identifier_name(name), "{name} is an identifier name");
    }
    for name in ["", "what-ran", "1st", "a b", "a.b", "\"q\""] {
        assert!(
            !is_identifier_name(name),
            "{name} is not an identifier name"
        );
    }
}

#[test]
fn an_empty_module_prints_as_an_empty_file() {
    let module = Module {
        doc: None,
        declarations: Vec::new(),
    };
    assert_eq!(print_module(&module), Ok(String::new()));
}

#[test]
fn a_module_doc_without_declarations_is_an_error() {
    let module = Module {
        doc: Some("Nothing follows.".to_owned()),
        declarations: Vec::new(),
    };
    assert_eq!(
        print_module(&module),
        Err(Error::ModuleDocWithoutDeclarations)
    );
}
