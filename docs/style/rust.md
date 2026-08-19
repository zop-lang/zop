# Rust style

Zop's Rust bootstrap follows the Dedalus Rust guide and the established Ruff
and uv repository conventions. `rustfmt` owns layout. Clippy owns mechanical
linting. This guide owns the project-specific documentation and compiler rules.

## Source preamble

Every Rust source and test file starts with the repository copyright and SPDX
license identifier, then a module or crate document:

```rust
// Copyright (c) 2024 Windsor Nguyen.
// SPDX-License-Identifier: Apache-2.0

//! Typed high-level intermediate representation for target-independent checks.
//!
//! The frontend produces this form. MLIR, JavaScript, and the reference
//! interpreter consume it.
```

The copyright holder and year match [`LICENSE`](../../LICENSE). Git history
records detailed authorship. Do not add per-function `@author` tags or replace
the legal header with generated contributor lists.

Module documentation must let a new reader answer three questions without
opening another file:

1. What contract does this module own?
2. Why is it separate from neighboring modules?
3. Which stage produces and consumes its values?

Files implementing a non-trivial algorithm include a short numbered preamble.
The call-graph component analysis, parser recovery, and lowering pipelines are
examples. Thin re-export modules do not need ceremonial diagrams.

## Rustdoc coverage

Every struct, enum, trait, union, type alias, and enum variant has a `///`
contract comment, regardless of visibility. Every field has a `///` comment,
including tuple fields and fields on private compiler state.

This all-fields rule is stricter than the Dedalus and Astral defaults. A field
comment must still pass the deletion test: it states units, ownership, identity,
ordering, invariants, absence semantics, or the stage that owns the value. It
must not merely repeat the field name.

```rust
/// One source function after type checking.
pub struct Function {
    /// Stable source-order identity used by calls and lowering.
    pub id: FunctionId,

    /// Declared result type shared by every successful exit.
    pub result: Type,
}
```

Public functions document their contract. Fallible functions add `# Errors`
when the error type alone does not explain the rejected cases. Functions that
can panic on caller-controlled input add `# Panics`, though production compiler
code should normally return a structured diagnostic instead.

Prefer intra-doc references such as ``[`Function`]`` over code-formatted names when
the target is linkable. `cargo doc` must pass with broken links denied.

## Comments

Comments state what is true. They explain invariants, ordering constraints,
safety arguments, target limitations, or measured tradeoffs. They do not
narrate assignments, loops, or branches.

Every `unsafe` block has a preceding `// SAFETY:` comment naming the exact type,
arity, lifetime, alignment, or foreign-function guarantee that makes the block
sound. Every lint suppression uses `#[expect(...)]` when possible and carries a
nearby justification.

Concrete examples anchor unfamiliar compiler behavior. A parser comment may
show the Zop source being recognized. An intermediate-representation comment
may show the before and after operations. Do not assume the reader knows an
acronym; spell it out on first use in each module document.

## Structure

- Keep files below 500 production lines.
- Keep functions below 70 lines.
- Keep nesting at three levels or fewer.
- Keep functions at five explicit arguments or fewer, excluding `self`.
- Put public entry points first and helpers in call order.
- Keep `mod.rs` files as declarative module maps.
- Keep imports at module scope and let `rustfmt` order them.
- Place a type next to its inherent `impl` unless a dedicated large-type file
  provides a clearer boundary.
- Default to private visibility. Use `pub(crate)` or `pub(super)` for real
  cross-module consumers.
- Prefer let chains over nested `if let` blocks.
- Avoid `panic!`, `unreachable!`, `unwrap`, and `expect` on user input.
- Add a trait only when a real second implementation exists.

Compiler stages fail closed. Unsupported syntax, semantics, intermediate
representation, or target behavior returns a typed diagnostic. No stage retries
through another implementation.

## Testing

Tests name the invariant or deliberate design choice rather than the function
under test. Use `invariant_` for safety and correctness properties and
`design_` for a policy that could reasonably differ.

Prefer the narrowest existing test entry point. Parser tests do not invoke code
generation. Lowering tests consume verified HIR or MLIR. End-to-end tests prove
only contracts that genuinely cross every stage. Keep examples small enough to
understand without unrelated standard-library knowledge.

Run focused tests while iterating, then run the complete gate:

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo doc --locked --no-deps
```

## References

- [Dedalus root style guide](https://github.com/dedalus-labs/dedalus/blob/dev/style/style.md)
- [Dedalus Rust style](https://github.com/dedalus-labs/dedalus/blob/dev/docs/src/style/rust.mdx)
- [Ruff repository guidance](https://github.com/astral-sh/ruff/blob/main/AGENTS.md)
- [uv repository guidance](https://github.com/astral-sh/uv/blob/main/AGENTS.md)
