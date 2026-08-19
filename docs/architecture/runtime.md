# Runtime

The runtime provides services that generated machine code cannot own alone.
It stays behind a small native application binary interface (ABI) shared by
just-in-time (JIT) and ahead-of-time (AOT) code.

> **Status:** This page defines the target runtime contract. The scalar Rust
> bootstrap does not implement `Mem`, tensor descriptors, input/output services,
> or the stable native ABI described here.

## Responsibilities

The runtime owns:

- Process entry and termination.
- Storage requested through `Mem` by generated code.
- Tensor storage and lifetime operations.
- Standard input and output.
- The selected explicit input/output implementation.
- Registration of symbols callable from JIT code.

The runtime does not parse source, optimize programs, select compiler modes,
or repair unsupported code generation.

## Tensor boundary

A tensor value pairs a CuTe-native `Engine` with a language-native
[Layout](layouts.md). The Engine owns or views an offsettable data iterator.
Layout maps logical coordinates to Engine indices. Both remain inspectable in
source, while element type and logical Shape remain type facts.

Tensor `.shape`, `.rank`, `extent axis=`, and `numel()` are logical queries,
not independent mutable metadata. Static queries fold away. Dynamic queries
read or derive only the dynamic shape leaves required by the
[indexing descriptor contract](indexing.md#runtime-representation).

The exact ABI field packing remains open until the fixed-rank matrix
multiplication slice supplies executable evidence. Its semantic contents are
already constrained: the value materializes only dynamic Engine state, required
dynamic Layout leaves, placement, and ownership state. It stores no separate
origin or redundant logical count, and static Engine and Layout profiles add no
runtime field. The [CuTe-native ABI contract](layouts.md#runtime-and-application-binary-interface)
defines Engine kinds, profiles, hierarchy, dynamic leaves, and foreign-boundary
conversion.

The application binary interface does not impose two full arrays of sizes and
strides on every tensor. CPU, GPU, and foreign boundaries must prove the same
coordinate map while using the smallest representation their static facts
permit.

The runtime ABI must state who allocates every buffer, who frees it, and how a
callee may retain it. Generated code must not infer ownership from a pointer.

The [memory-management contract](memory.md) defines ownership, borrowing,
destruction, shared values, arenas, and optional managed regions.

The [explicit input/output contract](io.md) defines capability passing, buffering,
flush and close obligations, asynchronous work, cancellation, and test
implementations.

## Errors and symbols

Language errors follow the typed [`fails` contract](errors.md) and remain
ordinary values. Runtime faults such as `mem.alloc` failure return through a
declared error channel or terminate according to a documented contract. They
do not unwind as language exceptions or silently switch execution modes.

Ordinary integer overflow, zero divisors for `//` or `%`, signed division
overflow, and invalid shifts enter the language trap path in every build mode.
A trap does not unwind and cannot be caught as a typed language error. Floating
`/` follows its selected profile and does not use this trap path. Recoverable
arithmetic uses fallible numeric members such as `add` and `divide`; their
declared errors use the ordinary typed failure channel. See the
[numeric contract](numerics.md).

JIT compilation registers every permitted runtime symbol before finalization.
The compiler rejects undeclared imports instead of searching the host process
for a similarly named function.

## Traps and execution domains

An execution domain is the smallest runtime instance that Zop invalidates after
an unrecoverable language or target fault. The domain is the native process for
ordinary CPU execution, one Zop application or WebAssembly instance for an
embedded browser region, and one device execution context for a GPU target.

A trap terminates its domain. It does not unwind, run fallible cleanup, enter a
`catch`, or promise transactional rollback. Writes and external effects that
completed before the trap may exist, but no safe Zop execution inside that
domain continues to observe them. An embedding host may report or restart a
terminated domain; it cannot resume the trapped Zop stack.

Shared memory, memory-mapped files, device-visible buffers, and external I/O may
remain observable outside the failed domain. Code that requires atomic durable
state uses an explicit transaction, journal, copy-on-write update, or protocol
commit. A language trap is never a persistence transaction.

This contract keeps ordinary checked arithmetic practical. A scalar compound
update computes and checks one result before its store. A tensor update may have
written a subset of elements before one lane traps, but rolling back those
writes would require a hidden preflight pass or temporary tensor. Terminating
the domain makes the partial value unusable instead.

An asynchronous device trap is observed by the host through the launch or
completion error channel. The runtime marks the complete device execution
context failed. Every tensor, stream, event, module, and allocation associated
with that context becomes invalid, including values not passed to the failed
kernel. Their handles may be discarded or expose saved metadata for diagnostics,
but they cannot read, copy, launch, or reuse device storage.

Host recovery creates a fresh context and explicitly reconstructs or uploads
every required value. The runtime never aliases an invalid allocation into the
new context, retries the kernel on another backend, or presents a partially
written output as valid.

A target that cannot detect a required trap and enforce domain invalidation
rejects the operation. It may not replace trapping integer arithmetic with
wrapping arithmetic or convert a device fault into NaN.

## Required tests

- Terminate an isolated native process on every integer trap without unwinding
  or executing later source.
- Prove scalar compound assignment stores nothing when its operation traps.
- Permit a tensor update to avoid rollback storage while preventing every later
  observation after its domain traps.
- Terminate one browser application or WebAssembly instance without allowing
  its host adapter to resume the trapped stack.
- Distinguish pre-execution device launch rejection from an execution-time
  fault.
- Invalidate every allocation and storage operation in a failed device context,
  including values unrelated to the trapping kernel.
- Preserve saved type, shape, and layout metadata for fault diagnostics without
  permitting storage access.
- Recreate a usable device context only through explicit construction and data
  upload.
- Prove no trap retries through a different backend or changes numeric policy.

## References

- [CUDA device assertions](https://docs.nvidia.com/cuda/cuda-programming-guide/05-appendices/cpp-language-extensions.html#assertion)
- [CUDA context invalidation](https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__TYPES.html)
