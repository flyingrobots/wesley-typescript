# wesley-typescript-syntax

TypeScript declarations built as an [swc](https://swc.rs) syntax tree and printed by swc's code generator. It knows nothing about GraphQL or Wesley's directives; it is the TypeScript tooling that Wesley targets, such as [wesley-parley](https://github.com/flyingrobots/wesley-parley), render through.

```rust
use wesley_typescript_syntax::{print_module, Declaration, Module, Property, TsType};

let module = Module {
    doc: Some("Generated. Do not edit.".to_owned()),
    declarations: vec![Declaration::Interface {
        name: "Started".to_owned(),
        doc: Some("A run started.".to_owned()),
        properties: vec![Property {
            name: "event".to_owned(),
            optional: false,
            readonly: true,
            ty: TsType::StringLiteral("started".to_owned()),
            doc: None,
        }],
    }],
};
let source = print_module(&module)?;
```

`print_module` covers exported interfaces (`export interface`) and type aliases (`export type`) over `string`, `number`, `boolean`, `unknown`, `null`, string, number and boolean literal types, named types, arrays, unions, inline object types and `Record<string, T>`; optional and readonly properties; property keys that are not identifiers, printed as string-literal keys; and JSDoc on the module, each declaration and each property.

The tree is `swc_ecma_ast` and the printer is `swc_ecma_codegen`. Docs become JSDoc block comments in swc's comment store, keyed by synthetic positions on the documented nodes, so the printer places them; quoting and escaping are the printer's. Invalid input (an empty name, a declaration name that is not a usable identifier, a repeated property or declaration name, an empty union, a number literal that is not a safe integer) is an `Error`, never a panic. The output is deterministic to the byte and ends with one newline.

Every golden file under `tests/golden/` is type-checked by `tsc --noEmit --strict`.

Pre-1.0: the API and the printed layout may change between releases. Apache-2.0.
