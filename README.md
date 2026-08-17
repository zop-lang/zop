# Bedrock

![Bedrock Logo](./bedrock.webp)

**A strong systems language first. Tensor-native by design.**

Bedrock is an experimental systems language. Tensor operations should feel
native enough that machine-learning frameworks grow directly from ordinary
Bedrock code, but machine learning is not its only job. Performance is a
premium. JAX-like purity is too, but we are not functional evangelists.

The compiler should run as fast as it can. Source should read like executable
pseudocode so systems programs and research ideas are easy to whiteboard and
quick to learn.

## Design goals

- Strong systems semantics and native performance.
- Tensor operations that are natural enough to build frameworks from.
- Compiler-generated autodiff with explicit gradient values.
- Purity where it enables optimization, without banning mutation.
- Explicit `Mem` and `Io` capabilities without hidden global state.
- Fast compilation for scripts, research, and production builds.
- Executable-pseudocode syntax with succinct Python- and English-like phrasing.
- Explicit central processing unit (CPU) and graphics processing unit (GPU)
  targets built on Multi-Level Intermediate Representation (MLIR).

## Architecture

The target compiler uses MLIR for tensor optimization and Cranelift for native
CPU code generation. See the
[architecture documentation](docs/architecture/README.md) for the compiler
boundaries and current implementation scope.

## Compiler bootstrap

The current Rust bootstrap implements indentation-aware lexing, parsing,
scalar name and type checking, typed high-level intermediate representation,
verified MLIR, and Cranelift just-in-time and object code generation. The
executable scalar backend deliberately accepts only host `fn` code over `i64`
while the contracts for tensors, ownership, effects, errors, and GPU kernels
mature.

Building requires Rust 1.95 and LLVM/MLIR 22. On macOS, `brew install llvm`
provides the required toolchain.

```sh
cargo test --all-targets
cargo run -- mlir examples/answer.br
cargo run -- object examples/answer.br answer.o
```

## Contributing

Open an issue before implementing a new language or compiler contract.

## History

The original [`whitepaper.txt`](docs/history/whitepaper.txt) is preserved
verbatim as the source of Bedrock's first ideas.

The [design lineage](docs/architecture/design-lineage.md) credits the language
and compiler projects that inform the current architecture.

## License

Bedrock is licensed under the terms of the [MIT License](LICENSE).
