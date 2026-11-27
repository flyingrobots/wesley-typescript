# Releasing

This repository publishes one crate to crates.io: `wesley-typescript-syntax`. `wesley-emit-typescript` has `publish = false` and is not released from here; its next release belongs to flyingrobots/wesley#882. The release is driven by [`.github/workflows/release.yml`](.github/workflows/release.yml), and nobody runs `cargo publish` by hand.

## 1. Merge the release-prep pull request

The release-prep pull request sets the crate's `version` in `crates/wesley-typescript-syntax/Cargo.toml`, turns `## Unreleased` in [CHANGELOG.md](CHANGELOG.md) into `## [<version>] - <date>` with an empty `## Unreleased` above it, and passes CI, which includes `cargo publish --dry-run --locked -p wesley-typescript-syntax`. Merge it with a merge commit.

## 2. Tag main with an annotated tag

Update main locally, then create an annotated tag named `v` followed by the crate version, on a commit of main, and push it:

```sh
git switch main
git pull --ff-only
git tag -a v0.1.0-alpha.1 -m "wesley-typescript-syntax 0.1.0-alpha.1"
git push origin v0.1.0-alpha.1
```

Pushing the tag starts the release workflow. It can also be started with `workflow_dispatch`, but only on a tag; a run on a branch is refused.

The `gate` job refuses a lightweight tag, a tag whose commit is not an ancestor of `origin/main`, a tag that does not equal `v` plus the crate's `version` from `cargo metadata`, and a version with no `## [<version>]` heading in CHANGELOG.md. It then runs `cargo fmt --check`, `cargo clippy -D warnings`, the full test suite with Node set up so the `tsc` checks run, and `cargo publish --dry-run --locked -p wesley-typescript-syntax`.

## 3. Approve the crates-io deployment

The `publish` job runs in the `crates-io` GitHub environment, which requires the repository owner's approval and accepts only `v*` tags. Open the workflow run, review the gate's result, and approve the deployment. The job publishes with `cargo publish --locked -p wesley-typescript-syntax` and the `CARGO_REGISTRY_TOKEN` secret. If that version is already on crates.io it skips the upload, so a re-run after a later failure is safe.

## 4. What verify checks

The `verify` job polls `cargo info wesley-typescript-syntax@<version> --registry crates-io` up to 20 times, 15 seconds apart, until crates.io serves the version. It then creates a scratch crate that depends on `wesley-typescript-syntax = "=<version>"` from crates.io, prints a small module with an interface, a union type alias, a readonly property, an optional property with a key that is not an identifier, and JSDoc, and type-checks the output with `npx --yes -p typescript@5 tsc --noEmit --strict`. A failure here means the published crate does not build or prints TypeScript the compiler rejects: yank it (below) and release a fixed version.

The last job, `github-release`, creates the GitHub release for the tag with `gh release create --verify-tag`, using that version's CHANGELOG.md section as the notes, and marks it as a prerelease when the version has a `-` suffix. It is the only job with `contents: write`.

## Yanking a release

A published version cannot be deleted or overwritten. If it is broken, yank it so that new lockfiles do not select it, and release a fixed version with a new number:

```sh
cargo yank --version 0.1.0-alpha.1 wesley-typescript-syntax
```

`cargo yank --undo --version 0.1.0-alpha.1 wesley-typescript-syntax` reverses a yank. Record the yank and its reason in CHANGELOG.md, and edit the GitHub release to say the version is yanked; do not delete the tag.
