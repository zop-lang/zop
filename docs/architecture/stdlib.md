# Standard library

Zop ships a small mandatory core and a modular standard library. The core
defines contracts that every target needs. Standard modules add allocation,
input/output, concurrency, tensors, testing, and platform integration through
explicit capabilities.

> **Status:** This page defines target semantics. The Rust bootstrap does not
> implement Zop modules or a Zop standard library yet.

## Boundary

The distribution has three rings:

<!-- markdownlint-disable MD013 -->

| Ring | Versioning | Purpose |
| --- | --- | --- |
| `core` and `std` | Bundled with the compiler | Stable language and runtime contracts |
| Official packages | Independent releases | Maintained batteries that may evolve faster |
| Community packages | Independent releases | Frameworks, applications, and domain libraries |

<!-- markdownlint-enable MD013 -->

The standard library prevents foundational types and protocols from
fragmenting across packages. It does not select one application framework for
the ecosystem.

## Modules

<!-- markdownlint-disable MD013 -->

| Module | Owns | Required capability |
| --- | --- | --- |
| `core` | Primitive contracts, tuples, records, errors, ranges, views, pointers, pure algorithms | None |
| `math` | Pure scalar mathematics and numeric constants | None |
| `core.format` | Deterministic allocation-free formatting contracts | None |
| `core.layout` | Pure CuTe-compatible layout algebra and inspection | None |
| `core.simd` | Provisional portable fixed-width vector and mask values | None |
| `std.mem` | Owned strings, buffers, lists, maps, sets, boxes, arenas, and shared ownership | `Mem` |
| `std.tensor` | Tensor construction, transformations, reductions, and fundamental linear algebra | `Mem` when allocating |
| `std.task` | Structured task scopes, groups, bounded channels, and completion handles | `Io` for scheduling |
| `std.io` | Readers, writers, formatting, files, networking, time, entropy, and processes | `Io` |
| `std.sync` | Threads, locks, events, semaphores, and atomics | `Io` for blocking operations |
| `std.gpu` | Device identity, address spaces, barriers, atomics, and kernel operations | `kn` only |
| `std.sys` | Unsafe pointers, foreign interfaces, volatile access, and application binary interfaces (ABIs) | `unsafe` |
| `core.test` | Allocation-free expectations, comparisons, failures, and caller locations | Test builds |
| `std.test` | Test capabilities, temporary resources, captured I/O, clocks, entropy, and process isolation | Test builds |

<!-- markdownlint-enable MD013 -->

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
- the language-native `Engine` and `Layout` types plus pure layout algebra;
- deterministic primitive, layout, and tensor formatting;
- pure numeric functions;
- target-neutral algorithms;
- compiler-known tensor primitives.

Bare-metal programs and graphics processing unit (GPU) kernels may use the
legal subset of `core` without linking the hosted standard library.

`core.zip` combines iterables lazily. Its compile-time `strict` argument follows
the [language iteration contract](language.md#iteration-and-zip): `false` stops
at the shortest input, while `true` treats unequal exhaustion as a trap. CuTe
layout `zipped_divide` remains a separate `Layout` operation.

Every fixed-width integer type exposes fallible `add`, `subtract`, and
`multiply` members plus explicit `wrapping_*` and `saturating_*` variants. The
ordinary members return the mathematical result or a typed arithmetic failure.
They remain available in bare-metal code and never depend on the optimization
profile. The compiler recognizes their primitive implementations directly and
emits no library call. Wrapping and saturating members are also legal in `kn`.

Integer division exposes these additional members:

```zop
quotient = try to left.divide right, rounding=Floor
exact = try to left.divide_exact right
quotient, remainder = try to left.divide_with_remainder(
    right,
    rounding=Trunc,
)
```

`rounding` is a required compile-time value with `Floor`, `Trunc`, `Ceil`, and
`Euclidean` cases. Rounded division fails with `DivideByZero` or `Overflow`.
Exact division additionally fails with `InexactDivision`. A paired operation
lets the backend compute quotient and remainder together. Fallible members
are legal in `kn` only when their errors are handled locally and converted to
ordinary data, a mask, or an explicit error buffer. They cannot propagate
through the kernel boundary.

The member names are not keywords or prelude functions. `try to left.add right`
propagates `Overflow` without reserving a global `add` identifier. Ordinary
operators remain separate: `left + right` traps when the result cannot be
represented.

Primitive numeric types also expose `cast`, `saturating_cast`,
`wrapping_cast`, and `bitcast`. Plain `cast` is exact unless it names a rounding
mode, and remains fallible for range, precision, or invalid-value errors. The
compiler lowers these members directly; applications do not import an alternate
conversion framework.

Floating types expose pure `is_nan`, `is_infinite`, and `is_finite` predicates.
Their tensor forms are allocation-free elementwise core operations. The
aggregating `require_finite` check belongs to `std.tensor` because reporting the
first logical coordinate may require a scan, device work, and synchronization.

The [numeric contract](numerics.md) owns operators, cast policies,
floating-point profiles, and complete edge behavior. `core` supplies the
corresponding types and operations without introducing an ambient precision
setting.

The provisional `core.simd` module exposes portable fixed-width vectors for
expert algorithms only after the same contract passes on x86-64 and AArch64.
Ordinary code uses compiler-known algorithms and tensor operations, which keep
vector width and target features out of source. See the
[SIMD contract](simd.md#expert-vectors).

## Mathematics

`math` is a standalone public module bundled with the compiler. Its
implementation belongs to the mandatory core layer, but this module
intentionally omits that layer from its public path. There is no public
`core.math` alias and `math` is not a separately versioned dependency.

```zop
from math import min, max

lower = min left, right
upper = max left, right, limit
```

Qualification remains available when it improves local clarity:

```zop
import math

bounded = math.clamp value, lower, upper
root = math.sqrt value
```

`math` owns pure scalar mathematics:

- `min`, `max`, and `clamp`;
- absolute value and sign functions;
- same-type floating rounding functions;
- roots, exponentials, logarithms, and trigonometry;
- typed mathematical constants; and
- floating-point helpers that copy a sign or step to an adjacent value.

It does not own numeric types, operators, casts, or their failure rules. Those
are language and primitive-type contracts. Tensor reductions and linear
algebra remain tensor APIs. Statistics, random-number generation,
neural-network operations, search, sorting, and collection algorithms remain
outside `math`.

Every `math` operation is allocation-free and target-neutral. The legal subset
may run in bare-metal code and compatible `kn` kernels. A target that cannot
preserve the selected type and floating-point profile rejects the operation
instead of linking a weaker implementation. Scalar `min` and `max` propagate
NaN and preserve signed-zero ordering under the complete
[numeric contract](numerics.md#minimum-and-maximum).

## Prelude

The prelude is the small set of names available without an import. It contains
only primitive types and operations used by nearly every program. Adding a name
to the prelude is a language-compatibility decision.

`Engine` and `Layout` are predeclared because every tensor exposes those
CuTe-native language-owned types. The broader algebra remains under
`core.layout` and does not fill the prelude with specialist operations.

Owned collections, input/output functions, concurrency types, platform APIs,
and framework names are never imported implicitly. `print` is a standard
input/output function and requires `Io`; it is not a compiler statement or
ambient global operation.

Core defines deterministic formatting for primitive values, `Layout`, and
tensors. `std.io.print` supplies the writer and failure channel. This split lets
bare-metal and GPU compilation understand representation without granting
permission to emit bytes.

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
- first-match search plus `any`, `all`, `count`, and `map`;
- partition points and ranges;
- sort, stable sort, and selection;
- partition, merge, and heap operations;
- copy, move, fill, swap, and comparison;
- scans and reductions.

Allocation-free pure algorithms require neither `Mem` nor `Io`. Algorithms
that need temporary workspace accept a caller-provided view or `Mem`. A
target-neutral implementation may compile for both central processing unit
(CPU) `fn` and GPU `kn` code.

The compiler recognizes the allocation-free search, predicate aggregation,
map, reduction, and scan contracts before lowering them to loops. Their
vectorization behavior and proof requirements follow
[SIMD and vectorization](simd.md); `core.algorithm` never exposes a target lane
width.

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

Reductions accept an explicit order. `Source` preserves logical coordinate
order; `Tree` permits regrouping for vector, parallel CPU, or device execution.
`sum` is the standard addition reduction, while `reduce` accepts an identity
and first-class combining function. Zop reserves no reduction or accumulation
syntax beyond these calls and ordinary sequential `+=`.
`min` and `max` use the same order parameter and accept `initial` when an empty
reduction domain must have a total result.

Allocation-free views are part of `core`. Integer and basic slice indexing,
`at`, `slice`, `unsqueeze axis=`, `squeeze axis=`, and `expand shape` transform
shape and `Layout` without copying storage. Their bounds, failure, origin, and
rank rules follow the [indexing contract](indexing.md); broadcasting follows
the [trailing-axis contract](layouts.md#broadcasting-and-expansion).
Bracket slicing clips endpoints in the compiler. Core `slice` accepts trailing
`strict: known bool = false`; strict dynamic endpoints return `BoundsError`.

`std.tensor` owns data-dependent gather, mask selection, scatter, `repeat`, and
explicit materialization because they may create owned storage, derive shape
from loaded data, or require conflict policy. Allocation requires `Mem`.
It also owns `require_finite`, which checks without changing the tensor and
establishes the flow-sensitive finite fact defined by the
[numeric contract](numerics.md#nonfinite-values-and-validation).
Framework conveniences such as `tile`, `expand_as`, `broadcast_to`, and
`broadcast_tensors` remain ordinary wrappers; they cannot redefine language
operator semantics.

Every tensor exposes the [language-native Engine and Layout](layouts.md) that
map logical coordinates to values. `core.layout` contains pure composition, coalescing,
tiling, inverse, and analysis operations because CPU code, `kn` kernels, views,
foreign interfaces, and tooling must share one algebra. Allocation-backed
tensor construction and explicit `relayout` remain in `std.tensor` and require
`Mem`.

The [tensor formatter](layouts.md#tensor-formatting) is also stable. It streams
logical values with deterministic metadata and summarization. Visualization,
bank coloring, and thread/value diagrams remain toolchain operations rather
than standard-library rendering dependencies.

## Neural-network frameworks

Neural-network frameworks do not belong in `std`. Layers, parameter registries,
optimizers, training loops, checkpoint policy, distributed execution,
mixed-precision policy, datasets, quantization, and model zoos evolve too
quickly for the compiler's compatibility promise.

Zop may maintain a reference framework as an independently versioned
official package. Community frameworks compete on design while sharing the
same tensors, layouts, kernels, placement, serialization building blocks, and
profiling hooks.

## Program transformations

Automatic differentiation, batching, quantization, sparsity, and similar model
transformations live in independently versioned packages rather than `core`,
`std`, or source grammar. No package may depend on a private compiler intrinsic
or unstable high-level intermediate representation (HIR).

A future compiler-extension contract must satisfy the standard-library
admission rule for multiple unrelated transformations. It must also preserve
hermetic builds, deterministic cache identity, typed verification, and explicit
failure. Until then, optional transformations use ordinary Zop interfaces and
cannot rewrite compiler IR.

## Input, output, and tasks

`std.io` and `std.task` implement the
[explicit input/output contract](io.md). Public APIs depend on `Io`, not a
concrete executor or reactor. The standard contract owns readers, writers,
timers, task scopes, cancellation, and synchronization so packages cannot split
into incompatible async ecosystems.

The first implementations are threaded and deterministic test input/output.
Evented implementations follow only after the same semantic suite passes and
benchmarks justify their complexity.

## Testing

The language and compiler discover test declarations. `core.test` supplies only
the allocation-free assertion contract needed on every target. `std.test`
supplies hosted resources through an explicit `Test` capability. The default
runner belongs to the toolchain rather than the standard library so another
runner can implement the same versioned manifest and event protocols.

Property testing, snapshots, rich matchers, browser fixtures, and service
fixtures are official or community packages. Coverage-guided fuzzing and
benchmark measurement are toolchain modes because they require compiler
instrumentation and execution control. The [testing contract](testing.md)
defines discovery, isolation, runner interoperability, and rationale.

## Official packages

Broad batteries ship outside the standard library under independent versions:

```text
zop.json
zop.http
zop.crypto
zop.image
zop.nn
zop.web
```

Official packages receive first-party maintenance and documentation. They do
not inherit the compiler's release cadence or permanent compatibility burden.
The [package contract](package-management.md) pins them like any other
dependency.

`zop.web` contains generated browser bindings and the support used by
emitted ECMAScript, WebAssembly islands, and WebGPU kernels. Unused support
emits no code. Frontend frameworks remain independently versioned packages.
The [web target contract](web.md) defines that boundary.

## Compiler integration

The public standard library is ordinary Zop source wherever possible.
Private intrinsics exist only when code must communicate semantics that
ordinary source cannot preserve. Intrinsics are not importable by applications.

Unused modules and functions emit no code. Verified HIR and compiled artifacts
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
- Compare semantic search, aggregation, map, scan, and reduction operations
  across scalar and every certified SIMD schedule.
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
