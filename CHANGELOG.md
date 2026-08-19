# Changelog

All notable user-visible changes to Zop are recorded here. Zop follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) once a surface is
declared stable; before 1.0, each release documents its compatibility boundary.

## Unreleased

- Bootstrap the Rust stage-0 frontend, typed HIR, MLIR verification, Cranelift
  execution and object emission, and deterministic ECMAScript subset.
- Establish the canonical language, tensor-layout, target, tooling,
  self-hosting, and release contracts.
- Preserve source-order named calls through HIR and native and JavaScript
  lowering, including calling-convention parameter placement.
- Reject ambiguous same-line expressions, delimiter-hidden indentation tabs,
  nonnumeric ordering, and nested returns unsupported by bootstrap HIR; retain
  unit-valued native bindings and source-backed call diagnostics.
