# Changelog

All notable changes to this project are recorded here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.1.0-alpha.1] - 2026-11-26

The first crates.io release of `wesley-typescript-syntax`. `wesley-emit-typescript` is not released from this repository yet.

### Added

- A release workflow (`.github/workflows/release.yml`) that publishes `wesley-typescript-syntax` from an annotated `v*` tag on main after a gate, an approval in the `crates-io` environment, and a check that the published crate builds and prints TypeScript that `tsc --strict` accepts; it then creates the GitHub release from this changelog. [RELEASING.md](RELEASING.md) describes the procedure.
- `wesley-typescript-syntax` carries its own LICENSE and NOTICE, so both travel in the published crate, and a test keeps them identical to the repository's. Its manifest names `rust-version = "1.88"`.

- `crates/wesley-typescript-syntax`, the generic TypeScript syntax library: `print_module` builds exported interfaces and type aliases as an `swc_ecma_ast` tree, attaches JSDoc through swc's comment store, and prints with `swc_ecma_codegen`; `is_identifier_name` decides when a property key must be a string literal. Invalid input is an `Error`. Golden files are type-checked by `tsc --noEmit --strict`, and a test rejects TypeScript source in the crate's string literals (flyingrobots/wesley-parley#1).
- Declaration names and `TsType::Ref` names refuse the words TypeScript reads as type keywords or operators in a type position (`keyof`, `infer`, `readonly`, `unique`, `asserts`, `is`, `intrinsic` and the predefined type names), which would print invalid source, and refuse `Record`, which `TsType::Record` prints and a declaration would shadow (review of #3).
- CI sets up Node 22 so the TypeScript compiler check runs.
- `crates/wesley-emit-typescript`, Wesley's existing emitter, moved from flyingrobots/wesley at `bfff0f9a` (0.3.0-alpha.4) with its history and the fixtures its tests read. It depends on the published `wesley-core` and `wesley-emit-codec` 0.3.0-alpha.4 (flyingrobots/wesley#882).
- The repository, its license, and the rules for how the target generates code (flyingrobots/parley#116, flyingrobots/wesley#882).

### Changed

- `wesley-typescript-syntax` is versioned `0.1.0-alpha.1`.
- `wesley-emit-typescript` has `publish = false`: its next release belongs to flyingrobots/wesley#882.
- CI pins every action by commit SHA and runs `cargo publish --dry-run` for `wesley-typescript-syntax`.
- The syntax tree and printer named in the README are swc's (`swc_ecma_ast`, `swc_ecma_codegen`) rather than oxc's, because swc prints JSDoc comments attached to synthetic nodes.
