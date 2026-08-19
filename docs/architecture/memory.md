# Memory management

Zop uses single-owner values, checked borrowing, and deterministic
destruction. The core language does not require garbage collection.

> **Status:** This page defines target semantics. The bootstrap does not yet
> implement ownership, views, raw pointers, or unsafe checking.

## Scope

The language defines ownership, borrowing, transfer, copying, and destruction.
The runtime implements `Mem` capabilities and resource release. The backend may
reuse storage only when the language contract proves that reuse unobservable.

## The `Mem` capability

`Mem` is the memory-policy type. `mem` is the value passed to code that may
request storage. `alloc` is an operation on that value, not the name of the
capability.

```zop
fn zeros mem: Mem, count: int -> f32[count]
    values = mem.alloc f32, count
    values.fill 0
    values
```

Every owned allocation records the `Mem` origin responsible for releasing or
reusing its storage. The allocation cannot outlive that origin. The backend
does not materialize the origin at runtime when static analysis already proves
it.

`mem.alloc` returns an explicit failure unless that `Mem` implementation has an
infallible policy.

Member selection and calls on `mem` follow the
[callables contract](callables.md). Storing `mem.alloc` or `mem.free` creates a
bound callable whose receiver borrow cannot outlive `mem`.

Different `Mem` implementations may provide general heap storage, arenas,
pools, mapped storage, device storage, accounting, or leak checking. A function
accepts `mem: Mem` when it needs memory policy and omits it when it does not.

Zop has no ambient global `Mem`.

## Ownership rules

Every resource-owning value has one owner. `give` transfers that ownership and
invalidates the old binding. A value is destroyed when its owner leaves scope
unless it was given or explicitly dropped.

Zop permits either one mutable borrow or any number of immutable borrows.
Every borrow must end before its owner is destroyed or given.

Small scalar types may opt into implicit copying. Heap-owning and
resource-owning values never deep-copy implicitly. Copying them requires an
explicit operation.

Raw pointers exist for systems work and foreign interfaces. They do not own or
release their pointees. Unsafe source may assert obligations that the compiler
cannot prove, but it does not disable type, ownership, or effect checking and
does not weaken the contract expected by safe callers.

## Local mutation

A uniquely owned local value may be reassigned or updated without declaring the
binding `mut`. Ownership proves that no external alias can observe the update.
The binding's type remains fixed.

High-level intermediate representation (HIR) gives each local assignment a new
value identity. This lets later passes recover static single assignment form
without adding syntax to source.

`mut` appears only when writable access crosses an alias boundary. A `mut`
parameter or view grants one exclusive borrow for its valid lifetime. The
compiler rejects every conflicting borrow.

## Function arguments

Ordinary parameters borrow immutably. `mut` grants an exclusive mutable borrow.
`give` transfers ownership to the callee.

| Parameter form | Contract |
| --- | --- |
| `value: T` | Immutable borrow for the call |
| `mut value: T` | Exclusive borrow for the call |
| `give value: T` | Ownership transfer |

The declaration describes the call protocol:

```zop
fn enqueue give value: Tensor
```

A caller writes `give` when transferring a named value:

```zop
enqueue give tensor
```

The source binding is invalid after `give`. A fresh temporary transfers
automatically because no source binding remains available:

```zop
enqueue make_tensor()
```

Returning a local value also transfers it automatically because the local scope
ends. Implicitly copyable values use ordinary argument syntax and remain valid.

General references are not safe values that source can construct, store, or
return. Borrowing normally appears as a parameter mode. Language-defined forms
such as tensor views, slice views, bound methods, and closures may carry a
compiler-tracked origin.

## Raw pointers

`*` is a postfix type constructor. It attaches to the complete type, never to a
variable name:

```zop
value: i32*
indirect: i32**
deep: i32********
source: const f32*
destination: f32*
optional: f32*?
pointers: f32*[n]
buffer: f32[n]*
device: global f32*
callback: (fn value: i32 -> i32)*
```

Each declaration binds one name to one complete type. C-style declarations
such as `int *left, right` are invalid; write `left: int*, right: int*`.

Each `*` expresses exactly one pointer constructor. Zop has no counted pointer
form: `T*8` is invalid because it resembles multiplication, stride, or an
eight-element operation. Eight levels are written as `T********`.

Deep indirection should normally receive a name at its declaration boundary:

```zop
type DeviceHandle = Device********
```

The formatter preserves repeated stars and never invents a second spelling.

Type constructors compose from left to right. `f32*[n]` is a tensor of
pointers, while `f32[n]*` is a pointer to the tensor ABI value selected by its
static `Layout` profile and dynamic leaves, not one universal sizes-and-strides
descriptor. `const T*` cannot mutate `T`; `T*` may. `T*` is non-null and `T*?`
is nullable. A foreign C
pointer is nullable unless its imported contract proves otherwise.

`?` applies to the complete pointer type immediately before it:

```zop
outer_may_be_null: T**?
inner_may_be_null: (T*?)*
both_may_be_null: (T*?)*?
```

Parentheses are required when another pointer constructor follows a nullable
pointer. The compiler rejects dereference, indexing, arithmetic, or conversion
of `T*?` to `T*` until control flow proves it non-null. Address-of produces a
non-null pointer. Foreign declarations default to nullable pointers unless
their imported contract proves non-null.

Non-null is one safety fact, not a validity claim. Dereference still requires
an unsafe block plus valid provenance, lifetime, alignment, initialization, and
access permission.

`&value` takes an address and prefix `*pointer` dereferences one level. The
compiler applies repeated prefix stars one level at a time:

```zop
pointer = &value
unsafe
    value = *pointer
    nested = **indirect
```

Taking an address is safe when the resulting pointer cannot outlive its source.
Dereferencing, pointer arithmetic or indexing, pointer-integer conversion,
reinterpretation, raw view construction, foreign deallocation, and calling an
unsafe function require a lexical `unsafe` block.

`unsafe` must be followed immediately by a non-empty indented block. A missing
indent is a parse error; it never produces a smaller or implicit unsafe region.
Every unsafe operation outside the block is rejected independently. The
formatter preserves the block boundary, and an unsafe block containing no
unsafe operation is an unnecessary-unsafe diagnostic.

A pointer retains its allocation identity, address space, alignment, and access
permission. Pointer-to-integer conversion explicitly exposes the address;
integer-to-pointer conversion does not silently recover provenance. Volatile
and atomic access are distinct operations, never properties inferred from an
ordinary dereference.

Constructing a safe view from a pointer and extent must prove alignment,
initialization, bounds, lifetime, placement, and exclusive access when mutable.
Failure to prove any item leaves the result inside unsafe code; the compiler
does not bless a view from a plausible address.

`unsafe fn` states an obligation that its caller must prove. Its implementation
still places each unchecked operation in a lexical `unsafe` block so review can
see the exact trusted region. The compiler reports which obligation cannot be
proven; `unsafe` is never a blanket optimization switch.

Every unsafe function documents its safety preconditions. Violating one is
undefined behavior, and the optimizer may rely on it. Writing `unsafe` accepts
that obligation; it does not make an invalid operation defined.

Owned storage still releases deterministically. A raw pointer is never an
owner, so dropping it does nothing. Safe foreign ownership uses an owning
wrapper with one destructor. `mem.free_raw pointer` is the explicit unsafe
escape hatch when a foreign application binary interface transfers a raw
allocation to Zop.

`mem.free_raw` requires the original allocation address, matching `Mem`, layout,
and alignment, no prior release, and no live safe view. The operation fails to
compile when those facts are statically contradictory; remaining obligations
belong to the unsafe block.

## Tensors and views

A tensor pairs a CuTe-native `Engine` with a language-native `Layout`. An owning
Engine owns its array; a viewing Engine borrows an iterator into another owner.
Giving a tensor transfers its Engine and Layout rather than copying elements. A
tensor view borrows and may advance the source Engine, carries a transformed
Layout, and cannot outlive the source tensor. The
[tensor-layout contract](layouts.md) defines the mapping;
the [indexing contract](indexing.md#ownership-and-mutation) defines slice
origins, disjoint mutation, and checked general views.

```zop
fn row values: f32[m, n], index: int -> view f32[n]
    values[index]
```

The compiler infers that the returned view originates from `values`. High-level
intermediate representation (HIR) stores that ownership origin, advanced
Engine, and residual Layout even when source omits them. An exported or
ambiguous return states its
possible origins with `from`:

```zop
fn choose left: f32[n], right: f32[n], use_left: bool
    -> view f32[n] from left, right
```

The annotation bounds the valid origins; it does not choose one at runtime. An
unlisted or unprovable origin is a compile error.

A record or closure may store a view only while its origin remains alive. The
same rule applies to a bound method or closure that borrows a captured value.
Safe source does not expose general lifetime parameters merely to express this
constraint.

Pure tensor expressions have value semantics:

```zop
c = a @ b
```

`c` is logically a new tensor. The compiler may donate or reuse an input buffer
only when that input is uniquely owned and dead after the operation. Mutation
requires a mutable borrow. Zop does not perform implicit deep copies or
copy-on-write.

Static Shape facts remain in types. Static Engine kind, Layout structure, and
leaves remain compiler metadata. Runtime values contain only dynamic Engine
state, distinct dynamic Layout leaves, placement, and ownership state. A fully
static Layout adds no runtime field. Zop does not impose a full
sizes-and-strides array on every tensor.

## GPU lifetimes

A graphics processing unit (GPU) kernel launch borrows every input buffer until
the completion event. The launch owns its output buffers. Dropping a device
tensor queues deallocation after its final event; it does not force global
synchronization.

Successful completion publishes the owned outputs and ends the input borrows.
A device trap publishes no output and invalidates the complete execution
context. Every tensor allocated in that context loses storage validity, not
only values named by the failed kernel. A stale handle may be discarded and its
saved type or layout metadata may be reported for diagnostics, but it cannot
read, write, transfer, or launch against device storage.

Invalidation is not deallocation. The runtime destroys the failed context and
its resources according to the target contract; a tensor destructor never
attempts an ordinary device free through an already invalid context. Recovery
creates a new context with new storage identities. No borrow or owned tensor
crosses from the failed context into the replacement.

Host-to-device and device-to-host transfers are explicit. The type checker
rejects a host pointer or host tensor passed directly to a kernel.

See the [`fn` and `kn` contract](gpu.md) for placement and launch semantics.

## Shared and cyclic data

Shared ownership is an explicit library type, not the default representation.
A matching weak reference breaks ownership cycles. Thread-safe sharing may use
atomic reference counts when the type requests that cost.

Arenas own groups of values that die together. They are the preferred model for
syntax trees, compiler graphs, and request-scoped object graphs.

A future tracing collector may exist inside an explicit `gc` region. Managed
objects cannot be the sole owners of files, sockets, locks, device buffers, or
other resources that require prompt release. The collector is never a fallback
for code that fails ownership checking.

Readers, writers, and asynchronous tasks may carry explicit completion
obligations in addition to ordinary destruction. See the
[explicit input/output contract](io.md).

Destructors do not perform fallible or blocking input/output. Files, sockets,
writers, and tasks must resolve those operations explicitly before destruction.

## Compiler contract

The frontend checks ownership once on typed HIR, before generic specialization.
HIR records owners, borrow origins, mutability, address spaces, and destruction
points.

The result is cached per function. Machine-code generation does not repeat the
ownership analysis for every specialization.

Mojo 1.0's source-level `lit` dialect validates this phase boundary: reference
types carry origins, then one lifetime pass checks uses, tracks interior
origins, and inserts implicit destruction before those references lower to
ordinary pointers. Zop keeps origins as typed-HIR facts instead of exposing
general lifetime parameters, but applies the same rule before specialization
or MLIR emission.

Multi-Level Intermediate Representation (MLIR) receives value tensors plus the
proven alias and ownership facts. One-Shot Bufferize may reuse storage when
those facts permit it. Ownership-based buffer deallocation assigns and lowers
the responsibility to free each resulting buffer.

The backend emits alias guarantees only when HIR proves them. An optimistic
alias annotation is a miscompilation, not an optimization.

## Compilation cost

Ownership checking is an additional frontend dataflow analysis. It must be
measured, cached, and kept independent from expensive code generation.

Development builds use incremental per-function checking, minimal MLIR passes,
and Cranelift. Production builds reuse checked HIR and may spend more time on
whole-program summaries and tensor optimization.

Rust demonstrates that ownership checking and backend cost are separate.
Borrow checking runs on mid-level IR before monomorphization, while generic
specialization and backend optimization introduce their own compile-time cost.

## Required tests

- Reject use after ownership transfer with `give`.
- Require `give` when a named value enters an owning parameter.
- Permit a fresh temporary to enter an owning parameter directly.
- Reject mutable access while any conflicting borrow is live.
- Permit local mutation of a uniquely owned value without `mut`.
- Require `mut` for writable access through a borrowed parameter or view.
- Reject a view that outlives its tensor.
- Invalidate every descendant origin proof when its owning ancestor mutates,
  while preserving a proven-disjoint sibling origin.
- Insert each implicit destruction point before specialization and preserve it
  in every concrete instance.
- Require `from` when a public view may originate from more than one input.
- Prove a non-null pointer before converting it to `T*`.
- Reject dereference, indexing, and arithmetic on `T*?` before a non-null proof.
- Distinguish `T**?`, `(T*?)*`, and `(T*?)*?`.
- Require lexical `unsafe` for raw dereference, arithmetic, conversion,
  reinterpretation, raw views, foreign release, and unsafe calls.
- Reject counted pointer syntax such as `T*8`.
- Parse `T********` as exactly eight pointer constructors.
- Reject `unsafe` without a non-empty indented block.
- Diagnose an unsafe block that contains no unsafe operation.
- Distinguish a tensor of pointers from a pointer to a tensor descriptor.
- Prove dropping a raw pointer never releases storage.
- Prove every safe foreign owner releases exactly once.
- Prove that ordinary calls do not consume borrowed arguments.
- Prove that copying a tensor is explicit.
- Prove that pure tensor code may reuse a uniquely owned dead buffer.
- Keep device buffers alive until every borrowing kernel completes.
- Publish device outputs only after successful completion and reject every
  storage operation through a context invalidated by `DeviceFault`.
- Queue device deallocation without an implicit global synchronization.
- Reject ownership and lifetime violations before MLIR emission.

## References

- [Rust ownership](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)
  and [borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
- [Mojo ownership and argument conventions](https://docs.modular.com/mojo/manual/values/ownership)
- [Mojo origin-carrying reference types](https://github.com/modular/modular/blob/f66d4d522c34be0a961ffac3dbfc81e30f67942e/KGEN/include/KGEN/LITDialect/LITTypes.td)
- [Mojo lifetime checking](https://github.com/modular/modular/blob/f66d4d522c34be0a961ffac3dbfc81e30f67942e/KGEN/lib/LowerLIT/CheckLifetimes.cpp)
- [MLIR bufferization](https://mlir.llvm.org/docs/Bufferization/)
- [MLIR ownership-based buffer deallocation](https://mlir.llvm.org/docs/OwnershipBasedBufferDeallocation/)
- [Rust MIR](https://rustc-dev-guide.rust-lang.org/mir/index.html)
- [Rust monomorphization](https://rustc-dev-guide.rust-lang.org/backend/monomorph.html)
