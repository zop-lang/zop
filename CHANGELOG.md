# Changelog

All notable user-visible changes to Zop are recorded here. Zop follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) once a surface is
declared stable; before 1.0, each release documents its compatibility boundary.

## Unreleased

- Set the unreleased bootstrap package version to `0.0.1` and license
  project-owned work under Apache-2.0.
- Bootstrap the Rust stage-0 frontend, typed HIR, MLIR verification, Cranelift
  execution and object emission, and deterministic ECMAScript subset.
- Make MLIR a first-class compiler layer with verifier-gated canonicalization
  and common-subexpression elimination before backend translation.
- Reserve `0.0.x` for internal checkpoints and require an installable, usable
  scalar language before the first public `0.1.0` binary release.
- Add executable affine and composed Layout evaluation and slicing semantics.
- Establish the canonical language, tensor-layout, target, tooling,
  self-hosting, and release contracts.
- Preserve source-order named calls through HIR and native and JavaScript
  lowering, including calling-convention parameter placement.
- Reject ambiguous same-line expressions, delimiter-hidden indentation tabs,
  nonnumeric ordering, and nested returns unsupported by bootstrap HIR; retain
  unit-valued native bindings and source-backed call diagnostics.
