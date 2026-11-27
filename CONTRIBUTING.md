# Contributing

## Before you start

Every pull request names a GitHub issue. If there is no issue for the change, open one first; it is where the problem and the chosen approach are agreed.

- Use `Closes #123` for an issue the pull request fully resolves.
- Use `Refs #123` for one it only advances, and say what remains.

## How changes are made

- Enable the hooks once: `git config --local core.hooksPath scripts/hooks`. `pre-commit` runs `cargo fmt --check` and `cargo clippy -D warnings`; `pre-push` runs `cargo test`.
- Write a failing test first, then the change that makes it pass.
- Build output as a syntax tree and print it with the printer named in the [README](README.md). A pull request that writes generated code as strings is not merged.
- A new golden file is checked by the real toolchain in CI, as the README says.
- Commit with a new commit; do not amend, squash or rebase published history.

## Releasing

Releases are cut from an annotated tag on main by the release workflow, never by running `cargo publish` by hand. [RELEASING.md](RELEASING.md) describes the procedure.

## Licensing

By contributing you agree that your contribution is licensed under the Apache License 2.0, as the rest of the repository is.
