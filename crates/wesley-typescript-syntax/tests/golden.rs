//! Printed output is pinned byte for byte to the files under `tests/golden/`.
//!
//! Set `UPDATE_GOLDEN=1` to rewrite the files from the current output; the
//! compiler test (`tests/tsc.rs`) still has to accept whatever is written.

mod common;

use std::fs;

use wesley_typescript_syntax::print_module;

#[test]
fn printed_modules_match_their_golden_files() {
    let update = std::env::var_os("UPDATE_GOLDEN").is_some();
    for case in common::cases() {
        let printed = print_module(&case.module).expect("golden module prints");
        let path = common::golden_dir().join(case.file);
        if update {
            fs::write(&path, &printed).expect("golden file is writable");
        }
        let expected = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("reading {}: {err}", path.display()));
        assert_eq!(
            printed, expected,
            "{} differs from its golden file",
            case.file
        );
    }
}

#[test]
fn output_ends_with_exactly_one_newline() {
    for case in common::cases() {
        let printed = print_module(&case.module).expect("golden module prints");
        assert!(
            printed.ends_with('\n'),
            "{} has no final newline",
            case.file
        );
        assert!(
            !printed.ends_with("\n\n"),
            "{} ends with more than one newline",
            case.file
        );
    }
}

#[test]
fn printing_twice_gives_identical_bytes() {
    for case in common::cases() {
        let first = print_module(&case.module).expect("first print");
        let second = print_module(&case.module).expect("second print");
        assert_eq!(first.as_bytes(), second.as_bytes(), "{}", case.file);
    }
}

#[test]
fn printing_a_clone_gives_identical_bytes() {
    for case in common::cases() {
        let first = print_module(&case.module).expect("first print");
        let second = print_module(&case.module.clone()).expect("second print");
        assert_eq!(first.as_bytes(), second.as_bytes(), "{}", case.file);
    }
}
