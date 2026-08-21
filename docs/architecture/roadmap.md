# Roadmap

Zop versions are earned capability gates, not dates. A milestone ships only
when every exit criterion passes on its declared support matrix. Missing work
does not silently move to a fallback implementation.

The [release contract](releases.md) mirrors these criteria into one GitHub
milestone and release-gate issue per version. cargo-dist cannot publish until
that milestone, its gate checklist, and all attached work are closed. The
[GitHub roadmap](https://github.com/orgs/zop-lang/projects/1) is the public
execution view of this file.

## Development readiness

No open language decision blocks the Rust bootstrap, complete frontend phases,
scalar reference interpreter, or first internal fixed-`f32` tensor slice.
Implementation may begin against their existing contracts.

The remaining decisions belong to later evidence gates:

- strict cross-target accuracy before non-integral exponentiation and
  transcendental `math` functions ship;
- byte-level tensor and error-result packing before the foreign ABI freezes;
- final input/output and browser capability spellings before their standard
  modules freeze;
- general-view syntax after a non-slice consumer proves the required surface;
- expert-SIMD source APIs after two target implementations prove the common
  contract; and
- warning, compiler-backtrace, and runtime-trace policy before the complete
  toolchain milestone closes.

Compiler implementation cannot settle these choices silently. Each requires a
focused contract update and conformance evidence before its owning milestone
closes.

## 0.0.x: internal bootstrap checkpoints

`0.0.x` identifies disposable Rust stage-0 checkpoints. These builds may prove
compiler plumbing, but they are not supported language releases and cargo-dist
does not publish them.

The current checkpoint can parse and type-check the documented scalar subset,
emit deterministic ECMAScript, lower `i64` host functions through verified
MLIR, execute them with Cranelift JIT, and emit native objects. Its arithmetic
and command surface remain explicitly nonconforming or incomplete where the
architecture documents say so. CI may retain artifacts for compiler debugging;
users must not infer a language capability promise from their version numbers.

## 0.1.0: usable scalar language

The binary support matrix for this milestone is `aarch64-apple-darwin` with
Homebrew LLVM 22. Target breadth is earned at 0.5.0.

- Parse and type-check a documented scalar host language with functions,
  direct calls, locals, `i64`, `bool`, unit, comparisons, conditionals, and
  ordinary recursion into verified typed HIR.
- Lower that complete subset through an explicit verifier-gated MLIR pass
  pipeline and the restricted MLIR-to-Cranelift boundary.
- Implement build-invariant trapping `i64` arithmetic and the documented
  integer division and remainder semantics in the interpreter, JIT, and AOT
  paths. No bootstrap wrapping or generic truncating operation may remain.
- Ship a small reference interpreter for the same boundary and require value,
  output, and failure-class parity with Cranelift JIT and AOT execution.
- Provide minimal explicit `Io` standard output, including UTF-8 string and
  scalar formatting, without introducing ambient process state.
- Ship `zop run` for one source program and `zop build` for one runnable native
  executable. Retain explicit `mlir`, object, and JavaScript artifact commands.
- Emit deterministic ECMAScript for the documented scalar browser subset.
- Reject every unsupported construct with a structured, source-backed
  diagnostic before it reaches a weaker stage or target.
- Install the archive and shell installer into clean prefixes on the declared
  host, then pass hello-world, arithmetic, conditional Fibonacci,
  multi-function, recursion, and invalid-program smoke fixtures.

`0.1.0` is the first public binary release. A merge that improves compiler
plumbing without satisfying every item above remains a `0.0.x` checkpoint.

## 0.2.0: language core

- Complete blocks, `type` products and sums, tuples, closures, typed failures,
  modules, defaults, and named calls.
- Index declaration names, resolve signatures, and check bodies as separate
  phases with forward-reference, interface-hash, and cycle tests.
- Complete local bidirectional inference and exhaustive pattern checking.
- Implement compound update assignments, fallible numeric members, and explicit
  wrapping and saturating operations across the interpreter and backends.
- Implement fractional `/`, integer `//` and `%`, every quotient mode, explicit
  numeric casts, bundled `math.min` and `math.max`, and strict cross-target
  floating-point profiles.
- Implement process-domain traps with invariant semantics across development and
  optimized native, JavaScript, and WebAssembly builds.
- Ship intent-aware numeric diagnostics whose structured suggestions are tested
  for applicability and compile after machine-applied edits.
- Distinguish ordinary `#` comments from structured `##` documentation, bind
  the initial `@` tag set, and preserve documentation through semantic queries.
- Implement interpreted and raw triple-quoted strings, structural
  destructuring, explicit `zip` strictness, and Go-like table subtests.
- Ship the canonical formatter plus lexical highlighting fixtures when the
  source grammar freezes.
- Freeze the source grammar represented by the conformance corpus.

## 0.3.0: tensor CPU

- Implement fixed-rank, statically typed tensor literals and symbolic extents.
- Implement core axis semantics, trailing-axis broadcasting, `unsqueeze`,
  `squeeze`, and zero-stride `expand` views.
- Implement the [indexing contract](indexing.md): negative integer indices,
  half-open basic slices, recoverable `at`, residual layouts, bounds proof, and
  zero-copy descriptor construction with clipped brackets and named strict
  slicing.
- Implement language-native affine and
  [composed Layout expressions](layout-expressions.md), exact zero-copy views,
  pure algebra, and conformance across CUTLASS CuTe, PyCuTe, and `tensor-layouts`.
- Lower elementwise operations and matrix multiplication through MLIR tensor,
  Linalg tiling, upstream structured vectorization, bufferization, and
  deallocation passes.
- Implement trapping, wrapping, saturating, and fresh-output fallible tensor
  arithmetic plus `require_finite` and nonfinite debug instrumentation.
- Implement source-ordered and explicit tree reductions without a separate
  accumulation operator.
- Implement Zop-owned legality and schedule selection for compiler-known
  search, predicate aggregation, map, scan, and reduction operations on one
  pinned current-host profile. Add specialized patterns for ordered operations
  not represented by upstream structured vectorization.
- Lower every accepted semantic schedule through explicit MLIR Vector and CLIF
  operations; do not gate this milestone on a general upstream loop
  auto-vectorizer.
- Ship vectorization reports, scalar/vector differential tests, Layout and tail
  conformance, pinned code-generation checks, and measured crossover baselines.
- Match the reference interpreter in JIT and AOT modes.
- Publish compile-time and runtime baselines for the tensor kernel corpus.

## 0.4.0: systems core

- Implement ownership, borrowing, `mut`, `give`, deterministic destruction,
  `Mem`, `Io`, views, raw pointers, and unsafe checking.
- Ship the C foreign-function boundary and stable native runtime ABI.
- Implement the [generic contract](generics.md) under
  [proposal #2](https://github.com/zop-lang/zop/issues/2), after its evidence
  gate proves three core-library consumers.
- Ship checked polymorphic HIR, pre-elaboration simplification, a pure
  compile-time interpreter, deterministic parallel instance expansion, and
  content-addressed instance caching.
- Implement structured tasks, channels, cancellation, and race rejection for
  safe code.
- Implement proper self-recursive, mutually recursive, direct, and indirect
  tail calls before freezing the native calling convention.

## 0.5.0: target breadth

- Support the declared native CPU host matrix without semantic drift.
- Pass fixed-width SIMD conformance on pinned x86-64 and AArch64 profiles, ship
  explicit native feature variants, and freeze the portable `core.simd` surface.
- Compile `kn` through one production GPU toolchain with explicit placement,
  launch, transfer, and lifetime checks.
- Ship one vendor-checked matrix-multiply and copy atom registry plus GF(2)
  analysis for eligible bit-linear thread/value layouts.
- Implement kernel-local failure handling, context-invalidating device traps,
  `DeviceError` host reporting, invalid-allocation rejection, and explicit
  context reconstruction.
- Pass the first browser ECMAScript, Web IDL, WebAssembly-island, and WebGPU
  conformance profiles.
- Publish target-specific unsupported-feature diagnostics and performance
  budgets.

## 0.6.0: complete toolchain

- Ship `zop init`, test, format, doc, package, and workspace commands from one
  locked dependency graph, and complete the build and run workspace surfaces.
- Make builds hermetic after resolution and make cache identity explainable.
- Ship the lean `core` and `std` boundary, structured documentation comments,
  checked examples, generated API reference, and configured narrative books.
- Ship a stable Language Server Protocol baseline with diagnostics, completion,
  signature help, hover, navigation, references, semantic rename, semantic
  tokens, inlay hints, code actions, and canonical formatting.
- Ship thin official editor integrations and publish syntax, documentation,
  protocol, responsiveness, and monorepo conformance results.
- Ship fuzzing and the conformance runner.
- Ship isolated parser, signature, semantic-pass, elaboration, MLIR-pass, and
  end-to-end compiler test entry points with minimal package stubs.
- Build the same workspace natively through Bessemer.

## 0.7.0: self-host entry audit

- Dogfood the formatter, language server, documentation generator, package or
  build tool, substantial systems program, and tensor framework across two
  releases.
- Complete three consecutive releases without redesigning source semantics,
  typed HIR, the runtime ABI, or backend contracts.
- Meet every gate in [self-hosting](self-hosting.md), including offline stage-0
  reconstruction and published compiler resource budgets.
- Freeze the compiler-facing `core` surface needed by a Zop implementation.

Failing any item keeps self-hosting closed.

## 0.8.0: self-hosted compiler

- Implement the frontend, semantic analysis, HIR, lowering, and driver in Zop.
- Build stage 1 with Rust stage 0, then build stage 2 with stage 1.
- Match normalized HIR, MLIR, Cranelift IR, diagnostics, behavior, and artifacts
  under the bootstrap proof.
- Keep stage 0 supported as the recovery and audit root.

## 0.9.0: compatibility candidate

- Make the self-hosted compiler the default after the bootstrap proof passes on
  every supported host.
- Freeze the 1.0 source, package, runtime ABI, tool protocol, and target-profile
  contracts.
- Resolve every release-blocking diagnostic, fuzzing, security, conformance,
  reproducibility, and performance finding.
- Permit only compatibility fixes during the release-candidate cycle.

## 1.0.0

Zop 1.0.0 makes a compatibility promise only after all of these are true:

- The specification has no open decision in the stable language surface.
- Stage 0, stage 1, and stage 2 reproduce from pinned sources without network
  access on every supported host.
- Differential, property, fuzz, ABI, package, and target conformance suites pass
  with published evidence.
- At least one native CPU, one GPU, and the browser profile meet their published
  correctness and performance floors.
- The standard library, package manager, workspaces, cache, formatter, language
  server, test runner, documentation, and diagnostics are release-quality.
- A compiler, tensor framework, browser framework, and production systems
  application exercise the stable surface without private compiler hooks.
- Unsafe boundaries are auditable, safe code is data-race-free, and no known
  soundness defect remains open.

Self-hosting is necessary for 1.0.0, but self-hosting alone is not maturity.
