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
The hand-written parser handles `fn`, `kn`, comma-separated parameters,
trailing commas, expression precedence, calls, `label=value` named arguments,
assignments, final-expression returns, and the agreed error syntax. String
literals support `\\`, `\"`, `\n`, `\r`, and `\t`; other escapes are rejected.

The semantic pass currently resolves module functions and lexical locals. It
checks scalar expressions, stable assignment types, function calls, argument
labels, and return types. Named arguments evaluate in source order and enter
typed HIR in parameter order. Numeric literals adopt an immediate expected
type, but concrete values never promote. Control flow, tensors, ownership,
effects, and error channels stop with structured diagnostics before HIR.
Compile-time parameters are not implemented in the bootstrap grammar yet.

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

## Target contract

The production frontend accepts source text and a file identity. It returns
typed HIR or structured diagnostics.

Valid HIR has these properties:

- Every name resolves to one declaration.
- Every expression has a concrete type.
- Every numeric literal has a concrete, representable type before HIR.
- Contextual typing changes literals only, never an expression that already has
  a concrete type.
- Ambiguous inference produces a diagnostic, not a dynamic value.
- Every annotation is proven or rejected.
- Every named function parameter has an explicit type and ownership mode.
- Every exported function has explicit parameter, success, and error types.
- Every runtime type, shape, or placement check introduced by dynamic code
  traces to an explicit dynamic source construct.
- Tensor expressions record rank and every statically known dimension.
- Tensor types preserve symbolic dimension identities and equality constraints.
- Every tensor has a concrete shape before construction and keeps that shape.
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
- Every node retains its source location.
- User input cannot cause a compiler panic.

The frontend owns language semantics. It does not choose machine types,
calling conventions, memory layouts, or optimization passes.

## Stages

The lexer turns leading spaces into `Indent` and `Dedent` tokens. It rejects
leading tabs and dedents that do not match an enclosing block. Newlines end
expressions except inside an explicit delimiter. Blank lines produce no layout
token.

The grammar has no semicolon or brace-delimited block. The parser builds syntax;
it does not recover by inventing values. Name resolution assigns stable symbol
identities. Type, shape, effect, and ownership checking rejects ambiguity before
HIR construction.

Binding-time checking treats `known` as part of a parameter's function type.
It rejects runtime expressions at those call positions. Symbolic tensor
dimensions remain shape facts unless an interface separately requires their
values during compilation.

A `Catch` syntax node contains a required pattern and handler block. The parser
rejects bare `catch` instead of emitting a node with a missing pattern.

A `Fail` syntax node contains a required error value. Type checking rejects it
inside a function without a compatible error channel. HIR lowers it to an
error-return terminator after local destruction.

Inference and annotation checking produce the same HIR facts. Dynamic source
constructs instead produce explicit runtime values, checks, and typed failure
paths. The compiler never changes a static value into a dynamic value to make a
program compile.

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

The frontend lowers a function's final expression to the same return terminator
as an explicit `return`. `return` exits the function rather than the nearest
source block.

Resolution keeps type names, globals, functions, and lexical locals in
separate contexts. A declaration inside one branch or block cannot escape its
scope. Stable symbol identities come from the module, not a process-global
counter, so compilation order does not change the IR.

Diagnostics name the failed rule and point to the source range that violated
it. Later stages may attach more context, but they do not reinterpret an
invalid frontend result.

## Required inference tests

- Infer a uniquely constrained local binding without an annotation.
- Reject an ambiguous local with every conflicting constraint in the diagnostic.
- Require types and ownership modes on named function parameters.
- Require explicit success and error types on exported functions.
- Require a return type before checking a recursive function body.
- Infer a closure parameter only from one expected callable type.
- Reject implicit polymorphic generalization of a local binding.
- Once generics exist, infer call-site arguments for an explicitly declared
  generic.
- Produce identical HIR facts from equivalent inferred and annotated source.
