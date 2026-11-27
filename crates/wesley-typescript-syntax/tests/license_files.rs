//! The crate carries its own LICENSE and NOTICE, so both travel in the
//! published `.crate`: Apache-2.0 asks that the NOTICE go with every
//! redistribution, and cargo packages only files under the crate directory.
//!
//! In the repository the copies must match the workspace root byte for byte.
//! Built from the published `.crate`, where there is no workspace root, only
//! the crate's own copies are checked.

use std::fs;
use std::path::PathBuf;

const FILES: [&str; 2] = ["LICENSE", "NOTICE"];

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn the_crate_carries_its_license_and_notice() {
    for file in FILES {
        let path = crate_dir().join(file);
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("reading {}: {err}", path.display()));
        assert!(!text.trim().is_empty(), "{} is empty", path.display());
    }
}

#[test]
fn the_crate_copies_match_the_workspace_root() {
    let root = crate_dir().join("..").join("..");
    if !root.join("Cargo.toml").is_file() {
        return;
    }
    for file in FILES {
        let root_copy = root.join(file);
        let crate_copy = crate_dir().join(file);
        let expected = fs::read(&root_copy)
            .unwrap_or_else(|err| panic!("reading {}: {err}", root_copy.display()));
        let actual = fs::read(&crate_copy)
            .unwrap_or_else(|err| panic!("reading {}: {err}", crate_copy.display()));
        assert!(
            expected == actual,
            "{} differs from {}",
            crate_copy.display(),
            root_copy.display()
        );
    }
}
