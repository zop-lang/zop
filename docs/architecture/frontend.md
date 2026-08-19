# Frontend

The frontend turns source text into typed high-level intermediate
representation (HIR). Invalid source never reaches Multi-Level Intermediate
Representation (MLIR).

```mermaid
flowchart LR
    source["Source text"] --> lexer["Lexer"]
    lexer --> parser["Parser"]
    parser --> ast["Syntax tree"]
    ast --> resolver["Name resolution"]
    resolver --> checker["Type, shape, effect, and ownership checking"]
    checker --> hir["Typed HIR"]
```

## Implemented bootstrap

The lexer emits logical newlines plus `Indent` and `Dedent` tokens. It rejects
leading tabs, invalid dedents, unmatched delimiters, braces, and semicolons.
`#` starts a comment outside a string and continues through the physical line.
Comment-only lines do not change indentation.
The bootstrap currently discards `##` through the same ordinary-comment path;
the target documentation token and attachment rules are not implemented.
The hand-written parser handles `fn`, `kn`, comma-separated parameters,
trailing commas, expression precedence, calls, `label=value` named arguments,
assignments, final-expression returns, and the agreed error syntax. String
literals support `\\`, `\"`, `\n`, `\r`, and `\t`; other escapes are rejected.
The bootstrap does not implement interpreted or raw triple-quoted strings.

The semantic pass indexes every module function name, resolves every signature,
validates recursive signature components, then checks function bodies and
lexical locals. Forward calls resolve before the callee body is checked. Direct
and mutual recursion require explicit result types; callers outside a recursive
component do not. The body pass checks scalar expressions, stable assignment
types, function calls, argument labels, and return types. Named arguments
evaluate in source order and enter typed HIR in parameter order. Numeric
literals adopt an immediate expected type, but concrete values never promote.
Control flow, tensors, ownership, effects, and error channels stop with
structured diagnostics before HIR. Compile-time parameters are not implemented
in the bootstrap grammar yet.
The bootstrap also lacks the target language's overflow traps, compound update
assignments, and fallible numeric members; its scalar arithmetic tests currently
cover only non-overflowing executions. It still accepts concrete integer `/` as
truncating division, treats integer `%` as truncating remainder, accepts
floating `%`, and has no `//` token. Those behaviors are nonconforming with the
target [numeric contract](numerics.md).

## Stage-0 implementation policy

Rust is the bootstrap implementation, not Zop's permanent source language.
Language semantics therefore live in these contracts, typed HIR, verifiers,
and conformance tests rather than Rust-specific abstractions. The Rust compiler
remains the small, supported recovery root after self-hosting.

Stage 0 minimizes code under these rules:

- Use a maintained crate when it deletes substantial compiler code behind a
  narrow boundary.
- Pin every dependency and commit the lockfile; compilation never downloads a
  replacement implementation.
- Keep syntax, semantics, MLIR lowering, and Cranelift translation as concrete
  modules. Add a trait only after a second real implementation needs it.
- Keep `mod.rs` files as module maps, not implementation containers.
- Reject unsupported semantics with structured diagnostics instead of adding
  placeholders or fallback paths.

Logos owns raw-token recognition, Melior owns the MLIR API, and Cranelift owns
native code generation. The existing recursive-descent parser stays while it
is smaller and more precise than a replacement. Chumsky is the preferred
parser-combinator candidate when a measured spike deletes parser code without
weakening spans, recovery, compile time, or diagnostics. Framework churn is not
a self-hosting milestone.

## Declaration resolution

The production frontend parses each file into syntax once, then resolves a
selected module in three semantic phases:

1. **Names.** Index declarations, imports, and lexical scopes without checking
   signatures or bodies.
2. **Signatures.** Resolve parameter and result types, ownership modes, effects,
   error channels, generic kinds, constraints, defaults, and visibility.
3. **Bodies.** Check expressions against the resolved signature and construct
   typed HIR.

This ordering permits forward references among peer declarations without a
source-level forward-declaration syntax. A recursive function still requires
an explicit result type before its body is checked. Cycles that cannot resolve
from signatures alone are diagnostics.

The package interface hash is available after signature resolution. An editor
may resolve bodies lazily for the requested symbol and its dependencies. A
build or package check resolves every body in the selected target closure, so
unused invalid source cannot hide in a release.

Mojo 1.0's open
[declaration resolver](https://github.com/modular/modular/blob/f66d4d522c34be0a961ffac3dbfc81e30f67942e/KGEN/lib/MojoParser/DeclResolver.cpp)
validates the name-signature-body separation. Zop does not copy its parser
architecture: Mojo reparses declaration regions and emits source-level MLIR,
while Zop keeps one syntax tree and emits MLIR only after typed HIR. This
preserves one semantic input for native, browser, device, and interpreter
targets.

## Target contract

The production frontend accepts source text and a file identity. It returns
typed HIR or structured diagnostics.

Valid HIR has these properties:

- Every name resolves to one declaration.
- Every declaration reaches indexed-name, resolved-signature, and checked-body
  states in that order.
- Every expression has a concrete type.
- Every numeric literal has a concrete, representable type before HIR.
- Every string literal has one normalized UTF-8 value; interpreted and raw
  multiline forms record their exact indentation transformation.
- Every fixed-width integer operation records its trapping, wrapping,
  saturating, or fallible arithmetic contract.
- Every division records fractional or integral kind, quotient mode, failure
  policy, and floating-point profile before HIR.
- Every trap records the execution domain it terminates; a device launch records
  the context invalidated by an execution-time fault.
- Every proven or explicitly checked finite-value fact names the tensor identity
  and the mutation or escape events that invalidate it.
- Concrete integer operands never reach fractional `/`; concrete
  floating-point operands never reach `//` or integer `%`.
- Contextual typing changes literals only, never an expression that already has
  a concrete type.
- Ambiguous inference produces a diagnostic, not a dynamic value.
- Every annotation is proven or rejected.
- Every assignment pattern is irrefutable for its value type; redundant
  top-level tuple parentheses produce identical HIR.
- Every multi-iterable loop contains an explicit `zip` with one known strictness
  policy.
- Every named function parameter has an explicit type and ownership mode.
- Every exported function has explicit parameter, success, and error types.
- Tensor expressions record rank and every statically known extent.
- Tensor types preserve symbolic extent identities and equality constraints.
- Every tensor has a concrete shape before construction and keeps that shape.
- Every implicit elementwise broadcast has a statically proven trailing-axis
  mapping and an explicit zero-stride HIR view.
- Every tensor selector is normalized once against its logical axis, and every
  retained slice axis has a derived nonnegative extent and residual layout.
- Every omitted slice endpoint remains distinguishable from an explicit
  negative endpoint until step direction and axis extent are known.
- Every tensor value carries one verified CuTe-native `Engine` and one verified
  language-native `Layout`.
- Every layout has congruent shape and stride profiles plus explicit static and
  dynamic leaves.
- Every tensor access is inside its logical shape and backing storage.
- Every compiler-known search, map, reduction, and scan retains its logical
  domain, predicate or combining operation, effects, and observable order.
- Control flow is explicit.
- Every source block has a result type.
- Every basic block ends in exactly one terminator.
- Every function body yields its declared success type on every successful
  path.
- Assignment targets are typed places, distinct from computed values.
- Every local assignment preserves the binding's type and creates a new HIR
  value identity.
- Every owned value has one owner or an explicit shared representation.
- Every borrow records its owner, mutability, and valid lifetime.
- Every view records its advanced Engine, transformed Layout, and Zop borrow
  origin.
- Every write through a borrowed value traces to an exclusive `mut` boundary.
- Destruction points are explicit in HIR.
- Every storage request traces to an explicit `Mem` capability.
- Every input/output effect traces to an explicit `Io` capability.
- Every owned writer or future resolves its completion obligation.
- Every call records its callable form, ownership modes, effects, error
  channel, and target.
- Every named call evaluates arguments in source order before arranging them
  in parameter order.
- Every `known` argument is proven available during compilation.
- Every fallible result is handled, propagated through a compatible error
  channel, or preserved as a complete value.
- Every `fail with` value matches the current function's error type.
- A `kn` has no propagating language error channel, and every fallible
  kernel-local operation is handled before the kernel boundary.
- Every structured documentation block attaches to one declaration or module,
  and every documentation name binds to a valid symbol in that context.
- Every node retains its source location.
- User input cannot cause a compiler panic.

The frontend owns language semantics, including construction and verification
of the source `Layout`. It does not choose machine types, vector widths,
calling conventions, target descriptor encodings, kernel schedules, or
optimization passes. The [SIMD pass](simd.md) consumes HIR facts rather than
changing them.

## Stages

The lexer turns leading spaces into `Indent` and `Dedent` tokens. It rejects
leading tabs and dedents that do not match an enclosing block. Newlines end
expressions except inside an explicit delimiter. Blank lines produce no layout
token. Longest-token matching recognizes `//=`, `//`, and `/` separately;
`//` never starts a comment.

Lexical analysis distinguishes ordinary `#` comments from line-leading `##`
documentation while preserving every physical newline for layout. The parser
attaches documentation before ordinary comments are discarded. Documentation
name binding and example extraction follow the
[documentation contract](documentation.md); they never alter executable HIR.

The grammar has no semicolon or brace-delimited block. The parser builds syntax;
it does not recover by inventing values. Name resolution assigns stable symbol
identities. Type, shape, effect, and ownership checking rejects ambiguity before
HIR construction. Every rejection follows the [diagnostic contract](diagnostics.md).

Binding-time checking treats `known` as part of a parameter's function type.
It rejects runtime expressions at those call positions. Symbolic tensor
extents remain shape facts unless an interface separately requires their
values during compilation.

A `Catch` syntax node contains a required pattern and handler block. The parser
rejects bare `catch` instead of emitting a node with a missing pattern.

A `Fail` syntax node contains a required error value. Type checking rejects it
inside a function without a compatible error channel. HIR lowers it to an
error-return terminator after local destruction.

Inference and annotation checking produce the same HIR facts. The compiler
never changes a static value into a dynamic value or inserts a numeric
conversion to make a program compile.

Inference is bidirectional and local to the current declaration. Named
parameters provide input types; expected types flow into literals, closures,
calls, and block results; synthesized types flow outward. Local bindings are
not generalized implicitly. When user-defined generics are introduced, their
declarations will be explicit even when arguments infer at call sites.

The parser distinguishes member selection from invocation without consulting
types. Name resolution later classifies fields, methods, module functions, and
callable fields according to the [callables contract](callables.md).

Name resolution also maps argument labels to parameters. It rejects positional
arguments after a named argument plus missing, unknown, or duplicate labels.

Type checking preserves the error channel as part of the function type. Only
private functions and closures may infer it after an explicit `or fails`
clause. Error handling lowers to explicit control flow according to the
[error contract](errors.md), never exception unwinding.

Tensor selection follows the [indexing contract](indexing.md). The parser
preserves integers, slices, commas, omitted endpoints, and signed steps without
guessing bounds semantics. Type checking determines result rank, normalizes
static coordinates, derives the residual `Layout`, attaches origin and bounds
obligations, and rejects advanced bracket selectors. Brackets record clipping;
the named slice operation records its known `strict` policy. No backend chooses
or inherits endpoint behavior.

The frontend lowers a function's final expression to the same return terminator
as an explicit `return`. `return` exits the function rather than the nearest
source block.

Resolution keeps type names, globals, functions, and lexical locals in
separate contexts. A declaration inside one branch or block cannot escape its
scope. Stable symbol identities come from the module, not a process-global
counter, so compilation order does not change the IR.

Diagnostics name the failed rule and point to the source range that violated
it. The frontend supplies actual and expected types, literal status, callable
error channels, placement, and target restrictions so the diagnostic layer can
rank only legal repairs. Later stages may attach more context, but they do not
reinterpret an invalid frontend result. See the
[intent-aware diagnostic contract](diagnostics.md#intent-aware-help).

## Required inference tests

- Infer a uniquely constrained local binding without an annotation.
- Reject an ambiguous local with every conflicting constraint in the diagnostic.
- Require types and ownership modes on named function parameters.
- Require explicit success and error types on exported functions.
- Require a return type before checking a recursive function body.
- Resolve forward peer references from indexed names and signatures without
  checking a body early.
- Keep an exported signature hash unchanged when only a private body changes.
- Infer a closure parameter only from one expected callable type.
- Reject implicit polymorphic generalization of a local binding.
- Once generics exist, infer call-site arguments for an explicitly declared
  generic.
- Produce identical HIR facts from equivalent inferred and annotated source.
- Prove every implicit broadcast from trailing extents and reject a relation
  that depends on coincidental runtime sizes.
- Preserve `axis`, extent, rank, shape, and hierarchical layout modes as
  distinct HIR facts.
- Preserve omitted slice endpoints until directional normalization, derive
  every residual layout, and reject step zero and excess selectors.
- Prove static and guarded dynamic indexing share one normalization and bounds
  predicate without compiler-integer overflow.
- Preserve `##` blocks, attach them at the correct indentation, and bind every
  structured documentation name without changing executable semantics.
- Normalize interpreted and raw triple-quoted strings from their closing
  delimiter margin and reject ambiguous or unterminated delimiters.
- Destructure bare and parenthesized top-level tuple patterns identically,
  preserve nested tuple/product structure, and reject tensor-element
  destructuring.
- Reject implicit multi-iterable loops; lower `zip` with `strict=false` to
  shortest exhaustion and `strict=true` to a proven or trapping equality check.
