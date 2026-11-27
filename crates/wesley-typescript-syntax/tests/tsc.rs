//! Every golden file is type-checked by the real TypeScript compiler.
//!
//! The files under `tests/typescript/pass/` must type-check together with the
//! golden files. Each file under `tests/typescript/fail/` must be rejected on
//! its own, with the error code named on its first line
//! (`// expect-error: TS2322`).
//!
//! The compiler is run as `npx --yes -p typescript@5 tsc --noEmit --strict`.
//! When `npx` is not on `PATH` the test is skipped with a printed reason,
//! unless the `CI` environment variable is set, in which case it fails.
//!
//! The tests run on parallel threads, but `npx` is run one at a time: two
//! `npx` processes installing `typescript@5` into an empty npm cache at once
//! leave a half-written package, and `tsc` then fails with
//! `Cannot find module '../lib/tsc.js'`.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Mutex;

const EXPECT_PREFIX: &str = "// expect-error: ";

/// Held while `npx` runs, so no two installs of `typescript@5` overlap.
static NPX: Mutex<()> = Mutex::new(());

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn ts_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("reading {}: {err}", dir.display()))
        .map(|entry| entry.expect("directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "ts"))
        .collect();
    files.sort();
    files
}

fn npx_on_path() -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .flat_map(|dir| [dir.join("npx"), dir.join("npx.cmd")])
        .find(|candidate| candidate.is_file())
}

/// The `npx` to run, or `None` when the test is skipped.
fn npx() -> Option<PathBuf> {
    match npx_on_path() {
        Some(npx) => Some(npx),
        None if env::var_os("CI").is_some() => {
            panic!("npx is not on PATH and CI is set: the TypeScript compiler check cannot be skipped in CI")
        }
        None => {
            eprintln!("skipping the TypeScript compiler check: npx is not on PATH (set CI to make this a failure)");
            None
        }
    }
}

fn tsc(npx: &Path, files: &[PathBuf]) -> Output {
    // A test that panicked while holding the lock poisons it; the lock
    // guards no data, so the next run proceeds.
    let _one_at_a_time = NPX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Command::new(npx)
        .args([
            "--yes",
            "-p",
            "typescript@5",
            "tsc",
            "--noEmit",
            "--strict",
            "--pretty",
            "false",
        ])
        .args(files)
        .current_dir(crate_dir())
        .output()
        .expect("npx runs")
}

fn describe(output: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn golden_files_and_passing_fixtures_type_check() {
    let Some(npx) = npx() else { return };
    let mut files = ts_files(&crate_dir().join("tests").join("golden"));
    assert!(!files.is_empty(), "there are golden files to check");
    files.extend(ts_files(
        &crate_dir().join("tests").join("typescript").join("pass"),
    ));
    let output = tsc(&npx, &files);
    assert!(
        output.status.success(),
        "tsc rejected the golden files:\n{}",
        describe(&output)
    );
}

#[test]
fn failing_fixtures_are_rejected_with_the_expected_error() {
    let Some(npx) = npx() else { return };
    let files = ts_files(&crate_dir().join("tests").join("typescript").join("fail"));
    assert!(!files.is_empty(), "there are failing fixtures to check");
    for file in files {
        let source = fs::read_to_string(&file).expect("fixture is readable");
        let expected = source
            .lines()
            .next()
            .and_then(|line| line.strip_prefix(EXPECT_PREFIX))
            .unwrap_or_else(|| panic!("{} must start with `{EXPECT_PREFIX}<code>`", file.display()))
            .trim()
            .to_owned();
        let output = tsc(&npx, std::slice::from_ref(&file));
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !output.status.success(),
            "tsc accepted {}, which must fail:\n{}",
            file.display(),
            describe(&output)
        );
        assert!(
            stdout.contains(&format!("error {expected}:")),
            "tsc rejected {} without {expected}:\n{}",
            file.display(),
            describe(&output)
        );
    }
}
