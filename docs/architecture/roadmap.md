# Roadmap

Zop versions are earned capability gates, not dates. A milestone ships only
when every exit criterion passes on its declared support matrix. Missing work
does not silently move to a fallback implementation.

## 0.1: bootstrap

- Parse and type-check the scalar host subset into typed HIR.
- Verify MLIR and translate one restricted boundary to Cranelift.
- Execute JIT code and emit linkable AOT objects.
- Emit deterministic ECMAScript for the documented scalar browser subset.
- Reject every unsupported construct with a structured diagnostic.

This is the current Rust stage-0 scope.

## 0.2: language core

- Implement control flow, blocks, `type` products and sums, tuples, closures,
  typed failures, modules, defaults, and named calls.
- Complete local bidirectional inference and exhaustive pattern checking.
- Establish a reference interpreter for the shared scalar boundary.
- Freeze the source grammar represented by the conformance corpus.

## 0.3: tensor CPU

- Implement fixed-rank, statically typed tensor literals and symbolic
  dimensions.
- Lower elementwise operations and matrix multiplication through MLIR tensor,
  linalg, bufferization, and deallocation passes.
- Match the reference interpreter in JIT and AOT modes.
- Publish compile-time and runtime baselines for the tensor kernel corpus.

## 0.4: systems core

- Implement ownership, borrowing, `mut`, `give`, deterministic destruction,
  `Mem`, `Io`, views, raw pointers, and unsafe checking.
- Ship the C foreign-function boundary and stable native runtime ABI.
- Implement the [generic contract](generics.md) from proven core-library
  duplication.
- Implement structured tasks, channels, cancellation, and race rejection for
  safe code.

## 0.5: target breadth

- Support the declared native CPU host matrix without semantic drift.
- Compile `kn` through one production GPU toolchain with explicit placement,
  launch, transfer, and lifetime checks.
- Pass the first browser ECMAScript, Web IDL, WebAssembly-island, and WebGPU
  conformance profiles.
- Publish target-specific unsupported-feature diagnostics and performance
  budgets.

## 0.6: complete toolchain

- Ship `zop init`, build, run, test, format, document, package, and workspace
  commands from one locked dependency graph.
- Make builds hermetic after resolution and make cache identity explainable.
- Ship the lean `core` and `std` boundary, generated API documentation,
  editor protocol, fuzzing, and conformance runner.
- Build the same workspace natively through Bessemer.

## 0.7: self-host entry audit

- Dogfood a formatter, package or build tool, substantial systems program, and
  tensor framework across two releases.
- Complete three consecutive releases without redesigning source semantics,
  typed HIR, the runtime ABI, or backend contracts.
- Meet every gate in [self-hosting](self-hosting.md), including offline stage-0
  reconstruction and published compiler resource budgets.
- Freeze the compiler-facing `core` surface needed by a Zop implementation.

Failing any item keeps self-hosting closed.

## 0.8: self-hosted compiler

- Implement the frontend, semantic analysis, HIR, lowering, and driver in Zop.
- Build stage 1 with Rust stage 0, then build stage 2 with stage 1.
- Match normalized HIR, MLIR, Cranelift IR, diagnostics, behavior, and artifacts
  under the bootstrap proof.
- Keep stage 0 supported as the recovery and audit root.

## 0.9: compatibility candidate

- Make the self-hosted compiler the default after the bootstrap proof passes on
  every supported host.
- Freeze the 1.0 source, package, runtime ABI, tool protocol, and target-profile
  contracts.
- Resolve every release-blocking diagnostic, fuzzing, security, conformance,
  reproducibility, and performance finding.
- Permit only compatibility fixes during the release-candidate cycle.

## 1.0

Zop 1.0 makes a compatibility promise only after all of these are true:

- The specification has no open decision in the stable language surface.
- Stage 0, stage 1, and stage 2 reproduce from pinned sources without network
  access on every supported host.
- Differential, property, fuzz, ABI, package, and target conformance suites pass
  with published evidence.
- At least one native CPU, one GPU, and the browser profile meet their published
  correctness and performance floors.
- The standard library, package manager, workspaces, cache, formatter, test
  runner, documentation, and diagnostics are release-quality.
- A compiler, tensor framework, browser framework, and production systems
  application exercise the stable surface without private compiler hooks.
- Unsafe boundaries are auditable, safe code is data-race-free, and no known
  soundness defect remains open.

Self-hosting is necessary for 1.0, but self-hosting alone is not maturity.
