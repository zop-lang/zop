# Design lineage

Bedrock composes proven ideas from systems languages and compiler projects.
Credit matters because the architecture is an adaptation, not a spontaneous
invention.

The composition is better for Bedrock's stated goals than adopting any one
influence wholesale. That is a design claim to prove, not a claim that an
unfinished language already outperforms mature tools.

## Original Bedrock notes

The original [`whitepaper.txt`](../history/whitepaper.txt) supplied the intent:
native performance, fast interactive compilation, tensor-first syntax,
Multi-Level Intermediate Representation (MLIR), first-class functions,
explicit pointers and mutation, and errors as values.

The archive remains historical. Current architecture documents decide which
ideas became contracts and which remain experiments.

## Rust

Bedrock draws its core safety invariants from Rust:

- One owner for a resource-owning value.
- Moves instead of implicit deep copies.
- Shared immutable borrows or one exclusive mutable borrow.
- References that cannot outlive their owner.
- Deterministic destruction and an explicit unsafe boundary.

Rust proves that these guarantees can protect production systems without a
garbage collector. Bedrock does not copy Rust's full language surface, trait
system, or crate-level optimization boundaries.

Sources: [ownership](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html),
[borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html),
and [reference cycles](https://doc.rust-lang.org/stable/book/ch15-06-reference-cycles.html).

## Mojo

Mojo shows how ownership can fit a Python-like systems language. Bedrock draws
from its immutable, mutable, consuming, and output argument conventions plus
compiler-tracked origins. These ideas reduce lifetime syntax in ordinary code.

Bedrock keeps its own source grammar, `fn` and `kn` target boundary, high-level
intermediate representation (HIR), and backend composition.

Sources: [ownership](https://docs.modular.com/mojo/manual/values/ownership) and
[origins and lifetimes](https://docs.modular.com/mojo/manual/values/lifetimes).

## Frontend implementation

[Logos](https://docs.rs/logos/0.16.1/logos/) supplies deterministic raw-token
recognition. Bedrock keeps indentation and logical newlines in a separate layout
pass, following the boundary used by
[Ruff's Python lexer](https://github.com/astral-sh/ruff/tree/885bf665fb8962571270356da80be080c75fe191/crates/ruff_python_parser).

[rust-analyzer's parser](https://github.com/rust-lang/rust-analyzer/tree/b821f090608b98263d2a82d65bfb09d596380b26/crates/parser)
informs the separation between tokens, syntax, and semantic analysis plus the
rule that error recovery must always consume input. Bedrock does not adopt its
event stream or Rowan tree until the grammar needs them.

Mojo's public language documentation informs the source contract. Modular's
compiler parser is not public, so Bedrock does not claim it as an implementation
reference.

## OCaml and Haskell

OCaml demonstrates expression-oriented programming and constant-stack tail
recursion. Haskell demonstrates how recursive definitions can remain the
ordinary language of functional algorithms.

Bedrock is strict and plans to make proper tail calls a language guarantee in a
future milestone. The guarantee will not depend on an optimization profile.
This is stronger than Glasgow Haskell Compiler (GHC) loopification, which
targets saturated self-recursive tail calls under optimization.

Sources: [OCaml tail recursion](https://ocaml.org/docs/loops-recursion),
[OCaml tail-call assertions](https://ocaml.org/manual/5.5/attributes.html), and
[GHC loopification](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/using-optimisation.html#ghc-flag--floopification).

## Go

Go demonstrates the value of keeping errors ordinary and control flow visible.
Bedrock adopts those goals, not the conventional final `(value, error)` return
or optional checking.

Bedrock adds a typed error channel, mandatory handling, and explicit
propagation. Exported functions name one domain error; private functions may
request inference. Multiple success values remain one success tuple.

Sources: [Errors are values](https://go.dev/blog/errors-are-values) and
[Error handling and Go](https://go.dev/blog/error-handling-and-go).

## JAX

JAX demonstrates composable program transformations for differentiation,
vectorization, and compilation. It also shows why transformed code needs
explicit state and controlled effects.

Bedrock adopts compiler-owned transformations without requiring a globally
functional programming style. The effect checker permits local mutation that
can be converted to value flow and rejects external effects inside a
differentiated region.

Sources: [JAX transformations](https://docs.jax.dev/en/latest/key-concepts.html)
and [stateful computations](https://docs.jax.dev/en/latest/stateful-computations.html).

## Shape polymorphism

Futhark and Dex demonstrate statically checked symbolic array dimensions. JAX
demonstrates symbolic shape relationships without requiring source-level
dependent types.

Bedrock adopts exact symbolic dimension identities for tensor signatures. A
dimension such as `n` expresses a relationship and does not force code
specialization. General symbolic arithmetic and an unrestricted constraint
solver are not part of the initial contract.

Sources: [Futhark size types](https://futhark.readthedocs.io/en/v0.26.3/glossary.html),
[Dex shape safety](https://google-research.github.io/dex-lang/examples/tutorial.html),
and [JAX shape polymorphism](https://docs.jax.dev/en/latest/export/shape_poly.html).

## Numeric literals

Rust demonstrates contextual typing for unsuffixed literals. JAX demonstrates
the readability benefit of allowing scalar literals to adopt an array's
precision. NumPy's promotion redesign demonstrates why a literal's numerical
value must not select the result type.

Bedrock adopts contextual numeric literals, then adds stricter boundaries.
Only literals may adopt an expected type. Concrete values never promote, and
lossy integer conversion is a compile error.

Sources: [Rust literal inference](https://doc.rust-lang.org/reference/expressions/literal-expr.html#integer-literal-expressions),
[JAX type promotion](https://docs.jax.dev/en/latest/jep/9407-type-promotion.html),
and [NumPy NEP 50](https://numpy.org/neps/nep-0050-scalar-promotion.html).

## Burn

Burn demonstrates backend-independent tensor APIs, explicit gradient storage,
activity tracking, and gradient checkpointing. Its move away from a backend
generic on every public tensor also validates keeping execution machinery out
of user-facing tensor types.

Bedrock moves these facts into compiler-checked HIR. It does not adopt Rust
trait towers, runtime backend mismatches, or a dynamic graph as the only
autodiff model.

Sources: [backend-generic removal](https://github.com/tracel-ai/burn/pull/4717),
[autodiff](https://burn.dev/books/burn/building-blocks/autodiff.html), and
[gradient storage](https://github.com/tracel-ai/burn/blob/b6e27bdca620fbbc15e524c7088a7711f1a999f1/crates/burn-autodiff/src/grads.rs).

## Zig and Andrew Kelley

Zig demonstrates the value of explicit allocators, explicit input/output (I/O)
capabilities, explicit resource flow, fast compilation, and optimizer-visible
interfaces.

Zig's allocator passing directly inspires Bedrock's `Mem` capability. Bedrock
names the object for the authority it carries; `alloc` remains one operation in
the `Mem` API.

Zig's `comptime` demonstrates one language for runtime and compile-time
evaluation. Bedrock adopts the smaller requirement that selected parameters be
available during compilation. The source spelling is `name: known Type`.
Bedrock does not initially adopt arbitrary compile-time reflection, type
generation, or mandatory specialization of symbolic dimensions.

Andrew Kelley's
[Don't Forget to Flush](https://www.youtube.com/watch?v=f30PceqQWko&t=1723s)
showed that a hot buffered path should manipulate concrete state directly.
Only the cold refill or flush boundary should require a runtime-known function
pointer. His follow-up connected Rust's inferior assembly in that example to
crate visibility and devirtualization, not to LLVM or memory safety.

Bedrock adopts that interface rule while adding checked ownership. It does not
adopt unchecked manual memory management as the safe-language default.

Zig 0.16 also requires an explicit `Io` instance for operations that may block
or introduce nondeterminism. Bedrock adopts that capability boundary and uses
ownership to enforce flush, close, await, and cancellation obligations.

Sources: [Zig `comptime`](https://ziglang.org/documentation/master/#comptime),
[Andrew's follow-up](https://ziggit.dev/t/systems-distributed-dont-forget-to-flush/11431),
and [Zig I/O as an Interface](https://ziglang.org/download/0.16.0/release-notes.html#io-as-an-interface).

## MLIR

MLIR supplies tensor semantics, shape-aware transformations, explicit dialect
boundaries, bufferization, and ownership-based buffer deallocation.

[Melior](https://docs.rs/melior/0.27.4/melior/) supplies the Rust bindings. It
keeps verified MLIR in memory between Bedrock lowering and backend translation.

MLIR is the tensor optimizer, not Bedrock's general central processing unit
(CPU) code generator. Bedrock stops before LLVM intermediate representation on
the CPU path and translates its strict base boundary to Cranelift intermediate
representation.

Sources: [bufferization](https://mlir.llvm.org/docs/Bufferization/) and
[ownership-based deallocation](https://mlir.llvm.org/docs/OwnershipBasedBufferDeallocation/).

## Cranelift

Cranelift supplies fast native CPU code generation for development,
just-in-time (JIT), and ahead-of-time (AOT) builds. It keeps the edit-run loop
independent from LLVM's production optimization cost.

Bedrock supplies the source semantics, HIR, tensor optimization, and ownership
facts that Cranelift does not define.

Sources: [Cranelift frontend](https://docs.rs/cranelift-frontend/latest/cranelift_frontend/)
and [object backend](https://docs.rs/cranelift-object/latest/cranelift_object/).

## CUTLASS and CuTe IR

NVIDIA's [CuTe IR contribution](https://github.com/NVIDIA/cutlass/pull/3426)
supplies hierarchical device-layout algebra and a strict lowering boundary to
upstream MLIR dialects and NVIDIA Virtual Machine (NVVM) operations.

This is Bedrock's graphics processing unit (GPU) path.

Bedrock should consume and co-develop this work upstream. Bedrock still owns
language semantics, kernel extraction, effects, and the host runtime.

## COS 320

The COS 320 compiler assignments supplied a compact reference architecture:
typed IR, source spans, separate contexts, definite-return analysis, a semantic
interpreter, application binary interface (ABI) tests, dataflow analyses, and
correctness-before-quality gates.

Bedrock adopts those testing and boundary patterns. It does not adopt the
LLVMlite instruction set, hand-written x86 backend, global symbol generator,
or pedagogical error handling.

Source: [LLVMlite reference interpreter](https://github.com/cos320/hw2-llvmlite-jeff-windsor/blob/master/lib/ll/llinterp.ml).

## Standard-library layering

Rust separates a platform-independent `core`, allocation-backed collections,
and hosted standard-library services. Zig demonstrates explicit allocation and
I/O capabilities inside a cross-platform standard library. Go demonstrates the
ergonomic value and long-term compatibility cost of a broad standard library.

Bedrock adopts a small mandatory core, capability-aware standard modules, and
independently versioned official packages. Universal algorithms such as binary
search belong in `core`; neural-network frameworks do not.

Sources: [Rust `core`](https://doc.rust-lang.org/core/),
[Rust `alloc`](https://doc.rust-lang.org/stable/alloc/),
[Zig memory](https://ziglang.org/documentation/master/#Memory), and
[Go standard library](https://pkg.go.dev/std).

## Packages and builds

Cargo demonstrates manifest and lockfile ergonomics. Go modules demonstrate
single-version resolution, checksums, and vendoring. Zig demonstrates
content-addressed package sources and a concurrent cross-target build graph.

Bedrock adopts those properties with a stricter boundary: dependency commands
may use the network, but compilation is hermetic. Packages cannot run
unrestricted build scripts, and mission builds can reconstruct the complete
toolchain from vendored immutable inputs.

Sources: [Cargo lockfiles](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html),
[Go modules](https://go.dev/ref/mod),
[Zig package hashing](https://ziglang.org/download/0.16.0/release-notes.html#build-system),
and [Zig build system](https://ziglang.org/learn/build-system/).

## Why the composition fits Bedrock

No individual influence satisfies all Bedrock goals:

- Rust supplies safety, but not a tensor-native CPU and GPU language design.
- Zig supplies explicit systems control and interface transparency, but not
  checked ownership.
- Mojo validates ergonomic ownership for a Python-like surface, but Bedrock
  chooses a different target and compiler boundary.
- Go supplies visible error values, but not a distinct typed error channel with
  mandatory handling.
- JAX supplies composable numerical transformations, but relies on Python
  tracing and restricted effects.
- Burn supplies a portable Rust tensor framework, but carries framework and
  dispatch machinery that a language compiler can make implicit.
- MLIR supplies tensor and GPU transformations, but not a source language or a
  fast general CPU backend.
- Cranelift supplies fast CPU code generation, but not tensor semantics or GPU
  lowering.
- CuTe IR supplies device layouts, but not the complete language, runtime, or
  tensor-compute stack.
- COS 320 supplies an executable semantic oracle, but not a production
  language architecture.

Bedrock's distinctive choice is where facts live. Source states intent. Typed
HIR stores the proof. Lowering consumes explicit facts without exposing them as
generic parameters or runtime guesses.

Bedrock composes the influences into one contract:

1. Inferred proofs, optional checked annotations, and explicit dynamic checks.
2. Symbolic tensor dimensions without mandatory code specialization.
3. Pure compile-time parameters only where generated code needs their values.
4. Parameter-mode borrowing with first-class origin-tracked tensor views.
5. Compiler-generated autodiff with explicit gradient values.
6. Checked ownership without a mandatory garbage collector.
7. Explicit `Mem`, `Io`, placement, mutation, ownership transfer, and errors.
8. MLIR tensor semantics, buffer reuse, and GPU lowering.
9. Cranelift development speed and native CPU artifacts.
10. An independent interpreter that checks every backend.
11. A layered standard library with correct universal algorithms.
12. Hermetic packages that rebuild from immutable offline inputs.

The boundaries are the advantage. Each component owns the problem it solves,
and no component silently substitutes for another when it fails.
