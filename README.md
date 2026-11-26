# wesley-typescript

A [Wesley](https://github.com/flyingrobots/wesley) target that emits TypeScript: declarations, operation bindings, and little-endian binary codecs.

Wesley compiles GraphQL SDL into a versioned intermediate representation (IR). It does not emit code itself: every output language is its own repository, run by Wesley as an external target over the IR. The decision is recorded in flyingrobots/parley#116, and the move of the emitters out of Wesley is flyingrobots/wesley#882.

## Status

Not started. This repository holds the plan and its rules; the first code waits on the versioned IR contract (flyingrobots/wesley#881) and on Wesley running external targets (flyingrobots/wesley#880).

## Where it comes from

It takes over `crates/wesley-emit-typescript` from Wesley at `bfff0f9a` (0.3.0-alpha.4): declarations, operation bindings and LE-binary codecs. The current emitter prints a private model with a hand-written printer and embeds its codec runtime as a string literal (audit on flyingrobots/wesley#882). This repository builds the same output with `ts.factory` and prints it with the compiler's own printer, and must reproduce today's output for every Wesley fixture before the built-in emitter is deleted.

## How it generates code

The output is built as a syntax tree of the target language and printed by that language's printer. There is no string templating and no regex anywhere in the generator.

| Concern | Choice |
| --- | --- |
| Written in | TypeScript, run with Bun |
| Syntax tree | the TypeScript compiler API (`ts.factory`) |
| Printer | `ts.createPrinter` |
| Checked by | the TypeScript compiler itself (`ts.createProgram`, no emit, strict) on every golden file |

Rules every change keeps:

- **No code as strings.** A generated identifier, literal or comment enters the output as a node of the tree, never by concatenating source text. Escaping is the printer's job.
- **Golden output is checked by the real toolchain.** Every golden file is compiled or type-checked in CI by the toolchain named above, not only compared byte for byte.
- **Deterministic to the byte.** The same IR and options give the same bytes on every machine. CI runs the generator twice and compares.
- **The IR is the only input.** The target reads the published IR (flyingrobots/wesley#881) and never depends on Wesley's internals.

## License

Apache 2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
