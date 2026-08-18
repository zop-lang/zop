# Design lineage

Zop composes proven ideas from systems languages and compiler projects.
Credit matters because the architecture is an adaptation, not a spontaneous
invention.

The composition is better for Zop's stated goals than adopting any one
influence wholesale. That is a design claim to prove, not a claim that an
unfinished language already outperforms mature tools.

## Original notes

The original [`whitepaper.txt`](../history/whitepaper.txt) supplied the intent:
native performance, fast interactive compilation, tensor-first syntax,
Multi-Level Intermediate Representation (MLIR), first-class functions,
explicit pointers and mutation, and errors as values.

The archive remains historical. Current architecture documents decide which
ideas became contracts and which remain experiments.

## Rust

Zop draws its core safety invariants from Rust:

- One owner for a resource-owning value.
- Moves instead of implicit deep copies.
- Shared immutable borrows or one exclusive mutable borrow.
- References that cannot outlive their owner.
- Deterministic destruction and an explicit unsafe boundary.

Rust proves that these guarantees can protect production systems without a
garbage collector. Zop does not copy Rust's full language surface, trait
system, or crate-level optimization boundaries.

Sources: [ownership](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html),
[borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html),
and [reference cycles](https://doc.rust-lang.org/stable/book/ch15-06-reference-cycles.html).

## Mojo

Mojo shows how ownership can fit a Python-like systems language. Zop draws
from its immutable, mutable, consuming, and output argument conventions plus
compiler-tracked origins. These ideas reduce lifetime syntax in ordinary code.

Zop keeps its own source grammar, `fn` and `kn` target boundary, high-level
intermediate representation (HIR), and backend composition.

Sources: [ownership](https://docs.modular.com/mojo/manual/values/ownership) and
[origins and lifetimes](https://docs.modular.com/mojo/manual/values/lifetimes).

## Frontend implementation

[Logos](https://docs.rs/logos/0.16.1/logos/) supplies deterministic raw-token
recognition. Zop keeps indentation and logical newlines in a separate layout
pass, following the boundary used by
[Ruff's Python lexer](https://github.com/astral-sh/ruff/tree/885bf665fb8962571270356da80be080c75fe191/crates/ruff_python_parser).

[rust-analyzer's parser](https://github.com/rust-lang/rust-analyzer/tree/b821f090608b98263d2a82d65bfb09d596380b26/crates/parser)
informs the separation between tokens, syntax, and semantic analysis plus the
rule that error recovery must always consume input. Zop does not adopt its
event stream or Rowan tree until the grammar needs them.

[Chumsky](https://docs.rs/chumsky/latest/chumsky/) supplies parser combinators,
Pratt parsing, and explicit recovery strategies. It is the preferred candidate
for replacing the bootstrap parser only when a measured implementation deletes
code while preserving spans, diagnostics, recovery, and compile speed. Zop
does not adopt a framework merely to replace working concrete code.

Mojo's public language documentation informs the source contract. Modular's
compiler parser is not public, so Zop does not claim it as an implementation
reference.

## OCaml and Haskell

OCaml demonstrates expression-oriented programming and constant-stack tail
recursion. Haskell demonstrates how recursive definitions can remain the
ordinary language of functional algorithms.

Zop is strict and plans to make proper tail calls a language guarantee in a
future milestone. The guarantee will not depend on an optimization profile.
This is stronger than Glasgow Haskell Compiler (GHC) loopification, which
targets saturated self-recursive tail calls under optimization.

Sources: [OCaml tail recursion](https://ocaml.org/docs/loops-recursion),
[OCaml tail-call assertions](https://ocaml.org/manual/5.5/attributes.html), and
[GHC loopification](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/using-optimisation.html#ghc-flag--floopification).

## Go

Go demonstrates the value of keeping errors ordinary and control flow visible.
Zop adopts those goals, not the conventional final `(value, error)` return
or optional checking.

Zop adds a typed error channel, mandatory handling, and explicit
propagation. Exported functions name one domain error; private functions may
request inference. Multiple success values remain one success tuple.

Sources: [Errors are values](https://go.dev/blog/errors-are-values) and
[Error handling and Go](https://go.dev/blog/error-handling-and-go).

## Concurrency

Go demonstrates lightweight tasks, first-class channels, and the clarity of
communicating ownership instead of sharing mutable state by default. Rust
demonstrates that transfer and concurrent sharing can be checked through type
structure. Kotlin demonstrates structured concurrency, where child work cannot
outlive its parent scope and failure propagates through the task tree.

Zop composes those ideas. Task groups and bounded channels are the easy
path. Ownership derives sendable and shareable properties without repeated
annotations. Threads, locks, atomics, and unsafe synchronization remain
available for systems code. Unlike a bare Go goroutine, ordinary spawned work
cannot detach silently or discard failure.

Sources: [Effective Go concurrency](https://go.dev/doc/effective_go#concurrency),
[Rust `Send` and `Sync`](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html),
and [Kotlin structured concurrency](https://kotlinlang.org/docs/coroutines-basics.html#coroutine-scope-and-structured-concurrency).

## JAX

JAX demonstrates composable program transformations for differentiation,
vectorization, and compilation. It also shows why transformed code needs
explicit state and controlled effects.

Zop adopts compiler-owned transformations without requiring a globally
functional programming style. The effect checker permits local mutation that
can be converted to value flow and rejects external effects inside a
differentiated region.

Sources: [JAX transformations](https://docs.jax.dev/en/latest/key-concepts.html)
and [stateful computations](https://docs.jax.dev/en/latest/stateful-computations.html).

## Shape polymorphism

Futhark and Dex demonstrate statically checked symbolic array dimensions. JAX
demonstrates symbolic shape relationships without requiring source-level
dependent types.

Zop adopts exact symbolic dimension identities for tensor signatures. A
dimension such as `n` expresses a relationship and does not force code
specialization. General symbolic arithmetic and an unrestricted constraint
solver are not part of the initial contract.

Sources: [Futhark size types](https://futhark.readthedocs.io/en/v0.26.3/glossary.html),
[Dex shape safety](https://google-research.github.io/dex-lang/examples/tutorial.html),
and [JAX shape polymorphism](https://docs.jax.dev/en/latest/export/shape_poly.html).

## Type inference

Damas-Milner inference demonstrates principal types and automatic polymorphic
generalization for an ML-like functional core. Bidirectional typing demonstrates
how local checking and synthesis support richer type systems with more
predictable annotations and error locations.

Zop uses unification inside a bounded bidirectional checker. Local values
and constrained closures infer naturally, while named parameter types, exported
contracts, recursive returns, effects, ownership modes, shapes, and generic
declarations stay explicit. It does not adopt automatic `let` generalization or
whole-program signature inference.

Sources: [Damas and Milner](https://steshaw.org/hm/milner-damas.pdf) and
[Dunfield and Krishnaswami](https://research.cs.queensu.ca/home/jana/papers/bidir-survey/).

## Generics

Go demonstrates why generics should follow concrete need rather than lead a
language design. Its type-parameter work began with flawed proposals shortly
after the language launched and took more than a decade to reach a design that
preserved clarity, build speed, and separate use of generic packages.

Zop adopts the resulting rule: add a type parameter when useful code would
otherwise be repeated with only its types changed. The
[generics contract](generics.md) specifies the syntax and semantics now while
deferring implementation until core collections provide real consumers. Type
arguments infer for ordinary callers, while template metaprogramming,
type-level reflection, and user specialization remain out of scope.

Sources: [Go generics proposal](https://go.dev/blog/generics-proposal),
[why generics](https://go.dev/blog/why-generics), and
[when to use generics](https://go.dev/blog/when-generics).

## Numeric literals

Rust demonstrates contextual typing for unsuffixed literals. JAX demonstrates
the readability benefit of allowing scalar literals to adopt an array's
precision. NumPy's promotion redesign demonstrates why a literal's numerical
value must not select the result type.

Zop adopts contextual numeric literals, then adds stricter boundaries.
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

Zop moves these facts into compiler-checked HIR. It does not adopt Rust
trait towers, runtime backend mismatches, or a dynamic graph as the only
autodiff model.

Sources: [backend-generic removal](https://github.com/tracel-ai/burn/pull/4717),
[autodiff](https://burn.dev/books/burn/building-blocks/autodiff.html), and
[gradient storage](https://github.com/tracel-ai/burn/blob/b6e27bdca620fbbc15e524c7088a7711f1a999f1/crates/burn-autodiff/src/grads.rs).

## Zig and Andrew Kelley

Zig demonstrates the value of explicit allocators, explicit input/output (I/O)
capabilities, explicit resource flow, fast compilation, and optimizer-visible
interfaces.

Zig's allocator passing directly inspires Zop's `Mem` capability. Zop
names the object for the authority it carries; `alloc` remains one operation in
the `Mem` API.

Zig's `comptime` demonstrates one language for runtime and compile-time
evaluation. Zop adopts the smaller requirement that selected parameters be
available during compilation. The source spelling is `name: known Type`.
Zop does not initially adopt arbitrary compile-time reflection, type
generation, or mandatory specialization of symbolic dimensions.

Andrew Kelley's
[Don't Forget to Flush](https://www.youtube.com/watch?v=f30PceqQWko&t=1723s)
showed that a hot buffered path should manipulate concrete state directly.
Only the cold refill or flush boundary should require a runtime-known function
pointer. His follow-up connected Rust's inferior assembly in that example to
crate visibility and devirtualization, not to LLVM or memory safety.

Zop adopts that interface rule while adding checked ownership. It does not
adopt unchecked manual memory management as the safe-language default.

Zig 0.16 also requires an explicit `Io` instance for operations that may block
or introduce nondeterminism. Zop adopts that capability boundary and uses
ownership to enforce flush, close, await, and cancellation obligations.

Sources: [Zig `comptime`](https://ziglang.org/documentation/master/#comptime),
[Andrew's follow-up](https://ziggit.dev/t/systems-distributed-dont-forget-to-flush/11431),
and [Zig I/O as an Interface](https://ziglang.org/download/0.16.0/release-notes.html#io-as-an-interface).

## MLIR

MLIR supplies tensor semantics, shape-aware transformations, explicit dialect
boundaries, bufferization, and ownership-based buffer deallocation.

[Melior](https://docs.rs/melior/0.27.4/melior/) supplies the Rust bindings. It
keeps verified MLIR in memory between Zop lowering and backend translation.

MLIR is the tensor optimizer, not Zop's general central processing unit
(CPU) code generator. Zop stops before LLVM intermediate representation on
the CPU path and translates its strict base boundary to Cranelift intermediate
representation.

Sources: [bufferization](https://mlir.llvm.org/docs/Bufferization/) and
[ownership-based deallocation](https://mlir.llvm.org/docs/OwnershipBasedBufferDeallocation/).

## Cranelift

Cranelift supplies fast native CPU code generation for development,
just-in-time (JIT), and ahead-of-time (AOT) builds. It keeps the edit-run loop
independent from LLVM's production optimization cost.

Zop supplies the source semantics, HIR, tensor optimization, and ownership
facts that Cranelift does not define.

Sources: [Cranelift frontend](https://docs.rs/cranelift-frontend/latest/cranelift_frontend/)
and [object backend](https://docs.rs/cranelift-object/latest/cranelift_object/).

## WebAssembly and the browser

WebAssembly supplies a fast, safe, portable browser instruction set, but core
WebAssembly deliberately has no document object model (DOM) or other host
access. It fits self-contained computation better than DOM glue. `wasm-bindgen`
demonstrates generated Web Interface Definition Language (Web IDL) bindings,
opaque host references, and npm-compatible packaging. The WebAssembly Component
Model demonstrates a future typed boundary for resources, strings, futures,
streams, and ECMAScript modules.

Zop adopts a distinct browser backend, explicit DOM and browser `Io`
capabilities, generated Web IDL bindings, structured event lifetimes, and
WebAssembly compute islands. It emits DOM work as JavaScript because that is
the browser's direct host language. It does not pretend current browsers
provide direct DOM access from core WebAssembly.

Sources: [WebAssembly core](https://www.w3.org/TR/wasm-core/),
[WebAssembly Component Model](https://github.com/WebAssembly/component-model),
[MLIR `wasmssa`](https://mlir.llvm.org/docs/Dialects/WasmSSAOps/), and
[`wasm-bindgen`](https://github.com/wasm-bindgen/wasm-bindgen).

## TypeScript and Rust web frameworks

TypeScript and its native Go compiler demonstrate an explicit ordered
transformation pipeline, copy-on-write syntax trees, deterministic generated
names, helper dependency ordering, public-signature incremental compilation,
source maps, and a simple final printer. TypeScript erases and lowers code; it
does not attempt general JavaScript optimization.

Zop adopts the compiler organization and adds optimization while ownership,
effects, calls, shapes, and browser capabilities remain known. It does not
inherit TypeScript's legacy target ladder, arbitrary JavaScript semantics, or
unrestricted transformer plugins.

Topcoat, created by Julien Scholz with Carl Lerche and the Tokio project,
demonstrates type-checked Rust expressions cross-compiled directly to
JavaScript for browser reactivity without a WebAssembly bundle. Zop adopts
that target choice. It does not embed source in HTML, scan the document to
recover programs, or compile expressions with `new Function` at runtime.
Topcoat's checked-in benchmark measures server rendering rather than browser
interaction, so it supports the architecture choice rather than a client-speed
claim.

Leptos demonstrates fine-grained reactivity: a component establishes direct
dependencies once, then a changed signal updates only its affected DOM node or
attribute. Its static-template and batched-mutation work also shows how compile
time structure removes allocation and host-boundary traffic. Zop adopts
those compiler opportunities without requiring a WebAssembly framework
runtime.

Dioxus demonstrates reusable static templates, event delegation, mutation
batching, and explicit lifecycle tests. Its virtual DOM supports several
rendering platforms, but its component reruns, tree diff, node table, and
mutation interpreter are costs Zop does not require by default.

Frontend frameworks remain packages. Signals, a virtual DOM, server rendering,
and hydration do not become language syntax before independent frameworks prove
a stable common contract.

Sources: [TypeScript emitter](https://github.com/microsoft/TypeScript/wiki/Codebase-Compiler-Emitter),
[TypeScript Go emitter](https://github.com/microsoft/typescript-go/blob/e8359e74015bbcc68cfa2a4d24430dd99b941259/internal/compiler/emitter.go),
[Topcoat announcement](https://tokio.rs/blog/2026-07-22-announcing-topcoat),
[Topcoat expression compiler](https://github.com/tokio-rs/topcoat/blob/a2bd596af2a149f38fcf49570481f356a6cb1069/crates/topcoat-runtime/grammar/src/expr.rs),
[Topcoat client runtime](https://github.com/tokio-rs/topcoat/blob/a2bd596af2a149f38fcf49570481f356a6cb1069/crates/topcoat-runtime/browser/src/event.ts),
[Leptos architecture](https://github.com/leptos-rs/leptos/blob/0625dfd15230b05174284fd56642681b918460fb/ARCHITECTURE.md),
and [Dioxus Web mutations](https://github.com/DioxusLabs/dioxus/blob/393d190a801ccb441d41923e232289b4f8a5c669/packages/web/src/mutations.rs).

## CUTLASS and CuTe IR

NVIDIA's [CuTe IR contribution](https://github.com/NVIDIA/cutlass/pull/3426)
supplies hierarchical device-layout algebra and a strict lowering boundary to
upstream MLIR dialects and NVIDIA Virtual Machine (NVVM) operations.

This is Zop's graphics processing unit (GPU) path.

Zop should consume and co-develop this work upstream. Zop still owns
language semantics, kernel extraction, effects, and the host runtime.

## COS 320

The COS 320 compiler assignments supplied a compact reference architecture:
typed IR, source spans, separate contexts, definite-return analysis, a semantic
interpreter, application binary interface (ABI) tests, dataflow analyses, and
correctness-before-quality gates.

Zop adopts those testing and boundary patterns. It does not adopt the
LLVMlite instruction set, hand-written x86 backend, global symbol generator,
or pedagogical error handling.

Source: [LLVMlite reference interpreter](https://github.com/cos320/hw2-llvmlite-jeff-windsor/blob/master/lib/ll/llinterp.ml).

## Standard-library layering

Rust separates a platform-independent `core`, allocation-backed collections,
and hosted standard-library services. Zig demonstrates explicit allocation and
I/O capabilities inside a cross-platform standard library. Go demonstrates the
ergonomic value and long-term compatibility cost of a broad standard library.

Zop adopts a small mandatory core, capability-aware standard modules, and
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
uv demonstrates the value of one fast command-line tool for projects,
dependencies, and toolchains. Bazel demonstrates stable target addresses,
package boundaries, graph queries, and remote build scale.

Zop adopts those properties with a stricter boundary: dependency commands
may use the network, but compilation is hermetic. Packages cannot run
unrestricted build scripts, and mission builds can reconstruct the complete
toolchain from vendored immutable inputs. Read and build commands never rewrite
the lockfile, synchronize an environment, or install a toolchain. Package
configuration is part of instance identity instead of a workspace-wide union.
One manifest pin wins over ambient toolchain overrides.

Zop also rejects mandatory source layouts and executable build
configuration. A flat package may keep `main.zop` beside `zop.toml`; an
optional source root changes where module names begin. Targets and imports
define the build graph. Workspaces add explicit members and stable addresses
without per-directory build programs or repeated source lists.

Sources: [Cargo lockfiles](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html),
[Cargo feature resolution](https://doc.rust-lang.org/cargo/reference/resolver.html),
[Rustup overrides](https://rust-lang.github.io/rustup/overrides.html),
[uv locking and syncing](https://docs.astral.sh/uv/concepts/projects/sync/),
[Go modules](https://go.dev/ref/mod),
[Cargo target paths](https://doc.rust-lang.org/cargo/reference/cargo-targets.html),
[Bazel packages and targets](https://bazel.build/concepts/build-ref),
[Zig package hashing](https://ziglang.org/download/0.16.0/release-notes.html#build-system),
and [Zig build system](https://ziglang.org/learn/build-system/).

## Artifacts and caches

Go demonstrates a concurrent user-level build cache that ordinary builds do not
need to clean. Bazel separates its output base from convenient workspace
symlinks. Nix demonstrates immutable store objects plus reachability-based
garbage collection. Bessemer separates the action graph, content-addressed
storage, action-result map, and materialized outputs.

Zop adopts one immutable user-level content store, a complete action key,
and a disposable `.zop/` workspace view. Final exports are explicit copies,
not mutable cache entries. A missing cache object may be recomputed; a corrupt
claimed object fails loudly instead of hiding the integrity event.

Sources: [Go build cache](https://pkg.go.dev/cmd/go#hdr-Build_and_test_caching),
[Bazel output directories](https://bazel.build/remote/output-directories),
[Nix garbage-collector roots](https://nix.dev/manual/nix/stable/package-management/garbage-collector-roots),
and [Bessemer caching](https://github.com/dedalus-labs/bsmr/blob/c89ca926c4af24b6d0e2f20ed0b907cea6be14ba/docs/concepts/caching.md).

## Bessemer

Bessemer treats each ecosystem's native manifests, lockfiles, and resolver as
authoritative. It lowers that graph into private rules, then owns action
identity, content-addressed storage, scheduling, restoration, and local or
remote execution. Its minimal hermetic model is a pure action over a declared
input tree; hashing alone does not make undeclared host access safe.

Zop adopts Bessemer as a co-developed first-class build path. A versioned
semantic graph keeps Zop in charge of language and dependency meaning.
Bessemer turns that graph into fine-grained actions without requiring users to
write `BUILD.bsmr`. Toolchains, dependency closures, host and device targets,
diagnostics, and compatibility fixtures cross one explicit protocol boundary.

Sources: [Bessemer roadmap](https://github.com/dedalus-labs/bsmr/blob/c89ca926c4af24b6d0e2f20ed0b907cea6be14ba/docs/roadmap.md),
[hermetic build core](https://github.com/dedalus-labs/bsmr/blob/c89ca926c4af24b6d0e2f20ed0b907cea6be14ba/docs/concepts/hermetic_build_core.md),
and [native TypeScript integration](https://github.com/dedalus-labs/bsmr/blob/c89ca926c4af24b6d0e2f20ed0b907cea6be14ba/docs/users/languages/typescript/pnpm.md).

## Why the composition fits Zop

No individual influence satisfies all Zop goals:

- Rust supplies safety, but not a tensor-native CPU and GPU language design.
- Zig supplies explicit systems control and interface transparency, but not
  checked ownership.
- Mojo validates ergonomic ownership for a Python-like surface, but Zop
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
- TypeScript supplies a fast, disciplined JavaScript emitter, but not a
  systems-language optimizer.
- Topcoat, Leptos, and Dioxus supply useful web-framework techniques, but each
  commits to runtime and rendering tradeoffs that should remain package choices.
- CuTe IR supplies device layouts, but not the complete language, runtime, or
  tensor-compute stack.
- COS 320 supplies an executable semantic oracle, but not a production
  language architecture.

Zop's distinctive choice is where facts live. Source states intent. Typed
HIR stores the proof. Lowering consumes explicit facts without exposing them as
generic parameters or runtime guesses.

Zop composes the influences into one contract:

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
