# Standard library

Bedrock ships a small mandatory core and a modular standard library. The core
defines contracts that every target needs. Standard modules add allocation,
input/output, concurrency, tensors, testing, and platform integration through
explicit capabilities.

> **Status:** This page defines target semantics. The Rust bootstrap does not
> implement Bedrock modules or a Bedrock standard library yet.

## Boundary

The distribution has three rings:

| Ring | Versioning | Purpose |
| --- | --- | --- |
| `core` and `std` | Bundled with the compiler | Stable language and runtime contracts |
| Official packages | Independent releases | Maintained batteries that may evolve faster |
| Community packages | Independent releases | Frameworks, applications, and domain libraries |

The standard library prevents foundational types and protocols from
fragmenting across packages. It does not select one application framework for
the ecosystem.

## Modules

| Module | Owns | Required capability |
| --- | --- | --- |
| `core` | Primitive contracts, tuples, records, errors, ranges, views, pointers, pure algorithms | None |
| `std.mem` | Owned strings, buffers, lists, maps, sets, boxes, arenas, and shared ownership | `Mem` |
| `std.tensor` | Tensor construction, transformations, reductions, and fundamental linear algebra | `Mem` when allocating |
| `std.task` | Structured task scopes, groups, bounded channels, and completion handles | `Io` for scheduling |
| `std.io` | Readers, writers, formatting, files, networking, time, entropy, and processes | `Io` |
| `std.sync` | Threads, locks, events, semaphores, and atomics | `Io` for blocking operations |
| `std.gpu` | Device identity, address spaces, barriers, atomics, and kernel operations | `kn` only |
| `std.sys` | Unsafe pointers, foreign interfaces, volatile access, and application binary interfaces (ABIs) | `unsafe` |
| `std.test` | Assertions, fixtures, property tests, fuzzing, and benchmarks | Test builds |

Imports do not grant a capability. A function must still receive `Mem` or `Io`,
and a device operation remains illegal outside a compatible `kn`.

## Core

`core` is always available. It links no operating-system service and performs
no hidden allocation or input/output. It contains:

- primitive numeric and boolean contracts;
- tuples, records, optional values, and typed failures;
- ranges and iteration interfaces;
- borrowed strings, bytes, views, and slices;
- ownership operations such as `give` and `drop`;
- size, alignment, and type-layout queries;
- raw-pointer contracts;
- pure numeric functions;
- target-neutral algorithms;
- compiler-known tensor primitives.

Bare-metal programs and graphics processing unit (GPU) kernels may use the
legal subset of `core` without linking the hosted standard library.

## Prelude

The prelude is the small set of names available without an import. It contains
only primitive types and operations used by nearly every program. Adding a name
to the prelude is a language-compatibility decision.

Owned collections, input/output functions, concurrency types, platform APIs,
and framework names are never imported implicitly. `print` is a standard
input/output function and requires `Io`; it is not a compiler statement or
ambient global operation.

## Allocation

Allocation-capable APIs use the [memory contract](memory.md). A collection
records the `Mem` origin responsible for its storage. The type checker prevents
the collection from outliving that origin.

Construction makes allocation authority explicit. Growth and reallocation
remain fallible in the collection method's error type. No standard collection
silently switches to a global or system allocator.

The initial allocating types are deliberately few:

- one owned Unicode Transformation Format 8 (UTF-8) string;
- one owned byte buffer;
- one contiguous growable list;
- one standard hash map;
- one ordered map and set family;
- one unique-owner box;
- explicit shared and weak ownership;
- arenas for grouped lifetimes.

Alternative collection layouts belong in packages until multiple unrelated
consumers prove that one must become a stable interoperability contract.

## Algorithms

Universal, target-neutral algorithms belong in `core.algorithm` when competing
implementations would add bugs rather than useful choice. The initial set
includes:

- linear and binary search;
- partition points and ranges;
- minimum, maximum, and clamp;
- sort, stable sort, and selection;
- partition, merge, and heap operations;
- copy, move, fill, swap, and comparison;
- scans and reductions.

Allocation-free pure algorithms require neither `Mem` nor `Io`. Algorithms
that need temporary workspace accept a caller-provided view or `Mem`. A
target-neutral implementation may compile for both central processing unit
(CPU) `fn` and GPU `kn` code.

### Binary search

The standard search family defines deterministic duplicate behavior:

- `lower_bound` returns the first position not less than the target;
- `upper_bound` returns the first position greater than the target;
- `equal_range` returns both bounds;
- `binary_search` returns the first equal position or the lower insertion
  position.

Search handles empty inputs and computes midpoints without integer overflow.
The input must be ordered consistently with the selected comparison. A future
sorted-view type may preserve that proof; the first API states and tests the
precondition directly.

## Tensors

The tensor type, shape rules, placement, and fundamental operations are stable
language and standard-library contracts. Public functions wrap private
compiler intrinsics when an operation requires direct Multi-Level Intermediate
Representation (MLIR) lowering:

```text
std.tensor operation
        ↓
private compiler intrinsic
        ↓
typed high-level intermediate representation (HIR) and MLIR
```

Standard tensor operations include construction, elementwise arithmetic,
shape transformations, reductions, matrix multiplication, convolution, and
device transfer. They do not expose backend types or make tensors generic over
an execution framework.

## Neural-network frameworks

Neural-network frameworks do not belong in `std`. Layers, parameter registries,
optimizers, training loops, checkpoint policy, distributed execution,
mixed-precision policy, datasets, quantization, and model zoos evolve too
quickly for the compiler's compatibility promise.

Bedrock may maintain a reference framework as an independently versioned
official package. Community frameworks compete on design while sharing the
same tensors, autodiff, kernels, placement, serialization building blocks, and
profiling hooks.

## Input, output, and tasks

`std.io` and `std.task` implement the
[explicit input/output contract](io.md). Public APIs depend on `Io`, not a
concrete executor or reactor. The standard contract owns readers, writers,
timers, task scopes, cancellation, and synchronization so packages cannot split
into incompatible async ecosystems.

The first implementations are threaded and deterministic test input/output.
Evented implementations follow only after the same semantic suite passes and
benchmarks justify their complexity.

## Official packages

Broad batteries ship outside the standard library under independent versions:

```text
bedrock.json
bedrock.http
bedrock.crypto
bedrock.image
bedrock.nn
```

Official packages receive first-party maintenance and documentation. They do
not inherit the compiler's release cadence or permanent compatibility burden.
The [package contract](package-management.md) pins them like any other
dependency.

## Compiler integration

The public standard library is ordinary Bedrock source wherever possible.
Private intrinsics exist only when code must communicate semantics that
ordinary source cannot preserve. Intrinsics are not importable by applications.

Unused modules and functions emit no code. Checked HIR and compiled artifacts
are cached by public interface, source, compiler, target, and relevant `known`
arguments.

## Admission rule

An application programming interface (API) enters `std` only when all of these
statements are true:

1. Multiple unrelated domains need the same semantic contract.
2. Competing versions would harm interoperability, safety, or correctness.
3. The behavior is mature and precisely specified on every declared target.
4. Capability, ownership, failure, cancellation, and target behavior are
   explicit.
5. The project will preserve the public contract for years.
6. Tests, fuzzing, benchmarks, and documentation cover the complete contract.

Binary search passes this test. A neural-network optimizer does not.

## Required tests

- Build `core` without allocation, operating-system services, or hosted runtime.
- Reject hidden `Mem`, `Io`, unsafe, host, and device capabilities.
- Prove unused standard modules emit no code.
- Run pure algorithms on every legal host and device target.
- Test search and sort over empty, duplicate, boundary, and adversarial inputs.
- Fuzz collection growth, failure, ownership transfer, and destruction.
- Prove allocation-capable values cannot outlive their `Mem` origin.
- Run reader, writer, task, cancellation, and synchronization suites against
  every input/output implementation.
- Keep private compiler intrinsics inaccessible to package source.
- Rebuild the compiler and standard library through the self-hosting proof.

## References

- [Rust core library](https://doc.rust-lang.org/core/)
- [Rust allocation and collections library](https://doc.rust-lang.org/stable/alloc/)
- [Rust standard prelude](https://doc.rust-lang.org/std/prelude/)
- [Zig explicit allocation](https://ziglang.org/documentation/master/#Memory)
- [Zig I/O as an interface](https://ziglang.org/download/0.16.0/release-notes.html#io-as-an-interface)
- [Go standard library](https://pkg.go.dev/std)
- [Rust binary search and partition points](https://doc.rust-lang.org/stable/core/primitive.slice.html)
- [Go binary search](https://go.dev/src/sort/search.go)
- [C++ standard algorithms](https://eel.is/c++draft/algorithms)
