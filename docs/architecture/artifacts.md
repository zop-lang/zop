# Artifacts, caching, and materialization

Zop separates immutable content, reusable action results, project-local
views, and user-owned exports. A cache may improve latency; it never supplies
missing semantics or changes which program is built.

> **Status:** This page defines target semantics. The Rust bootstrap writes only
> explicitly requested native objects and does not implement the store, action
> cache, project view, remote cache, or garbage collector.

## Layers

**Committed truth** is source, `zop.toml`, `zop.lock`, vendored inputs,
and explicit policy. A build can be reconstructed from these files and the
pinned toolchain.

**The immutable store** contains source trees, toolchains, target packs,
intermediate representation, objects, device images, WebAssembly modules, and
documentation models, rendered reference sites, books, and other artifacts
addressed by content digest.

**The action cache** maps a complete action digest to the immutable output-tree
digest produced by that action.

**The project view** exposes selected outputs at convenient paths without
becoming their source of truth.

**An export** is an explicit copy of a final artifact into a caller-owned path
such as `dist/`, a release directory, or an installation prefix.

## Project-local state

The project directory is `.zop/` at the workspace root:

```text
.zop/
├── out/       # Read-only views of selected build outputs.
├── roots/     # Content digests retained for this workspace.
├── logs/      # Bounded structured reports for recent commands.
└── tmp/       # Interrupted or in-progress local materialization.
```

Member packages never create nested `.zop/` directories. The workspace
root owns one view and one set of retention roots.

Everything below `.zop/` is ignored by version control and disposable.
Deleting it does not alter source, the lockfile, installed toolchains, or the
global store. The next command recreates only the views and temporary state it
needs.

`zop build` materializes selected outputs below `.zop/out/` and reports
their paths and content digests. The layout is a user convenience, not a stable
linker or package interface. Tools consume the reported artifact record rather
than guessing paths.

Project views are read-only. An implementation may use a symbolic link, hard
link, copy-on-write clone, or verified copy, but every form exposes the same
bytes. Editing a view never edits the immutable store.

## User-level store

The default store lives in the operating system's user cache directory under
the current tool name. Its logical layout is:

```text
<user-cache>/<tool>/
├── cas/
│   ├── blobs/<algorithm>/<digest>
│   └── trees/<algorithm>/<digest>
├── actions/<algorithm>/<digest>
├── indexes/
├── pins/
└── tmp/
```

Package, toolchain, and artifact indexes point into one content-addressed store
(CAS); they do not duplicate the bytes into separate caches. A shared or remote
store uses the same object and action identities.

The human-visible `Zop`, `.zop`, and `ZOP_*` environment-variable names are the
current public contract. Store schemas and digest domains remain versioned
independently from the brand, so an explicit future migration would not
invalidate immutable content.

## Action identity

For canonical action specification `A` and declared input tree `I`:

```text
action_digest = H(canonical_encode(A, I))
action_cache[action_digest] = output_tree_digest
```

`A` contains the executable, arguments, environment, target and execution
platforms, optimization, safety, and floating-point profiles, output contract,
CPU feature-variant set, required-vector assertions, debug instrumentation such
as `--check-nonfinite`, timeout, and rule implementation. `I` contains source,
generated inputs, selected dependencies, toolchains, target packs, and foreign
artifacts. The selected schedule and its vectorization report are outputs; their
content digests expose any change without pretending a derived decision was an
action input.

The complete workspace or lockfile digest does not salt an unrelated action.
Only the selected dependency closure and semantic inputs enter its key. A
private implementation edit may therefore reuse downstream type checking while
still relinking any final artifact whose object input changed.

A cache hit is a correctness claim: the same complete inputs must produce the
recorded output tree. The tool exposes cache-key explanations so a developer can
see which input caused a miss.

## Atomicity and integrity

Writers create an object in a temporary path on the destination filesystem,
verify its canonical digest and shape, make it read-only, then publish it
atomically. Readers never observe a partial tree.

Concurrent writers for the same digest may duplicate work, but they converge on
one identical object. A running action holds leases on its inputs and outputs so
garbage collection cannot remove them.

A missing object is a cache miss. The declared action may execute normally. An
object whose bytes disagree with its claimed digest is corruption: the command
fails with a typed diagnostic and does not hide the event by rebuilding or
fetching another implementation.

## Materialization and export

Materialization makes an immutable output visible. It does not change artifact
identity. If a project view is deleted, Zop restores it from CAS without
running the compiler when the action result remains available.

An explicit output option exports a final artifact:

```sh
zop build //apps/site:web --out dist
zop doc //packages/tensor --out site
```

The export is written to a sibling temporary path and renamed atomically. It is
caller-owned and may be edited or packaged, but modified exports never reenter
the cache without a new declared action.

`zop run` may execute a materialized immutable artifact directly. JIT code
memory remains process-owned and is never treated as a durable artifact.

## Cleaning and garbage collection

Commands distinguish project cleanup from global reclamation:

```sh
zop clean
zop cache status
zop cache verify
zop cache gc --dry-run
zop cache gc
```

`zop clean` removes only `.zop/` views, logs, temporary files, and their
retention roots. It never removes downloaded packages, installed toolchains, or
global CAS objects.

Cleanup resolves one workspace root and refuses a `.zop` path that is
itself a symbolic link. It unlinks nested views without following them into the
store or another directory.

Global garbage collection traces from explicit pins, installed toolchains,
active leases, retained action results, and live project roots. Unreachable
cache objects are eligible for size- and recency-based deletion. Dry-run output
lists the exact objects and bytes before deletion.

Vendored mission archives and explicitly pinned artifacts are not cache entries
and are never selected for cache garbage collection. Automatic collection may
enforce a user-configured size budget, but it never runs inside a compiler
action or removes a pinned or leased object.

## Local and remote reuse

The standalone Zop driver owns its local action cache and CAS. When BSMR
drives a build, Zop compiler action mode ignores project and user caches;
BSMR owns action identity, CAS, materialization, remote reuse, and reporting.
Both paths must produce identical artifact digests.

A remote miss permits execution of the same declared action. A remote digest or
provenance mismatch is an integrity failure, not a miss. Upload remains disabled
until toolchain, platform, path, environment, and sandbox identities are
portable and enforced.

## Required tests

- Rebuild successfully after deleting `.zop/`.
- Rebuild from locked and vendored inputs after deleting the user cache.
- Restore a deleted project view without executing its producing action.
- Prove project-view mutation cannot change an immutable CAS object.
- Produce equal action and artifact digests across worktrees and machines.
- Explain every cache miss by one changed semantic input.
- Change the action digest when the floating-point profile changes.
- Change the action digest when CPU feature variants or required-vector
  assertions change, and content-address the selected schedule report.
- Change the action and artifact digests when nonfinite instrumentation changes.
- Prove unrelated lockfile changes do not invalidate an action closure.
- Publish concurrent identical objects atomically without partial readers.
- Reject corrupt local and remote objects instead of rebuilding silently.
- Keep pinned and leased objects live through garbage collection.
- Prove dry-run and actual garbage collection select the same object set.
- Keep `zop clean` confined to the selected workspace's local state.
- Reject a symbolic-link project-state root and never follow nested view links.
- Produce equal standalone and BSMR artifact digests.
- Restore documentation models, reference sites, and books without rechecking
  unchanged source or examples.

## References

- [Cargo build cache](https://doc.rust-lang.org/cargo/reference/build-cache.html)
- [Go build and test caching](https://pkg.go.dev/cmd/go#hdr-Build_and_test_caching)
- [Bazel output directories](https://bazel.build/remote/output-directories)
- [Nix garbage-collector roots](https://nix.dev/manual/nix/stable/package-management/garbage-collector-roots)
- [BSMR caching contract](https://github.com/dedalus-labs/bsmr/blob/c89ca926c4af24b6d0e2f20ed0b907cea6be14ba/docs/concepts/caching.md)
- [BSMR hermetic build core](https://github.com/dedalus-labs/bsmr/blob/c89ca926c4af24b6d0e2f20ed0b907cea6be14ba/docs/concepts/hermetic_build_core.md)
