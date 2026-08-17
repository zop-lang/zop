# Packages, imports, and builds

Bedrock packages are reproducible source units. The package manager resolves
and fetches dependencies before compilation. The compiler builds only from a
complete local dependency graph.

> **Status:** This page defines target semantics. The Rust bootstrap does not
> implement modules, dependency resolution, or package commands yet. Exact
> manifest spelling may change without changing these contracts.

## Goals

The same package system must support two extremes:

- fast incremental and remotely cached datacenter builds;
- fully offline, auditable reconstruction for embedded and mission software.

Resolution may use the network. Compilation never does. A missing dependency,
tool, target artifact, or lock entry stops the build with a diagnostic.

## Package layout

A package has one manifest, one lockfile, and explicit source roots:

```text
rover-navigation/
├── bedrock.toml
├── bedrock.lock
├── src/
│   ├── lib.br
│   ├── navigation.br
│   └── navigation/
│       ├── filter.br
│       └── terrain.br
└── tests/
```

`navigation.br` owns the `navigation` module and may export child modules.
Bedrock has no `mod.rs`, `__init__`, or implicit package file. One source file
defines one module. Package tools reject ambiguous file-to-module mappings.

## Imports

Imports bind names during compilation:

```bedrock
use std.tensor
use geometry.rotation
use geometry.rotation as rotation
use geometry.rotation: Quaternion, rotate
```

The import contract is strict:

- Imports appear only at module scope.
- Importing never executes initialization code.
- Package versions and source locations never appear in source.
- Dependency aliases come from `bedrock.toml`.
- Wildcard and parent-directory imports are not valid.
- Package and module import graphs are acyclic.
- Imported declarations remain qualified unless selected explicitly.
- A dependency exposes only its declared public modules and declarations.
- An import grants no `Mem`, `Io`, `unsafe`, host, or device capability.
- A `kn` call graph cannot import host-only operations.

Module constants are pure compile-time values. Runtime initialization is an
ordinary function called explicitly from the program entrypoint. There is no
module initialization order to observe.

## Manifest

`bedrock.toml` declares package identity, source targets, dependency
constraints, and build policy:

```toml
[package]
name = "rover-navigation"
version = "1.4.0"
language = "0.1"

[targets]
library = "src/lib.br"

[dependencies]
geometry = { package = "nasa/geometry", version = "^3.2" }
control = { package = "nasa/control", version = "^5.1" }

[policy]
unsafe = "deny"
```

Source imports the local aliases `geometry` and `control`. Registry ownership,
version selection, mirrors, and local overrides remain outside source code.

## Resolution

The manifest states compatible versions. The resolver selects one version per
package major version. Minor or patch duplicates of the same package are
rejected. Two major versions may coexist only under distinct aliases because
they expose distinct source contracts.

Resolution is deterministic for a fixed set of manifests and registry
metadata. It produces `bedrock.lock`; it never changes source files. A normal
build neither resolves versions nor rewrites the lockfile.

Dependency configuration belongs to one explicit dependency edge:

```toml
geometry = {
    package = "nasa/geometry",
    version = "^3.2",
    config = { precision = "f64" },
}
```

Configurations do not merge globally. Two incompatible configurations require
two aliases and remain visibly different package instances. Target selection
uses compiler-known target facts instead of arbitrary environment variables.

## Lockfile

`bedrock.lock` records the complete build graph:

- package identity and exact version;
- immutable source identity;
- canonical source-content hash;
- dependency edges and selected configuration;
- publisher identity and signature;
- required language and toolchain versions.

Applications and workspaces commit the lockfile. Libraries also commit one for
their own development and tests; downstream resolution uses the published
manifest constraints instead of the library's private lock.

A package release is immutable. A registry may mark it withdrawn, but it cannot
replace the bytes associated with its version and content hash.

## Commands and network boundary

Dependency commands may access configured registries and mirrors:

```sh
bedrock add nasa/geometry@3
bedrock resolve
bedrock fetch
bedrock update geometry
```

Build commands use local immutable inputs only:

```sh
bedrock build
bedrock test
bedrock build --locked --offline
```

If required content is absent, the build names the missing package and asks the
user to fetch or vendor it. It does not contact a registry automatically.

## Content-addressed storage

Fetched sources live in an immutable store keyed by canonical content hash:

```text
$BEDROCK_CACHE/packages/<content-hash>/
```

The archive format, path normalization, included files, executable bits, and
symbolic-link rules are canonical. The same hash denotes the same source tree
on every host. Mutable tags and branches resolve to an immutable revision and
content hash before entering the lockfile.

The same store may be local, shared by trusted datacenter builders, copied to an
offline machine, or archived with a mission toolchain.

## Hermetic build graph

Package targets form a directed acyclic graph. Independent steps run in
parallel. Build artifacts are keyed by:

- package source and dependency hashes;
- compiler and standard-library hash;
- target architecture, operating system, and application binary interface;
- central processing unit features;
- graphics processing unit architecture;
- optimization and safety profile;
- relevant `known` arguments.

An interface hash allows downstream packages to reuse checked high-level
intermediate representation when a dependency changes without changing its
public contract.

## Build actions

Packages cannot run unrestricted install or build scripts. Ordinary targets
are declarative. If generated code or a foreign toolchain is required, a
sandboxed build action declares:

- one pinned executable and content hash;
- all input files and environment values;
- all outputs;
- required capabilities;
- target constraints;
- a network prohibition.

Undeclared reads, writes, processes, or network access fail the action. Outputs
are hashed before later build steps consume them. Failure never switches to a
system-installed tool or library.

## Host and device targets

One package may expose central processing unit (CPU) `fn` code and graphics
processing unit (GPU) `kn` kernels. Resolution locks one source graph;
compilation derives separate legal host and device graphs.

Host artifacts include runtime calls and Cranelift code. Device artifacts
include legal Multi-Level Intermediate Representation (MLIR) operations and
target images. Each artifact names its exact target. A host call cannot launch
an image for an incompatible device, and device code cannot import host `Io`,
filesystem, network, or pointer interfaces.

Static linking is the default for embedded and mission targets. Dynamic linking
requires a separate versioned application binary interface (ABI) contract.

## Workspaces

A workspace contains multiple local packages and one lockfile:

```toml
[workspace]
members = [
    "services/inference",
    "packages/tensor",
    "packages/runtime",
]
```

Path dependencies are workspace-local development inputs. Publishing resolves
or vendors them as immutable package sources. A release build rejects an
unlocked path dependency.

## Vendoring and mission builds

`bedrock vendor` writes the complete locked source graph into one local tree.
`bedrock verify` checks sources, toolchains, foreign artifacts, and target data
before compilation. `bedrock audit` reports dependencies, capabilities,
licenses, withdrawals, and known advisories. `bedrock sbom` produces a software
bill of materials.

A mission build archive contains:

- compiler source and binary;
- standard-library source;
- manifest and lockfile;
- every vendored dependency;
- pinned foreign tools and artifacts;
- target description and build policy;
- expected source and output hashes.

The reconstruction uses no network, undeclared environment, mutable dependency,
arbitrary script, host path, timestamp, or undeclared dynamic library.
Reproducibility does not prove program correctness, but it makes the audited
program recoverable and verifiable.

## Required tests

- Resolve identical manifests to byte-identical lockfiles.
- Reject a build that would change or omit the lockfile.
- Prove build and test commands perform no network access.
- Reject a dependency whose content hash or signature differs.
- Reject mutable release sources and ambiguous module paths.
- Reject package and module import cycles.
- Reject wildcard, parent-directory, and private imports.
- Prove imports execute no runtime initialization.
- Reject undeclared build-action inputs, outputs, tools, and capabilities.
- Rebuild a target from only its vendored mission archive.
- Produce identical artifacts after normalizing paths and timestamps.
- Reject host/device imports and artifacts at the wrong target boundary.

## References

- [Cargo manifests and lockfiles](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)
- [Cargo locked and offline builds](https://doc.rust-lang.org/cargo/commands/cargo-build.html)
- [Go module resolution, checksums, and vendoring](https://go.dev/ref/mod)
- [Zig package content hashing](https://ziglang.org/download/0.16.0/release-notes.html#build-system)
- [Zig package metadata rationale](https://ziglang.org/download/0.11.0/release-notes.html#Package-Management)
- [Zig build graph and cross-compilation](https://ziglang.org/learn/build-system/)
