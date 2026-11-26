# wesley-typescript

A [Wesley](https://github.com/flyingrobots/wesley) target that emits TypeScript: declarations, operation bindings, and little-endian binary codecs.

Wesley compiles GraphQL SDL into an intermediate representation (IR), and does nothing else: it attaches no meaning to any directive and runs no targets. Every output language is its own repository, a standalone program and library that reads the IR (`wesley schema lower --json`). This one is a **generic** target: it understands no directives and emits plain GraphQL, and it is the language tooling other targets (such as [wesley-parley](https://github.com/flyingrobots/wesley-parley)) render through. The decisions are recorded in flyingrobots/parley#116 and flyingrobots/wesley#882.

## Status

`crates/wesley-emit-typescript` is Wesley's TypeScript emitter, moved here with its history from flyingrobots/wesley at `bfff0f9a` (0.3.0-alpha.4) rather than rewritten. Its tests pass here against the published `wesley-core` 0.3.0-alpha.4. Wesley keeps its own copy until flyingrobots/wesley#882 removes it.

The emitter does not yet meet the rules below: it prints a private model with a hand-written printer and embeds its codec runtime as a string literal (audit on flyingrobots/wesley#882). Moving its printing onto a real TypeScript syntax tree is issue #1.

## How it generates code

The output is built as a syntax tree of the target language and printed by that language's printer. There is no string templating and no regex anywhere in the generator.

| Concern | Choice |
| --- | --- |
| Written in | Rust (stable) |
| Syntax tree | `oxc_ast`, built with `oxc_ast::AstBuilder` |
| Printer | `oxc_codegen` |
| Checked by | `tsc --noEmit --strict` on every golden file |

Rules every change keeps:

- **No code as strings.** A generated identifier, literal or comment enters the output as a node of the tree, never by concatenating source text. Escaping is the printer's job.
- **Golden output is checked by the real toolchain.** Every golden file is compiled or type-checked in CI by the toolchain named above, not only compared byte for byte.
- **Deterministic to the byte.** The same IR and options give the same bytes on every machine. CI runs the generator twice and compares.
- **The IR is the only input.** The target reads the published IR (flyingrobots/wesley#881) and never depends on Wesley's internals.

## License

Apache 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
