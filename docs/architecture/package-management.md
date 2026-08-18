# Packages, imports, and builds

Zop packages are reproducible source units. The package manager resolves
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

Zop does not require a `src/` directory. The package directory is the
source root by default:

```text
rover-navigation/
├── zop.toml
├── zop.lock
├── main.zop
├── navigation.zop
├── navigation/
│   ├── filter.zop
│   └── terrain.zop
└── tests/
```

`navigation.zop` owns the `navigation` module and may export child modules.
Zop has no `mod.rs`, `__init__`, or implicit package file. One source file
defines one module. Package tools reject ambiguous file-to-module mappings.

`source = "code"` may move the module root when a repository already has a
preferred layout. The setting is one normalized path relative to the package
manifest. It must remain inside the package after symbolic links are resolved.
Target entries and module paths are relative to this root. There is no separate
source-end setting: the compiler builds only files reachable from a target's
entry file through imports. A nested `zop.toml` starts another package
boundary, so no file belongs to two packages.

## Imports

Imports use familiar Python spelling, but bind names only during compilation:

```zop
import std.tensor
import geometry.rotation
import geometry.rotation as rotation
from geometry.rotation import Quaternion, rotate
```

The import contract is strict:

- Imports appear only at module scope.
- Importing never executes initialization code.
- Package versions and source locations never appear in source.
- Dependency aliases come from `zop.toml`.
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

`zop.toml` is declarative UTF-8 TOML. It cannot execute code, read the host,
or import another configuration file. Unknown keys are errors. Common package
fields remain at the root so a small project does not begin with configuration
ceremony:

TOML is not a strict superset of JavaScript Object Notation (JSON): JSON has a
`null` value, while TOML has native date and time values plus comment syntax.
Zop chooses TOML because the manifest is human-owned and comments are
useful. The schema stays shallow so nested tables do not obscure common
settings. Missing optional values are omitted rather than written as `null`.

```toml
schema = 1
name = "rover-navigation"
version = "1.4.0"
language = "0.1"
toolchain = "0.1.7"
source = "."

[targets]
app = "main.zop"

[dependencies]
geometry = { package = "nasa/geometry", version = "^3.2" }
control = { package = "nasa/control", version = "^5.1" }

[policy]
unsafe = "deny"
```

Source imports the local aliases `geometry` and `control`. Registry ownership,
version selection, mirrors, and local overrides remain outside source code.
`language` names the source-language contract. `toolchain` pins the exact
compiler distribution used by the root package or workspace. Published
dependencies expose language compatibility, not their development toolchain
pin. `source = "."` is the default and is normally omitted.

## Resolution

The manifest states compatible versions. The resolver selects one version per
package major version. Minor or patch duplicates of the same package are
rejected. Two major versions may coexist only under distinct aliases because
they expose distinct source contracts.

Resolution is deterministic for a fixed set of manifests and registry
metadata. It produces `zop.lock`; it never changes source files. A normal
build neither resolves versions nor rewrites the lockfile.

`zop add` writes a caret requirement such as `^3.2` by default. It accepts
releases in the selected compatibility line and never admits the next breaking
version.
`zop update` changes locked versions only within requirements. Crossing a
declared compatibility boundary requires an explicit
`zop upgrade package@version` command.

The resolver checks every candidate against the pinned toolchain before
selecting it. If no compatible candidate exists, resolution fails. It does not
select code that the pinned compiler cannot build.

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

Zop has no globally additive package features. A package declares typed
configuration values with defaults. Configuration is part of package-instance
identity, so building another workspace member cannot silently change an
existing instance.

## Lockfile

`zop.lock` is canonical generated TOML. Humans may review it, but package
commands own every write. It records the complete build graph:

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

The lockfile has a versioned schema and canonical ordering. It contains no
timestamps, cache paths, or machine-local state. Read-only commands never
rewrite it. Schema migration is an explicit, atomic command whose dependency
diff is reviewable.

## Commands and network boundary

Commands that edit the dependency graph may access configured registries and
are the only commands that rewrite the manifest or lockfile:

```sh
zop add nasa/geometry@3
zop remove geometry
zop resolve
zop update geometry control
zop upgrade geometry@4
```

Fetching is a separate explicit store mutation:

```sh
zop fetch
```

`zop outdated` reports direct dependencies with locked, allowed, and latest
versions. `zop outdated --all` includes transitive dependencies. Package
names are positional arguments to `update`; a repeated per-package flag is not
required. Every graph edit reports direct, transitive, capability, and
toolchain changes. `--dry-run` produces the same report without writing.

Inspection and build commands never change the manifest, lockfile, installed
toolchains, or dependency store. Build outputs and derived caches are their
only writes:

```sh
zop tree
zop outdated
zop build
zop test
zop run
```

Zop has no activatable or mutable project environment. `zop run`
executes a target built from the locked graph. It does not install packages
into a project, user, or system namespace.

Building, packaging, and installing a source artifact use the same locked
dependency graph. Artifact destination does not trigger a second resolution.

If required content is absent, the build names the missing package and asks the
user to fetch or vendor it. It does not contact a registry automatically.

## Toolchain selection

The compiler, package and build driver, `core`, `std`, pinned backend libraries,
and lockfile writer form one signed toolchain release. A project manifest
selects one exact release. Optional target packs are immutable artifacts
recorded in the lockfile, not mutable components added to an installed
compiler.

There is no global active toolchain, hidden directory override, or environment
variable that changes a package build. A stable launcher may select an already
installed release from `zop.toml`. If the release is absent, it fails and
names the explicit command:

```sh
zop toolchain fetch
```

Build, test, run, inspection, and shell-completion commands never download or
modify a toolchain. `zop toolchain pin VERSION` is the only command that
changes a project's toolchain requirement.

Toolchains and target packs enter an immutable content-addressed store through
a temporary path and one atomic publish. Builds treat shared stores as
read-only. Concurrent processes either observe a complete verified artifact or
no artifact; they never repair or mutate one in place.

## Content-addressed storage

Fetched sources, toolchains, target packs, intermediate forms, and compiled
outputs share one immutable content-addressed store. Package and toolchain
indexes point into that store instead of duplicating bytes into separate cache
trees.

The archive format, path normalization, included files, executable bits, and
symbolic-link rules are canonical. The same digest denotes the same typed
object on every host. Mutable tags and branches resolve to an immutable revision
and content digest before entering the lockfile.

The same store may be local, shared by trusted datacenter builders, copied to an
offline machine, or archived with a mission toolchain.

The [artifact and caching contract](artifacts.md) defines action identity, the
`.zop/` project view, materialization, export, integrity, and garbage
collection.

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

## Host, device, and browser targets

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

A browser target emits an optimized ECMAScript module, declarations, and source
maps. It emits a validated WebAssembly module or WebGPU kernel only when target
placement selects a compute region. The exact ECMAScript, browser-interface,
WebAssembly, and WebGPU profiles enter the artifact key. Browser compilation is
a separate target graph, not a Cranelift fallback.

## Workspaces and monorepos

A workspace coordinates multiple packages under one toolchain, lockfile,
policy, and target graph. Members retain explicit package boundaries and direct
dependencies. The [workspace contract](workspaces.md) defines layout, stable
target addresses, version catalogs, sparse builds, and affected tests.

The [Bessemer integration contract](bsmr.md) exposes this graph through a
versioned read-only protocol. BSMR consumes native Zop manifests and
lockfiles without a handwritten build file or a second dependency resolver.

## Vendoring and mission builds

`zop vendor` writes the complete locked source graph into one local tree.
`zop verify` checks sources, toolchains, foreign artifacts, and target data
before compilation. `zop audit` reports dependencies, capabilities,
licenses, withdrawals, and known advisories. `zop sbom` produces a software
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
- Prove build, test, run, tree, and outdated never rewrite project inputs.
- Prove build and test commands perform no network access.
- Keep updates inside declared bounds and require an explicit major upgrade.
- Reject dependencies incompatible with the pinned toolchain.
- Reject unsupported lockfile schemas without rewriting them.
- Fail on a missing toolchain without downloading it.
- Publish concurrently fetched toolchains atomically and verify their hashes.
- Reject a dependency whose content hash or signature differs.
- Reject mutable release sources and ambiguous module paths.
- Resolve the default and configured source roots to the same module names.
- Reject absolute, parent, symbolic-link, and nested-package source escapes.
- Prove an unreachable source file is absent from the build graph.
- Reject package and module import cycles.
- Reject wildcard, parent-directory, and private imports.
- Prove imports execute no runtime initialization.
- Reject undeclared build-action inputs, outputs, tools, and capabilities.
- Rebuild a target from only its vendored mission archive.
- Produce identical artifacts after normalizing paths and timestamps.
- Reject host/device imports and artifacts at the wrong target boundary.
- Reject browser capabilities and artifacts at native or device boundaries.

## References

- [TOML 1.0 specification](https://toml.io/en/v1.0.0)
- [JSON specification](https://www.rfc-editor.org/rfc/rfc8259)
- [Cargo manifests and lockfiles](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)
- [Cargo locked and offline builds](https://doc.rust-lang.org/cargo/commands/cargo-build.html)
- [Cargo dependency and feature resolution](https://doc.rust-lang.org/cargo/reference/resolver.html)
- [Cargo workspace feature-selection issue](https://github.com/rust-lang/cargo/issues/4463)
- [Cargo install lockfile issue](https://github.com/rust-lang/cargo/issues/7169)
- [Cargo locked-by-default proposal](https://github.com/rust-lang/cargo/issues/8207)
- [Rustup toolchain selection](https://rust-lang.github.io/rustup/overrides.html)
- [Rustup process-safety goal](https://rust-lang.github.io/rust-project-goals/2026/process-safe-rustup.html)
- [uv locking and syncing](https://docs.astral.sh/uv/concepts/projects/sync/)
- [uv universal resolution](https://docs.astral.sh/uv/concepts/resolution/)
- [Community feedback on uv maintenance commands](https://www.loopwerk.io/articles/2026/uv-ux-mess/)
- [Go module resolution, checksums, and vendoring](https://go.dev/ref/mod)
- [Zig package content hashing](https://ziglang.org/download/0.16.0/release-notes.html#build-system)
- [Zig package metadata rationale](https://ziglang.org/download/0.11.0/release-notes.html#Package-Management)
- [Zig build graph and cross-compilation](https://ziglang.org/learn/build-system/)
