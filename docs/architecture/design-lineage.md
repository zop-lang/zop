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

Mojo 1.0 shows how ownership, compiler-tracked origins, compile-time values,
and heterogeneous kernels can fit a Python-like systems language. Its open
compiler makes those features implementation references rather than only
source-language precedents.

The Mojo compiler uses MLIR for every persistent intermediate representation.
Its parser emits a source-level `lit` dialect. Lifetime checking consumes
origin-carrying reference types before lowering. A parametric KGEN (Kernel
Generator) dialect is simplified before parallel elaboration into concrete
functions and types.

Zop adopts four boundaries:

- resolve peer declaration names, then signatures, then bodies;
- prove ownership and origins before generic specialization;
- simplify checked polymorphic IR before producing concrete instances; and
- isolate parser, semantic-pass, elaboration, lowering, and integration tests.

Zop does not adopt parser-time MLIR emission, universal monomorphization, or an
LLVM-only backend. Its syntax tree and typed high-level intermediate
representation (HIR) remain the source of truth for Cranelift, browser, device,
and reference-interpreter targets. Symbolic tensor extents still avoid
specialization unless generated code requires their value.

Mojo 1.0 was tagged before the compiler source appeared in the repository. Zop
therefore pins the open compiler snapshot at commit
`f66d4d522c34be0a961ffac3dbfc81e30f67942e`. The repository uses the Apache
License 2.0 with LLVM exceptions, but Modular does not yet accept external
compiler contributions. Zop may study and adapt its implementation patterns.
Any copied code must satisfy the repository's license and attribution terms;
co-development begins only when Modular opens that contribution boundary.

Sources: [Mojo 1.0 release](https://github.com/modular/modular/releases/tag/max/v26.5.0),
[open compiler repository](https://github.com/modular/modular/blob/f66d4d522c34be0a961ffac3dbfc81e30f67942e/README.md),
[compiler walkthrough](https://github.com/modular/modular/blob/f66d4d522c34be0a961ffac3dbfc81e30f67942e/KGEN/docs/MojoCompilerWalkthrough.md),
[declaration resolver](https://github.com/modular/modular/blob/f66d4d522c34be0a961ffac3dbfc81e30f67942e/KGEN/lib/MojoParser/DeclResolver.cpp),
[lifetime checker](https://github.com/modular/modular/blob/f66d4d522c34be0a961ffac3dbfc81e30f67942e/KGEN/lib/LowerLIT/CheckLifetimes.cpp),
[compiler testing](https://github.com/modular/modular/blob/f66d4d522c34be0a961ffac3dbfc81e30f67942e/KGEN/docs/testing.md),
[ownership](https://docs.modular.com/mojo/manual/values/ownership), and
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

Mojo's open `MojoParser`, source-level `lit` dialect, lifetime pass,
elaborator, and isolated test harness now join Logos, Ruff, rust-analyzer, and
Chumsky as concrete implementation references. Zop borrows their phase and
verification boundaries without copying Mojo's direct parser-to-MLIR
representation.

## OCaml and Haskell

OCaml demonstrates expression-oriented programming and constant-stack tail
recursion. Haskell demonstrates how recursive definitions can remain the
ordinary language of functional algorithms.

Zop makes proper tail calls a language guarantee at the systems-core milestone.
The guarantee covers compatible self-recursive, mutually recursive, direct, and
indirect CPU `fn` calls and never depends on an optimization profile. This is
stronger than Glasgow Haskell Compiler (GHC) loopification, which targets
saturated self-recursive tail calls under optimization.

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

Zop adopts the analyzable typed programs, explicit state, and controlled effects
that make such transformations possible without requiring a globally functional
programming style. It does not make `grad`, vectorization, or another model
strategy a language feature. Any future compiler-extension boundary must be
general rather than designed around one JAX transformation.

Sources: [JAX transformations](https://docs.jax.dev/en/latest/key-concepts.html)
and [stateful computations](https://docs.jax.dev/en/latest/stateful-computations.html).

## Shape polymorphism

Futhark and Dex demonstrate statically checked symbolic array dimensions. JAX
demonstrates symbolic shape relationships without requiring source-level
dependent types.

Zop adopts exact symbolic extent identities for tensor signatures. An extent
such as `n` expresses a relationship and does not force code
specialization. General symbolic arithmetic and an unrestricted constraint
solver are not part of the initial contract.

Sources: [Futhark size types](https://futhark.readthedocs.io/en/v0.26.3/glossary.html),
[Dex shape safety](https://google-research.github.io/dex-lang/examples/tutorial.html),
and [JAX shape polymorphism](https://docs.jax.dev/en/latest/export/shape_poly.html).

## PyTorch tensor surface

PyTorch establishes the tensor vocabulary most framework users already know:
trailing-dimension broadcasting, `dim`, `unsqueeze`, and allocation-free
`expand` views implemented with zero strides. Its documentation also warns that
expanded views alias storage and that broad `squeeze()` can remove a size-one
batch dimension accidentally.

Zop adopts the broadcast and view behavior while strengthening the core
contract. Core says `axis`, `extent`, `rank`, and hierarchical layout `mode`;
frameworks may expose `dim`. Core `squeeze` requires one explicit axis, expanded
views obey injectivity rules, and materialization requires `Mem`.

Sources: [PyTorch broadcasting](https://docs.pytorch.org/docs/stable/notes/broadcasting.html),
[`Tensor.expand`](https://docs.pytorch.org/docs/stable/generated/torch.Tensor.expand.html),
and [`torch.squeeze`](https://docs.pytorch.org/docs/stable/generated/torch.squeeze.html).

## Iterable and layout zipping

Python `zip` stops at the shortest iterable by default and provides
`strict=True` when equal exhaustion is an asserted invariant. Zop adopts that
single familiar callable and named policy rather than auto-zipping commas in a
`for` header or creating separate `zip_equal` and `zip_shortest` APIs. A strict
dynamic mismatch traps because Zop iteration has no hidden exception channel.

CuTe's similarly named `zipped_divide` is unrelated. It promotes a tiler to one
layout, performs logical division, then groups tile modes and remainder modes.
Zop keeps the full name under `Layout` so tensor programmers can use CuTe
terminology without changing ordinary iterable `zip`.

Sources: [Python `zip`](https://docs.python.org/3/library/functions.html#zip)
and [PyCuTe `zipped_divide`](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/docs/04_layout_algebra.md#zipped_divide-tiled_divide-flat_divide).

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
implicit lossy integer conversion is a compile error. An explicit cast names
its destination and loss policy; `bitcast` remains representation-only.

Sources: [Rust literal inference](https://doc.rust-lang.org/reference/expressions/literal-expr.html#integer-literal-expressions),
[JAX type promotion](https://docs.jax.dev/en/latest/jep/9407-type-promotion.html),
and [NumPy NEP 50](https://numpy.org/neps/nep-0050-scalar-promotion.html).

## Integer arithmetic

C leaves signed overflow undefined and defines unsigned arithmetic modulo the
type width. Go and Java define deterministic fixed-width wraparound. Rust checks
ordinary arithmetic in debug builds and wraps when overflow checks are
disabled. Zig treats overflow as illegal behavior, catches it in safe modes,
and exposes separate wrapping and saturating operators. Swift checks ordinary
arithmetic and requires separate overflow operators. Ada checks signed
arithmetic and provides explicit modular types.

Zop adopts the strongest stable composition: ordinary signed and unsigned
arithmetic requires every mathematical result to fit its type in every build
mode. Compile-time overflow is a diagnostic and runtime overflow is a trap.
Wrapping, saturating, and recoverable operations are explicit. Optimization may
prove a test unnecessary but cannot change its observable semantics.

Rust establishes numeric dot methods for checked, wrapping, and saturating
policies. Swift pairs trapping operators with a dot method that reports
overflow. Zig pairs its ordinary operator with `std.math.add`, which returns an
overflow error. Zop composes those precedents as `try to left.add right`: `add`
is an ordinary member whose type declares `Overflow`, while `try to` propagates
the failure. The member is not a keyword, and the ordinary `+` operator retains
one context-independent trapping meaning.

Sources: [C integer terminology](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n2811.pdf),
[Go integer overflow](https://go.dev/ref/spec#Integer_overflow),
[Rust overflow](https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow),
[Rust integer methods](https://doc.rust-lang.org/std/primitive.i64.html),
[Zig integer overflow](https://ziglang.org/documentation/master/#Integer-Overflow),
[Java integer operations](https://docs.oracle.com/en/java/javase/26/docs/specs/jls/jls-4.html#jls-4.2.2),
[Swift overflow operators](https://docs.swift.org/swift-book/documentation/the-swift-programming-language/advancedoperators/#Overflow-Operators),
[Swift integer methods](https://developer.apple.com/documentation/swift/int64),
and [Ada integer types](https://www.adaic.org/resources/add_content/standards/22rm/html/RM-3-5-4.html).

## Division and precision

Python and JAX make `/` true division and provide a separate floor-division
operation. Haskell restricts `/` to fractional types and gives integral values
separately named quotient operations. MLIR distinguishes floating, truncating,
floor, ceiling, and exact division in typed operations. WebGPU Shading Language
does not automatically convert a materialized integer to floating point and has
no materialized `f64` type.

Zop composes those boundaries. `/` is fractional and preserves one floating
type. `//` is same-type integer floor division, while named members expose
truncating, ceiling, Euclidean, exact, and recoverable behavior. Only literals
may adopt the operator's expected type. A named integer never becomes floating
point because division has a remainder or because a target prefers another
type. Non-strict floating semantics use an ordinary trailing parameter of
`known FloatProfile`, so the call site names weaker behavior without another
annotation grammar or ambient compiler mode.

Sources: [Python arithmetic](https://docs.python.org/3/reference/expressions.html#binary-arithmetic-operations),
[JAX true division](https://docs.jax.dev/en/latest/_autosummary/jax.numpy.true_divide.html),
[GHC numeric classes](https://downloads.haskell.org/ghc/9.14.1/docs/libraries/ghc-9.14.1-da80/GHC-Prelude-Basic.html),
[MLIR arithmetic operations](https://mlir.llvm.org/docs/Dialects/ArithOps/),
and [WGSL scalar types](https://www.w3.org/TR/WGSL/#scalar-types).

## Numeric failure and execution domains

Python avoids integer overflow with arbitrary-precision integers but gives up a
fixed representation and predictable machine cost. NumPy restores fixed-width
arrays but permits integer wraparound and uses mutable ambient `seterr` policy
to ignore, warn, raise, call, print, or log floating exceptions. PyTorch follows
IEEE floating behavior but documents that some linear-algebra backends may
return nonfinite values, raise, or fail catastrophically when inputs are
nonfinite. JAX provides valuable NaN instrumentation, but its documentation
warns about device-host round trips, performance regressions, and false
positives. JAX also clamps out-of-bounds reads and drops out-of-bounds updates
because accelerator error propagation is difficult.

Zop rejects those loose seams. Fixed-width integer operators trap in every
build, while named fallible, wrapping, and saturating members make expected
overflow concise. Floating operations retain strict IEEE NaN and infinity
values; an explicit `require_finite` check establishes a flow-sensitive fact
when an algorithm or vendor backend requires finite input. Optional
`--check-nonfinite` instrumentation helps find accidental values without
becoming ambient program semantics.

Swift and Zig provide the strongest default-overflow precedent. Rust provides
the clearest checked, strict, wrapping, and saturating operation families but
lets ordinary overflow behavior vary with compiler settings. Zop composes
Swift and Zig's invariant default with Rust's discoverable named alternatives.
It uses methods rather than additional operator alphabets so source remains
readable to researchers and systems programmers.

CUDA demonstrates the honest device-failure boundary. A device assertion aborts
the kernel, corrupts its context, and invalidates every allocation in that
context. Zop elevates that hardware fact into a portable execution-domain rule:
a CPU trap terminates its process, and a device trap invalidates its complete
context. No successful path pays for transactional tensor rollback, while no
failed path exposes partial storage as a valid tensor. Host recovery creates a
fresh context and reconstructs data explicitly.

The end-user composition is strict without demanding routine annotations.
Ordinary integer and floating expressions use familiar operators. The compiler
proves safety when it can and otherwise preserves one documented behavior.
Users name extra policy only when they need recovery, modular arithmetic,
saturation, finite-only data, or debugging. No build mode, backend, or ambient
library setting silently changes their choice.

Sources: [Python numeric types](https://docs.python.org/3/library/stdtypes.html#numeric-types-int-float-complex),
[NumPy data types and overflow](https://numpy.org/doc/stable/user/basics.types.html),
[NumPy floating error handling](https://numpy.org/doc/2.4/reference/routines.err.html),
[PyTorch numerical accuracy](https://docs.pytorch.org/docs/stable/notes/numerical_accuracy.html),
[JAX NaN debugging](https://docs.jax.dev/en/latest/debugging/flags.html#jax-debug-nans-configuration-option-and-context-manager),
[JAX out-of-bounds indexing](https://docs.jax.dev/en/latest/notebooks/Common_Gotchas_in_JAX.html#out-of-bounds-indexing),
[Swift overflow operators](https://docs.swift.org/swift-book/documentation/the-swift-programming-language/advancedoperators/#Overflow-Operators),
[Zig integer overflow](https://ziglang.org/documentation/master/#Integer-Overflow),
[Rust integer methods](https://doc.rust-lang.org/std/primitive.i32.html#method.checked_add),
[CUDA device assertions](https://docs.nvidia.com/cuda/cuda-programming-guide/05-appendices/cpp-language-extensions.html#assertion),
and [CUDA context invalidation](https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__TYPES.html).

## Burn

Burn demonstrates backend-independent tensor APIs, explicit gradient storage,
activity tracking, and gradient checkpointing. Its move away from a backend
generic on every public tensor also validates keeping execution machinery out
of user-facing tensor types.

Zop adopts the tensor-type lesson without adopting Burn's automatic
differentiation strategy as a language feature. Gradient storage and
checkpointing remain framework choices rather than hidden tensor state.

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
generation, or mandatory specialization of symbolic extents.

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

MLIR's Transform dialect demonstrates an extensible transformation
intermediate representation without exposing the payload compiler's internal
objects as a source-language API. Zop may draw on that design only after
multiple unrelated package transformations justify a public extension boundary.

Sources: [bufferization](https://mlir.llvm.org/docs/Bufferization/) and
[ownership-based deallocation](https://mlir.llvm.org/docs/OwnershipBasedBufferDeallocation/),
and the [Transform dialect](https://mlir.llvm.org/docs/Dialects/Transform/).

## Cranelift

Cranelift supplies fast native CPU code generation for development,
just-in-time (JIT), and ahead-of-time (AOT) builds. It keeps the edit-run loop
independent from LLVM's production optimization cost.

Zop supplies the source semantics, HIR, tensor optimization, and ownership
facts that Cranelift does not define.

Sources: [Cranelift frontend](https://docs.rs/cranelift-frontend/latest/cranelift_frontend/)
and [object backend](https://docs.rs/cranelift-object/latest/cranelift_object/).

## SIMD and vectorization

Mitchell Hashimoto presents everyday SIMD as a regular five-part schedule:
broadcast, full-width chunks, lane operations, reduction or store, then a
tail. His Ghostty example also identifies the practical failure mode of
heuristic vectorization: a hot loop can quietly become scalar after a compiler
or source change.

MLIR's Vector dialect supplies retargetable multidimensional vector values,
transfers, masks, reductions, scans, and contractions. Its own documentation
places automatic scalar-to-vector raising outside the dialect's scope. Its
structured transform vectorizes Linalg operations but explicitly does not
vectorize loops or straight-line scalar code. Its affine vectorizer describes
its profitability analysis as a simple strawman rather than a universal cost
model.

The Intermediate Representation Execution Environment (IREE) compiler
demonstrates the production composition. It reuses upstream MLIR while
maintaining its own generic vectorization policy, target vector sizes, masking,
gather-like operations, contractions, and microkernel selection. A microkernel
is a small target-tuned implementation of one compute operation. IREE's RISC-V
guidance reports generic vectorization as efficient while preferring
microkernels as the more stable path. Cranelift supplies fixed-width vector
static single-assignment (SSA) types plus mask reductions such as `vany_true`,
`vall_true`, and `vhigh_bits` for fast native code generation.

Zop composes the three layers. Source uses semantic search, map, aggregation,
scan, reduction, and tensor operations. HIR retains Layout, bounds, aliases,
effects, traps, and order long enough for Zop to prove the schedule. MLIR
expresses the accepted vector work, and Cranelift emits the CPU instructions.
A versioned report makes both vector and deliberate scalar decisions testable.
This preserves approachable source without trusting an invisible optimizer or
requiring ordinary users to write target intrinsics.

Sources: [Everyone Should Know SIMD](https://mitchellh.com/writing/everyone-should-know-simd),
[MLIR Vector dialect](https://mlir.llvm.org/docs/Dialects/Vector/),
[MLIR structured vectorization](https://mlir.llvm.org/python-bindings/autoapi/mlir/dialects/transform/structured/index.html),
[MLIR affine super-vectorization](https://mlir.llvm.org/doxygen/SuperVectorize_8cpp.html),
[MLIR Transform dialect tutorial](https://mlir.llvm.org/docs/Tutorials/transform/),
[IREE generic vectorization](https://iree.dev/reference/mlir-passes/CodegenCommon/),
[IREE RISC-V code generation](https://iree.dev/community/blog/2026-07-23-running-models-on-risc-v-with-iree/),
and [Cranelift intermediate representation](https://github.com/bytecodealliance/wasmtime/blob/main/cranelift/docs/ir.md).

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
supplies hierarchical layout types, algebraic operations, static folding, and
a strict lowering boundary to upstream MLIR dialects and NVIDIA Virtual Machine
(NVVM) operations.

[PyCuTe](https://github.com/NVlabs/CuTe) supplies a target-independent,
executable reference implementation of the same algebra plus layout, tensor,
copy, contraction, and visualization examples.

Zop adopts CUTLASS CuTe's `Tensor<Engine, Layout>` model as a language-native
contract on every target. Engine wraps the iterator or owned array; Layout maps
logical coordinates to Engine indices. PyCuTe calls its reference equivalent
`Tensor(Accessor, Layout)`. CPU code evaluates the algebra through Cranelift,
while `kn` code preserves it into CuTe IR. Zop adds checked bounds, ownership,
borrow origins, mutation rules, typed dynamic failures, and cross-target
conformance without inventing a third ABI origin field.

The [CuTe layout paper](https://arxiv.org/abs/2603.02298) explains why a
hierarchical Shape and Stride must extend traditional flat tensor descriptors
to represent hardware instructions correctly. The [categorical
analysis](https://arxiv.org/abs/2601.05972) formalizes the composition, logical
product, and logical division algebra. CUTLASS's [Tensor and Engine
definition](https://github.com/NVIDIA/cutlass/blob/6c68991985ca8b09594ac6fd43abbfd5830c4140/media/docs/cpp/cute/03_tensor.md)
provides the concrete data half of that model, and PyCuTe's
[`Accessor`](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/docs/05_tensor.md#accessors)
supplies the executable reference adaptation.

Zop should consume and co-develop CuTe IR upstream. Zop still owns language
semantics, kernel extraction, effects, and the host runtime.

## Tensor indexing and views

Python supplies readable negative indexing and half-open slice notation, but
its sequence slices may copy and its dynamic object model is not a systems
contract. PyTorch supplies the valuable distinction that basic tensor indexing
creates a view while advanced indexing creates a copy, but the allocation and
shape boundary remains library behavior. Go demonstrates a compact shared-array
descriptor and strict bounds, while its capacity field permits reslicing that
does not fit fixed-shape tensors. Java demonstrates a stable constant-time
logical length but only for one array level. C++ `mdspan` separates extents,
mapping, accessor, and data handle without providing language ownership.

Zop composes the precise subset: Python surface syntax and endpoint clipping,
PyTorch basic-view behavior, Java-like stable Shape, CuTe Engine-plus-Layout
representation and residual algebra, explicit ownership, and MLIR lowering.
Named `slice(..., strict=true)` adds a recoverable exact-boundary assertion
without making routine bracket chunking verbose.
`.shape`, `.rank`, `extent axis=`, and `numel()` name distinct logical facts;
Layout `.cosize` names a scalar codomain size when defined. Static Engine and
Layout profiles erase, only dynamic leaves remain, and no basic selection copies
storage. The complete
[indexing rationale](indexing.md#why-this-composition) records the composition
and primary references.

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
search belong in `core`; neural-network frameworks do not. Go added predeclared
`min` and `max` in 1.21 with rigorous NaN propagation, signed-zero ordering,
infinity behavior, and one-or-more-argument semantics. Zop adopts those
semantics under explicit imports from the standalone bundled `math` module
while retaining stricter same-concrete-type operands and explicit tensor
reduction order. Keeping them out of the prelude preserves ordinary user names
without making a ubiquitous algorithm depend on the hosted standard library.

Python demonstrates the readability of a flat `math` import path. Zop keeps
the module allocation-free and legal on compatible bare-metal and device
targets. The internal core-versus-hosted layering therefore does not leak into
ordinary mathematical imports, and `math` remains bundled rather than becoming
an independently versioned package.

Sources: [Python `math`](https://docs.python.org/3/library/math.html),
[Rust `core`](https://doc.rust-lang.org/core/),
[Rust `alloc`](https://doc.rust-lang.org/stable/alloc/),
[Zig memory](https://ziglang.org/documentation/master/#Memory),
[Go standard library](https://pkg.go.dev/std), and
[Go `min` and `max`](https://go.dev/ref/spec#Min_and_max).

## Documentation and editor tooling

JSDoc popularized explicit `@param`, `@returns`, `@throws`, and `@example`
sections, but it must encode types in comments because JavaScript may not
provide them. TSDoc makes the tag grammar substantially more rigorous by
distinguishing summary content, block tags, modifier tags, and inline tags.
Rustdoc contributes compiler-resolved symbol links, generated signature pages,
and executable documentation tests. mdBook demonstrates that narrative books
remain authored Markdown even when their examples are compiled and tested.
The Language Server Protocol supplies one editor-neutral transport for hover,
completion, navigation, rename, formatting, diagnostics, inlay hints, and
semantic tokens, including a standard `documentation` modifier.

Zop adopts `##` as a distinct documentation token inside its existing `#`
comment family. It uses a small compiler-checked `@` tag schema without
duplicating types, defaults, effects, or ownership from the signature. One
semantic documentation model drives editor hover, generated API reference,
package registries, and executable examples. Narrative books remain separate
Markdown inputs rendered by the same toolchain. A thin LSP adapter exposes the
compiler's symbol identities and documentation instead of maintaining a second
parser or index.

This composition is more structured than Go's positional prose, avoids
JSDoc's duplicate type language, preserves rustdoc's strongest correctness
features, and prevents API generation from replacing tutorials and design
explanation.

Sources: [JSDoc `@param`](https://jsdoc.app/tags-param),
[TSDoc tag kinds](https://tsdoc.org/pages/spec/tag_kinds/),
[rustdoc documentation tests](https://doc.rust-lang.org/rustdoc/documentation-tests.html),
[mdBook](https://rust-lang.github.io/mdBook/), and the
[Language Server Protocol](https://microsoft.github.io/language-server-protocol/).

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
- Mojo validates ergonomic ownership and phased MLIR compilation for a
  Python-like surface, but Zop retains typed HIR, adaptive specialization,
  Cranelift, and an independent browser path.
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
- CuTe supplies the universal layout algebra and a device lowering, but not the
  complete language, runtime, or tensor-compute stack.
- COS 320 supplies an executable semantic oracle, but not a production
  language architecture.

Zop's distinctive choice is where facts live. Source states intent. Typed
HIR stores the proof. Lowering consumes explicit facts without exposing them as
generic parameters or runtime guesses.

Zop composes the influences into one contract:

1. Inferred proofs and optional checked annotations without hidden dynamic
   typing.
2. Symbolic tensor extents without mandatory code specialization.
3. Pure compile-time parameters only where generated code needs their values.
4. Explicit numeric type, quotient, conversion, and precision policy.
5. Parameter-mode borrowing with first-class origin-tracked tensor views.
6. Checked ownership without a mandatory garbage collector.
7. Explicit `Mem`, `Io`, placement, mutation, ownership transfer, and errors.
8. MLIR tensor semantics, buffer reuse, and GPU lowering.
9. Cranelift development speed and native CPU artifacts.
10. An independent interpreter that checks every backend.
11. A layered standard library with correct universal algorithms.
12. Hermetic packages that rebuild from immutable offline inputs.

The boundaries are the advantage. Each component owns the problem it solves,
and no component silently substitutes for another when it fails.
