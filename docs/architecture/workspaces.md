# Workspaces and monorepos

Zop treats a monorepo as one typed build graph. Packages remain independent
units, while the workspace owns the toolchain, lockfile, policy, and graph-wide
commands.

> **Status:** This page defines target semantics. Workspace discovery, target
> addressing, graph queries, and affected builds are not implemented.

## Layout

A workspace root is coordination only. It is not also a package:

```text
mission-control/
├── zop.toml
├── zop.lock
├── apps/
│   └── inference/
│       ├── zop.toml
│       └── main.zop
└── packages/
    ├── tensor/
    │   ├── zop.toml
    │   └── lib.zop
    └── navigation/
        ├── zop.toml
        └── lib.zop
```

The root manifest lists members relative to itself:

```toml
schema = 1
toolchain = "0.1.7"

[workspace]
members = ["apps/*", "packages/*"]
default = ["//apps/inference:app"]
```

Member patterns discover package manifests, not arbitrary source directories.
An unmatched pattern is an error. Resolution writes the canonical member list
and each manifest digest to `zop.lock`.

A package belongs to at most one workspace. A repository may contain multiple
disjoint workspaces, but workspaces cannot overlap or nest. One workspace uses
one exact toolchain and one lockfile.

## Source ownership

Each member owns the files below its package manifest, except files below a
nested package manifest. No file belongs to two packages. A member uses its
directory as the source root unless it sets `source` explicitly.

Targets declare entry files. Imports determine the source closure, which is the
complete set of files reachable from an entry file. Zop therefore needs no
per-directory build files, source globs, or repeated source lists.

## Target addresses

Every target has one stable workspace-relative address:

```text
//apps/inference:app
//packages/tensor:lib
```

`//` means the workspace root, the path names a member package, and the suffix
names a target. `:app` is the package-local shorthand. `//packages/...` selects
all targets below that path; `//...` selects the complete workspace.

Explicit addresses mean the same thing from every directory:

```sh
zop build //apps/inference:app
zop test //packages/...
zop test //...
```

Inside a member directory, an omitted address selects that member's default
target. At the workspace root, an omitted address uses `workspace.default`. If
no default is declared, Zop asks for a target instead of building the whole
monorepo accidentally.

## Dependencies and catalogs

Workspace membership does not grant dependency access. Every member declares
its direct package dependencies, and the compiler rejects imports through an
undeclared transitive dependency.

The root may provide a version catalog to keep shared dependency versions in
one place:

```toml
[catalog]
tensor = { path = "packages/tensor", version = "^0.1" }
geometry = { package = "nasa/geometry", version = "^3.2" }
```

A member opts into an entry explicitly:

```toml
[dependencies]
tensor = { catalog = "tensor" }
geometry = { catalog = "geometry" }
```

A catalog entry never injects a dependency. It only supplies the source and
version for an edge that a member declared. Publishing replaces workspace paths
with immutable package identities and rejects unpublished local edges.

## Graph operations

The workspace graph combines declared package edges with source imports and
target relationships. Build, test, documentation, and query commands operate
on the selected target closure rather than scanning every member.

```sh
zop query deps //apps/inference:app
zop query rdeps //packages/tensor:lib
zop test --affected changed/file.zop another/file.zop
```

`deps` returns the transitive inputs of a target. `rdeps` returns targets that
depend on it. `--affected` accepts changed paths and selects targets whose
source closure or configuration includes those paths. Source-control tools may
supply the paths; Zop does not assume Git or another repository system.

The lockfile records enough member metadata to validate one selected closure
without reading unrelated member sources. Sparse checkouts, container layers,
and remote builders can therefore build a target when its complete closure is
present, even if the rest of the monorepo is absent.

## Policy and visibility

The workspace root owns toolchain, registry, build, and safety policy. Members
may strengthen policy but cannot weaken it. A member that requires a different
toolchain belongs in a different workspace.

Workspace membership grants no language visibility. Packages expose only
public declarations, and targets may depend only on declared packages. These
rules preserve architectural boundaries as the repository grows.

## Performance contract

Graph loading is proportional to workspace manifests and selected target
metadata, not all source text. Parsing and compilation load source only for the
selected closure. Public-interface hashes prevent implementation-only changes
from rechecking or recompiling unaffected downstream source. A final link still
runs when one of its object inputs changes.

Local and remote caches use the same content keys defined by the
[artifact and caching contract](artifacts.md). Concurrent targets share work
without sharing mutable build state. One `.zop/` view belongs to the
workspace root; members never create nested views.

The target-address and selected-closure contracts map directly to the
[native Bessemer integration](bsmr.md). BSMR may schedule the graph, but it
cannot change workspace membership or Zop dependency semantics.

## Required tests

- Resolve member patterns to a canonical, byte-identical member list.
- Reject package fields in a workspace-root manifest.
- Reject unmatched members, nested workspaces, and overlapping packages.
- Resolve one explicit target address identically from every directory.
- Require an explicit root target when no workspace default exists.
- Reject imports through undeclared direct or transitive dependencies.
- Prove a catalog entry does not inject a dependency.
- Prove member selection cannot alter another member's package configuration.
- Build a selected closure with unrelated member source trees absent.
- Return exact dependency, reverse-dependency, and affected target sets.
- Reject member attempts to weaken workspace policy or change its toolchain.
- Reuse downstream artifacts after an implementation-only dependency change.

## References

- [Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Go workspaces](https://go.dev/ref/mod#workspaces)
- [Bazel packages and targets](https://bazel.build/concepts/build-ref)
- [Bazel target labels](https://bazel.build/tutorials/cpp-labels)
- [Bazel dependency graphs](https://bazel.build/concepts/dependencies)
