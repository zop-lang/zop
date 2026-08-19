# Diagnostics

Compiler diagnostics are structured data rendered for people, editors, build
systems, and tests. Every stage reports the same record instead of printing its
own prose or panicking on invalid input.

> **Status:** The bootstrap returns a stage, stable code, message, and half-open
> source-byte span. Source excerpts, related spans, suggestions, and the
> versioned machine protocol remain target contracts.

## Record

Every rejecting diagnostic contains:

- the compiler stage and stable diagnostic code;
- one concise message;
- one source identity and primary byte span;
- zero or more related spans with labels;
- zero or more notes that explain the violated rule; and
- an optional suggested edit when the compiler can state one exactly.

Byte spans are authoritative. Human rendering derives line, column, and source
excerpts from the original file. Intermediate representations retain source
origins so a lowering or backend failure points to the Zop construct that
caused it rather than only naming generated code.

Diagnostic codes are compatibility identifiers for tests and tools. Message
wording may improve without changing the code. Reusing one code for a different
invariant is a breaking tool-protocol change.

## Rendering

The command-line renderer prints the code, message, source excerpt, primary
underline, related labels, and notes. Color may improve readability but cannot
carry information absent from plain text. Redirecting output therefore retains
the complete diagnostic.

Machine output uses a versioned deterministic format. It carries structured
fields rather than requiring an editor or build system to parse terminal text.
Diagnostics sort by source, primary span, stage, and code so parallel checking
does not reorder identical results.

One invalid construct may produce several independent diagnostics, but later
stages do not diagnose values invented only to recover from an earlier error.
The frontend may continue only when it has a typed error node whose semantics
cannot be mistaken for a valid program value.

## Intent-aware help

A diagnostic explains the violated rule first, then uses facts already proven
by the compiler to rank legal repairs. Relevant facts include:

- actual and expected types;
- whether an operand is still a literal or is a named value;
- the operator and requested rounding or loss policy;
- the enclosing function's error channel;
- ownership, placement, and target restrictions; and
- whether each proposed replacement parses and type-checks.

For example:

```text
error: `/` requires floating-point operands
  ratio = count / size
                ^
  `count` and `size` both have type i64

help: use `count // size` for an integer floor quotient
help: cast both operands to the intended floating type for fractional division
```

An expected result type improves the second help message:

```text
ratio: f32 = count / size

help: cast both operands to f32
note: exact casts may fail with PrecisionLoss
```

Changing `/` to `//` or inserting a rounded, saturating, or wrapping cast is a
semantic choice. The compiler may explain those alternatives but never marks
one machine-applicable without source evidence of that intent. It never suggests
a narrower floating type merely because the selected target prefers it.

A suggestion carries structured applicability for editors and automated tools.
A machine-applicable edit must have exact replacement spans, contain no
placeholder, parse, type-check, and preserve every already-stated source
contract. Alternatives that require user intent remain help messages or
non-applicable suggestions.

Suggestions do not recover compilation. Invalid source never reaches HIR merely
because the compiler found a plausible repair.

Tensor diagnostics use the core terms `axis`, `extent`, `rank`, `shape`, and
layout `mode`. Structured output carries those canonical terms even when a
framework renders its own compatibility vocabulary.

Indexing diagnostics state the source index, normalized coordinate, axis,
extent, and failed logical or storage bound. They suggest `tensor.at` only when
a recoverable failure channel is valid, and distinguish `numel()` from
`layout.cosize`. The [indexing examples](indexing.md#diagnostics) define the
minimum human-readable content.

Documentation diagnostics retain spans inside `##` comments. Unknown tags,
stale parameter or error names, unresolved symbol links, missing required
sections, placeholders, and failing examples use stable codes. Suggested tags
come only from the attached declaration and are safe to insert mechanically
only when no prose must be invented. See the [documentation verification
contract](documentation.md#binding-and-verification).

Numeric diagnostics distinguish ordinary IEEE nonfinite results, recoverable
numeric failures, and execution-domain traps. They never describe NaN as an
integer overflow result or suggest a floating cast merely to avoid a trap. When
recovery is plausible, help may name fallible, wrapping, saturating, or explicit
finite-checking operations and state their different semantics.

`--check-nonfinite` reports the first instrumented operation that produces NaN
or infinity with its source span, target, element type, and logical coordinate
when available. The message identifies the instrumentation mode so users do not
mistake the report for ordinary IEEE failure behavior.

`min` and `max` diagnostics reject mixed named numeric types without suggesting
promotion, show NaN and signed-zero semantics when relevant, and suggest an
explicit `initial` for a possibly empty tensor reduction. Slice diagnostics
state whether brackets clipped an endpoint or a named `strict=true` operation
returned `BoundsError`.

A device fault diagnostic distinguishes launch rejection from a trap after
execution begins. The latter states that the entire device context and all its
allocations are invalid, then points to the host completion boundary where the
fault became observable. It never offers a retry on another backend.

## Boundaries

Typed language failures are program values governed by the
[`fails` contract](errors.md). Runtime traps are execution events governed by
the [runtime contract](runtime.md). Neither is a compiler diagnostic.

Compiler crashes and internal verifier failures identify the compiler build and
stage. They never present themselves as errors in the user's program.

## Open decisions

- Warning and lint severity policy.
- Compiler backtrace and runtime error-trace presentation.

## Required tests

- Preserve diagnostic codes and source spans through every compiler stage.
- Render the same information with and without terminal color.
- Emit deterministic human and machine output under parallel checking.
- Keep related spans and notes structured in machine output.
- Reject malformed input without panicking or inventing a valid value.
- Point lowering and backend failures back to original Zop source.
- Prove a suggested edit matches the exact source it intends to replace.
- Rank numeric suggestions from expected type, operand type, error channel, and
  target facts.
- Never mark a lossy cast or quotient-policy change machine-applicable without
  explicit source intent.
- Apply every machine-applicable edit in a test fixture and prove the result
  parses and type-checks.
- Diagnose invalid tensor indices, ranks, slice steps, and mutable aliasing with
  canonical tensor terms and only type-correct suggestions.
- Diagnose malformed or stale documentation at its tag or content span and
  keep non-machine-authored prose outside machine-applicable edits.
- Keep NaN, typed numeric failure, CPU trap, launch rejection, device fault, and
  device loss as distinct diagnostic classes.
- State context invalidation and recovery requirements after every device fault.

## References

- [Rust compiler JSON output](https://doc.rust-lang.org/rustc/json.html)
- [Clang diagnostic display](https://clang.llvm.org/docs/UsersManual.html#controlling-diagnostics-via-command-line-flags)
- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
- [Rust compiler suggestions](https://rustc-dev-guide.rust-lang.org/diagnostics.html#suggestions)
