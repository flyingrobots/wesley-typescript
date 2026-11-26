//! A crude guard against string templating: no string literal in the crate's
//! `src/` may contain TypeScript syntax. Output is built as an swc tree and
//! printed by swc; TypeScript source text has no business in a Rust string.

use std::fs;
use std::path::{Path, PathBuf};

/// Fragments of TypeScript source that only templated output would contain.
const FORBIDDEN: &[&str] = &[
    "interface ",
    "export ",
    "type ",
    "readonly ",
    "Record<",
    "[]",
    "?:",
    " | ",
    "=>",
    "};",
];

fn rust_files(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|err| panic!("{}: {err}", dir.display())) {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            rust_files(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            found.push(path);
        }
    }
}

/// The contents of the string literals in Rust source, roughly: ordinary and
/// raw strings, skipping comments and character literals.
fn string_literals(source: &str) -> Vec<String> {
    let chars: Vec<char> = source.chars().collect();
    let mut literals = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '/' if chars.get(i + 1) == Some(&'/') => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '/' if chars.get(i + 1) == Some(&'*') => {
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
            }
            '\'' => {
                // A character literal ('"', '\'', '\u{..}') or a lifetime.
                if chars.get(i + 1) == Some(&'\\') {
                    i += 2;
                    while i < chars.len() && chars[i] != '\'' {
                        i += 1;
                    }
                    i += 1;
                } else if chars.get(i + 2) == Some(&'\'') {
                    i += 3;
                } else {
                    i += 1;
                }
            }
            'r' if matches!(chars.get(i + 1), Some('"' | '#'))
                && (i == 0 || !(chars[i - 1].is_alphanumeric() || chars[i - 1] == '_')) =>
            {
                let mut j = i + 1;
                let mut hashes = 0;
                while chars.get(j) == Some(&'#') {
                    hashes += 1;
                    j += 1;
                }
                if chars.get(j) != Some(&'"') {
                    i += 1;
                    continue;
                }
                j += 1;
                let start = j;
                loop {
                    if j >= chars.len() {
                        break;
                    }
                    if chars[j] == '"' && (1..=hashes).all(|k| chars.get(j + k) == Some(&'#')) {
                        break;
                    }
                    j += 1;
                }
                literals.push(chars[start..j.min(chars.len())].iter().collect());
                i = j + 1 + hashes;
            }
            '"' => {
                let mut j = i + 1;
                let mut literal = String::new();
                while j < chars.len() && chars[j] != '"' {
                    if chars[j] == '\\' {
                        j += 1;
                    }
                    if let Some(&c) = chars.get(j) {
                        literal.push(c);
                    }
                    j += 1;
                }
                literals.push(literal);
                i = j + 1;
            }
            _ => i += 1,
        }
    }
    literals
}

fn offences(source: &str) -> Vec<String> {
    string_literals(source)
        .into_iter()
        .filter(|literal| FORBIDDEN.iter().any(|needle| literal.contains(needle)))
        .collect()
}

#[test]
fn the_guard_finds_templated_typescript() {
    let templated = r##"
        fn print(name: &str) -> String {
            // "export type" in a comment is fine.
            let quote = '"';
            format!("export interface {name} {{ id: string }}")
        }
        const ALIAS: &str = r#"export type Run = string;"#;
    "##;
    assert_eq!(
        offences(templated),
        vec![
            "export interface {name} {{ id: string }}".to_owned(),
            "export type Run = string;".to_owned(),
        ]
    );
}

#[test]
fn no_string_literal_in_src_contains_typescript_syntax() {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    files.sort();
    assert!(!files.is_empty(), "src/ has Rust files");
    let mut found = Vec::new();
    for file in files {
        let source = fs::read_to_string(&file).expect("source is readable");
        for literal in offences(&source) {
            found.push(format!("{}: {literal:?}", file.display()));
        }
    }
    assert!(
        found.is_empty(),
        "string literals in src/ contain TypeScript syntax:\n{}",
        found.join("\n")
    );
}
