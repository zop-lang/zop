# Indexing, slicing, and tensor views

Zop uses familiar Python indexing syntax, PyTorch's zero-copy basic-view
semantics, and the language-native [`Layout`](layouts.md) algebra. Indexing is
not a second addressing system layered on tensors. It fixes coordinates or
restricts coordinate ranges, then derives a residual layout over the same
storage.

> **Status:** This page defines target source and compiler contracts. The Rust
> bootstrap does not yet implement tensors, indexing, slicing, or views.
> Bracket syntax, negative integer indexing, Python-style endpoint clipping, and
> explicit strict slicing are settled target semantics.

## Design goals

The source surface should satisfy all of these constraints at once:

- Ordinary indexing should look like pseudocode familiar to Python users.
- Element access, slicing, transpose, tiling, and broadcasting should share one
  coordinate-to-storage model.
- Basic slicing should construct a view in constant time without allocation or
  element movement.
- Negative indexing should be convenient without weakening bounds safety.
- Static shape and layout facts should occupy no runtime storage.
- Dynamic metadata should be proportional to rank and remaining dynamic layout
  leaves, never to the number of elements.
- Bounds, ownership, aliasing, and placement should remain explicit compiler
  facts through high-level intermediate representation (HIR).
- A backend should never repair an unsupported layout by inserting a hidden
  copy.
- The same source meaning should hold on the interpreter, central processing
  unit (CPU), graphics processing unit (GPU), JavaScript, and WebAssembly
  targets.

The design deliberately does not make every convenient tensor-library
operation into grammar. Data-dependent gather, boolean masks, index tensors,
and materialization belong in `std.tensor` or a framework because they can
allocate, change result shape from data, or require scatter conflict policy.

## One model

A tensor is an Engine, an element type, and a verified Layout:

```text
Tensor = Engine composed with Layout
Layout : logical coordinate -> Engine index
```

An integer index fixes one coordinate. A slice restricts one coordinate to an
arithmetic progression. Applying either operation to a layout produces:

```text
View {
    engine:        source Engine advanced by the selected offset,
    layout:        residual coordinate mapping,
    borrow_origin: Zop owner and lifetime,
}
```

The advanced Engine already points at the view's logical coordinate zero. Zop
does not retain the old Engine plus a duplicate origin offset. This follows
CUTLASS CuTe's `Tensor<Engine, Layout>` model and PyCuTe's equivalent
`Tensor(Accessor, Layout)` reference model, with ownership and bounds proof added
by the language.

## Canonical tensor queries

Zop distinguishes the logical domain from its physical mapping:

```zop
matrix = [[1, 2, 3], [4, 5, 6]]

matrix.rank                 # 2
matrix.shape                # (2, 3)
matrix.extent axis=0        # 2
matrix.extent axis=1        # 3
matrix.numel()              # 6
matrix.layout.stride        # (3, 1)
matrix.layout.cosize        # 6 for this compact layout
```

The terms have non-overlapping meanings:

- `rank` is the number of logical axes.
- `shape` is the ordered tuple of logical extents.
- `extent axis=i` is the number of positions on one logical axis.
- `numel()` is the product of all logical extents.
- `layout.cosize` describes the size of a finite nonnegative scalar codomain,
  the set of offsets a layout can produce, when that quantity is defined. It is
  not the tensor's logical element count.

There is no tensor `.length`, `.len()`, `.size`, or `.count()` alias. Sequence
length usually means the leading extent, layout size can mean a storage span,
and collection `count` commonly means counting values that match a predicate.
`numel`, short for number of elements, has one established tensor meaning and
names the requested quantity directly. `Layout.size` remains a separate
algebraic query over its domain. A framework may expose compatibility aliases
without changing the core meanings.

These queries do not create redundant descriptor fields. `shape` is the
canonical logical domain. `extent` selects one of its leaves, and `numel()` is
its checked product. The optimizer folds static queries and common-subexpression
eliminates repeated dynamic queries. Storing a second `length` or
`element_count` field would create two representations of the same fact and an
invariant the compiler would have to maintain forever.

`numel()` uses the ordinary trapping integer contract. A statically
unrepresentable product is a diagnostic; a dynamic unrepresentable product
traps. Tensor construction already proves addressable owned storage, but a
zero-stride or otherwise virtual view can have a logical count larger than its
backing span, so the query must still define overflow independently.

The edge cases follow the mathematics of Cartesian products:

```zop
empty: f32[4, 0, 8]
scalar_tensor: f32[]

empty.rank                    # 3
empty.extent axis=1           # 0
empty.numel()                 # 0
scalar_tensor.rank            # 0
scalar_tensor.shape           # ()
scalar_tensor.numel()         # 1
```

A rank-zero tensor has one logical coordinate, the empty coordinate. Complete
integer indexing of a positive-rank tensor returns its element value rather
than manufacturing a rank-zero view. The source spelling for extracting a
value already held in an explicit rank-zero tensor remains outside the first
tensor slice.

Logical count and physical span can differ substantially:

```zop
base = [10, 20, 30]
many = base.expand (4, 3)

many.shape                  # (4, 3)
many.numel()                # 12 logical elements
many.layout.stride          # (0, 1)
many.layout.cosize          # 3 stored elements
```

That distinction is why `numel()` is a tensor query and `cosize` is a layout
query. Signed or structured codomains may not have a scalar `cosize`; bounds
verification still computes their complete reachable storage set or a proven
enclosing range.

## Element indexing

An integer index selects one logical coordinate and removes that axis from the
result. Indices are zero-based:

```zop
matrix = [[1, 2, 3], [4, 5, 6]]

matrix[0, 1]        # 2
matrix[1]           # view i64[3] containing [4, 5, 6]
matrix[1, 2]        # 6
```

Supplying fewer components than the rank implicitly leaves every trailing axis
unrestricted. `matrix[1]` therefore means `matrix[1, :]`. Supplying more
components than the rank is always invalid. This rule makes the common row and
subtensor cases terse without asking the parser or type checker to infer intent.

### Negative integer indices

A negative integer counts from the end of its axis:

```zop
values = [10, 20, 30, 40]

values[-1]          # 40
values[-4]          # 10
```

For an extent `n`, the compiler normalizes an integer `i` exactly once:

```text
normalized(i, n) = i       when i >= 0
normalized(i, n) = n + i   when i < 0
valid(i, n)      = 0 <= normalized(i, n) < n
```

Normalization is not modulo arithmetic. `values[-5]` does not wrap around a
four-element tensor. It fails the same bound as `values[4]`. An empty axis has
no valid integer index. `-0` is ordinary integer zero.

The implementation must compare and normalize without overflowing for the
minimum representable signed integer. An optimization may remove normalization
or a bounds check only after proving the same validity predicate.

Negative indexing is core syntax because it is local, deterministic, and
allocation-free. It compiles to one comparison and conditional addition when
the sign is not statically known, then shares the ordinary bounds and address
calculation. No negative value reaches a raw storage address.

### Bounds failure

A statically invalid index is a compile-time diagnostic. A dynamic invalid
index traps before reading or writing storage:

```zop
value = tensor[index]              # concise, trapping access
value = try to tensor.at index     # recoverable BoundsError
```

Bracket access is appropriate when an invalid coordinate violates the local
algorithm's invariant. `at` is appropriate when the coordinate came from an
untrusted file, network request, user input, or another recoverable boundary.
The two forms perform the same normalization and accept the same valid
coordinates. `at` is not a slower or more permissive indexing model; it changes
only the failure channel.

Zop does not make every access return an optional value. Mandatory optional
handling would obscure proven loops and kernels, while unchecked access would
make dynamic input unsafe. A trapping operator plus a fallible member keeps the
common proof-friendly path terse and the recoverable path explicit.

## Basic slicing

Basic slicing uses Python's familiar half-open `start:stop:step` notation:

```zop
values = [0, 1, 2, 3, 4, 5]

values[1:4]             # [1, 2, 3]
values[:3]              # [0, 1, 2]
values[3:]              # [3, 4, 5]
values[::2]             # [0, 2, 4]
values[::-1]            # [5, 4, 3, 2, 1, 0]
```

The start is inclusive and the stop is exclusive. The step is a nonzero signed
integer. A positive step walks toward larger coordinates; a negative step walks
toward smaller coordinates. A slice retains its axis even when the resulting
extent is zero or one.

Each axis has one selector. Commas separate axes:

```zop
matrix = [[0, 1, 2], [3, 4, 5], [6, 7, 8]]

matrix[1, :]            # rank-one row view: [3, 4, 5]
matrix[:, 1]            # rank-one column view: [1, 4, 7]
matrix[::2, 1:]         # rank-two view: [[1, 2], [7, 8]]
matrix[1, 1:]           # rank-one view: [4, 5]
```

An integer selector removes its axis. A slice selector retains its axis.
Unspecified trailing axes behave as `:`. These structural rules determine the
result rank during type checking; they do not depend on runtime values.

Computed code may use the equivalent named member form:

```zop
evens = values.slice axis=0, start=0, stop=values.extent(axis=0), step=2
middle = matrix.slice axis=1, start=1, stop=3
```

Named arguments are a language-wide call feature, not tensor-specific syntax.
They make algorithms over several axes readable and avoid CuTe-style APIs in
which a run of positional integers is easy to transpose accidentally. Bracket
syntax remains preferable for literal, local selections.

### Defaults and negative steps

Omitted endpoints depend on the step direction:

- A positive step defaults to the first coordinate and the boundary after the
  final coordinate.
- A negative step defaults to the final coordinate and the boundary before the
  first coordinate.
- An omitted endpoint is a boundary sentinel, not an integer literal.

The final distinction matters. In Python-compatible notation, `values[::-1]`
reverses the entire axis, while `values[:-1:-1]` supplies an explicit `-1`
endpoint that refers to the final element after negative-index normalization.
The compiler preserves omission through parsing and normalizes only after it
knows the extent and step direction.

For a positive step, explicit endpoints first add the extent when negative and
then clip to the closed boundary range `0` through `n`. For a negative step,
they first add the extent when negative and then clip to the boundary range
`-1` through `n - 1`. The omitted negative-step stop remains the distinct
before-begin sentinel represented by `-1` only after normalization. Therefore
`values[::-1]` reverses the axis while `values[:-1:-1]` is empty.

For a normalized positive step `s`, start `a`, and stop `b`, the result extent
is:

```text
0                              when a >= b
1 + floor((b - 1 - a) / s)    otherwise
```

For a normalized negative step `-s`, start `a`, and stop boundary `b`, it is:

```text
0                              when a <= b
1 + floor((a - 1 - b) / s)    otherwise
```

These formulas operate on mathematical integers. Compile-time evaluation and
runtime lowering must avoid intermediate overflow.

### Clipping and strict endpoints

Bracket slices clip explicit endpoints to the nearest valid boundary after
negative-index normalization. The compiler owns this source semantics and
derives the residual `Layout` directly; clipping is not a standard-library
helper and does not allocate or inspect element data.

Chunking therefore needs no manual `min`:

```zop
for start in range(0, values.numel(), step=tile)
    chunk = values[start:start + tile]
```

An endpoint entirely beyond the same side produces an empty view. Clipping
applies only to slice boundaries. An out-of-range integer element index still
traps or returns `BoundsError` through `at`.

Code that requires exact in-range endpoints uses the named strict operation:

```zop
chunk = values[start:stop]
exact = try to values.slice(
    axis=0,
    start=start,
    stop=stop,
    strict=true,
)
```

`strict` is a trailing `known bool` whose default is `false`. When true, a
statically invalid endpoint is a diagnostic and a dynamic invalid endpoint
returns `BoundsError` before constructing the view. Strictness asserts algorithm
intent; it does not select a stronger memory-safety mode. Both policies prove
every live coordinate and storage access in bounds.

Python, NumPy, and PyTorch establish clipping as the least surprising bracket
behavior for tensor users. Zop adds the explicit strict path for kernels,
protocol fields, fixed tiles, and other algorithms where a shortened view would
hide a bug. This composition keeps whiteboard-friendly chunking while making an
exact-boundary contract reviewable in source.

## A slice is a layout transformation

Consider one source axis with extent `n`, Layout stride `d`, normalized start
`a`, and step `s`. Slicing derives:

```text
view.engine       = source.engine.advance(a * d)
view.axis.extent  = selected coordinate count
view.axis.stride  = d * s
```

Other axes retain their Shape and Stride. An integer selection advances the
Engine and removes the selected axis from the residual Layout.

For example:

```zop
matrix = [[0, 1, 2], [3, 4, 5]]
column = matrix[:, 1]

column.shape            # (2,)
column.layout.stride    # (3,)
column.numel()          # 2
```

The column is not contiguous, but it is a complete ordinary view. Indexing it
performs the same integer layout evaluation as any dense tensor.

A reverse view advances its Engine and uses a negative Stride:

```zop
values = [10, 20, 30, 40]
reverse = values[::-1]

reverse.shape           # (4,)
reverse.layout.stride   # (-1,)
reverse[0]              # 40
reverse[3]              # 10
```

For this view, the Engine advances to the source's final element before the
residual Layout applies offsets `0, -1, -2, -3`. Bounds verification must
therefore reason about the complete reachable codomain and advanced Engine. It
must not assume that every Stride is nonnegative.

CuTe `coshape` and `cosize` are most useful for canonical nonnegative
codomains. A reverse view does not redefine those algebraic operations to mean
"absolute stride." The compiler separately carries the source allocation and
proves the minimum and maximum reachable signed offsets from the advanced
Engine. This preserves CuTe terminology without using a nonnegative bounding
shortcut where it is mathematically false.

An empty slice may carry a one-past Engine iterator because it has no live
coordinate and performs no dereference. The verifier proves emptiness before
allowing that Engine; no optimizer may dereference it.

This representation is powerful enough for dense slices, transposes, stepped
views, reversals, padding, broadcasts, tiles, and hierarchical CuTe layouts.
Zop does not need a special slice object whose semantics diverge from `Layout`.

## Basic and advanced indexing are intentionally different

Basic selectors are integers and slices. They transform only metadata and
therefore always return an element or zero-copy view.

Advanced selectors include:

- a tensor of integer coordinates;
- a boolean mask;
- a data-dependent list of coordinates; and
- a gather or scatter mapping that loads indices from storage.

Those operations can require allocation, produce data-dependent extents, or
map several logical coordinates to one destination. They are ordinary
`std.tensor` operations such as `gather`, `scatter`, and mask selection, not
alternate meanings of brackets in the first tensor contract.

This boundary follows PyTorch's useful distinction while making it more
visible. PyTorch basic indexing returns views and advanced indexing returns a
copy. Zop keeps basic indexing in grammar and puts potentially allocating
advanced operations behind named functions that expose `Mem`, output shape,
and scatter conflict policy. Source must never change from view to copy because
the runtime type of an index happened to differ.

## General layout views

Slicing is only one way to derive a view. A compiler-verified general view may
pair an existing or advanced Engine with another `Layout`. Its exact
surface spelling remains provisional, but its safety contract is fixed:

1. Every live logical coordinate maps within the source storage allocation.
2. The view records and cannot outlive the Zop owner from which its Engine is
   borrowed.
3. Read-only views may use non-injective layouts.
4. Mutable views require an injective mapping and exclusive access for the
   entire borrow.
5. Placement and element alignment remain compatible.
6. Failure to prove the contract is a compile-time diagnostic when all facts
   are static and a typed failure when a supported dynamic constructor is used.

An `unsafe` block may assert missing raw-pointer provenance or foreign storage
facts, but it does not erase the resulting safe view's lifetime, shape, layout,
or bounds obligations. Unsafe construction localizes one proof obligation; it
does not make later indexing unchecked.

## Ownership and mutation

A basic view borrows its source Engine. It does not increment a hidden shared
reference count, copy storage, or extend the source lifetime by magic. HIR
records the origin even when source syntax omits it:

```zop
fn row values: f32[m, n], index: int -> view f32[n]
    values[index]
```

The returned view originates from `values`, so it cannot outlive that input.
The [memory contract](memory.md#tensors-and-views) defines when an exported
return must state `from` explicitly.

A unique, injective slice of a mutable borrow may itself be mutable. A
zero-stride broadcast or another overlapping layout is readable but cannot
provide ordinary mutable element access. Such updates require a reduction,
accumulation, or atomic operation whose conflict semantics are explicit.

Two disjoint mutable slices may coexist when the compiler proves their storage
codomains do not overlap. Merely having different logical coordinates is not
enough because different layouts can alias the same storage. Dynamic splitting
therefore uses a checked operation that returns both views only after proving
disjointness.

## Runtime representation

The runtime representation is not one fixed array of dimensions and strides for
every tensor. It materializes only facts that remain dynamic after compilation:

```text
TensorValue {
    dynamic Engine state,
    dynamic Layout leaves,
    ownership or borrow state required by the application binary interface,
}
```

Static Engine kind, element access, address-space tag, rank, hierarchy, Shape,
Stride, and ownership proofs remain compiler metadata. Dense Strides derived
from Shape need not be stored. A fully static local tensor can lower to its
Engine iterator with no runtime Layout field.

This borrows the useful part of several established designs without inheriting
their restrictions:

- Java arrays keep one final leading `length`; Zop similarly treats shape as
  stable, but supports any rank and derived views.
- A Go slice stores a pointer, length, and capacity. Zop's ViewEngine and Layout
  provide the cheap shared view, but reject `capacity`: tensor Shape is fixed,
  and reslicing beyond the logical domain would violate bounds reasoning.
- C++ `mdspan` separates extents, a layout mapping, and an accessor. CuTe's
  Engine-plus-Layout model generalizes that decomposition to tagged iterators,
  owned arrays, hierarchy, composition, and swizzles. Zop adopts the CuTe model
  while adding ownership, placement, bounds, and failure contracts.

There is no per-element reference table. A rank-`r` dynamic dense view needs at
most `O(r)` dynamic Layout leaves plus its Engine iterator, while a partially
static or hierarchical profile may need less. Construction is `O(r)` in the
most general dynamic case and constant time for fixed rank; element access does
not depend on tensor element count.

## HIR contract

The parser preserves each selector and whether each endpoint was omitted. Type
checking and normalization produce a canonical HIR selection:

```text
TensorSelect {
    source,
    selectors: [Index | Slice],
    source_shape,
    normalized_bounds,
    result_shape,
    source_engine,
    result_engine,
    source_layout,
    result_layout,
    borrow_origin,
    bounds_proof_or_check,
    mutability,
    source_span,
}
```

Valid HIR satisfies these invariants:

- Every index has one signed mathematical value and one normalized coordinate.
- Every slice has a nonzero step and a derived nonnegative result extent.
- Every removed axis came from an integer selector.
- Every retained axis has one explicit residual layout mode.
- Every access is proven or guarded against both logical and storage bounds.
- Every view has one valid borrow origin.
- Every mutable view is injective and exclusive.
- Static normalization and dynamic normalization have identical semantics.
- Every bracket slice records clipping, and every named slice records its known
  clipping or strict policy before HIR.

The last invariant prevents backend drift. The compiler can implement parser
and diagnostic fixtures before executable slice lowering without leaving the
policy to an MLIR helper.

## MLIR and backend lowering

Positive-stride, representable slices lower naturally to Multi-Level
Intermediate Representation (MLIR)
`tensor.extract_slice` while tensors remain values and to `memref.subview`
after bufferization. Offsets, sizes, and strides retain static operands where
known and dynamic single static-assignment values where required. Rank-reducing
integer selections use the corresponding rank-reduction proof.

The Zop contract is broader than any one MLIR operation. A negative-stride or
hierarchical layout remains a general layout view when a particular dialect
operation cannot express it directly. CPU lowering expands its coordinate map
to integer address arithmetic. GPU lowering preserves the same mapping for
CuTe intermediate representation. A target may reject an unrepresentable
foreign-interface layout, but it may not make it contiguous or copy it
silently.

The reference interpreter evaluates the canonical Engine-plus-Layout
composition directly. For every live coordinate, the interpreter, MLIR path,
Cranelift path, and CuTe path must produce the same Engine index and value or the
same bounds failure.

## Performance contract

Basic indexing and slicing promise zero element movement, not universal
contiguity. The cost model is:

- Static integer indexing: folded offset arithmetic and no descriptor.
- Dynamic integer indexing: normalization when needed, one bounds decision,
  and layout address arithmetic.
- Fixed-rank basic slice: constant-time descriptor construction, often folded.
- Dynamic layout slice: work proportional to dynamic rank or hierarchy, never
  to logical element count.
- Iteration over a view: direct recurrence or vectorized address generation,
  not full layout reevaluation from scratch for every element when algebra
  proves a recurrence.

The optimizer may hoist bounds checks out of a proven loop, fold static
coordinates, combine offset arithmetic, coalesce compatible modes, and
specialize a profitable static layout. It may not:

- omit a check based only on profiling;
- change negative-index normalization;
- change the recorded clipping or strict endpoint policy;
- copy a view into contiguous storage without an explicit source operation;
- change an observable layout;
- turn an aliasing read view into an ordinary mutable view; or
- specialize every symbolic extent merely to remove descriptor arithmetic.

A noncontiguous view can be slower to traverse than a dense tensor because its
requested memory order is different. That is visible source semantics, not
metadata overhead. Performance tooling should report the layout, vectorization
proof, coalescing, and any explicit `relayout` so users can choose deliberately.

## Diagnostics

Index diagnostics use the canonical terms `rank`, `axis`, `extent`, `shape`,
and layout `mode`. They state the failed relation and offer a mechanically valid
repair when one is clear:

```text
error[ZOP-TENSOR-BOUNDS]: index -5 is outside axis 0 with extent 4
  normalized index would be -1; valid indices are -4 through 3
  help: use tensor.at index when this boundary is recoverable
```

```text
error[ZOP-TENSOR-RANK]: 3 selectors cannot index a rank-2 tensor
  help: remove 1 selector
```

```text
error[ZOP-SLICE-STEP]: slice step cannot be zero
  help: use ':' for the complete axis
```

```text
error[ZOP-VIEW-MUT]: this view maps several coordinates to one storage element
  note: axis 0 has extent 8 and stride 0
  help: use an explicit reduction, accumulation, or atomic operation
```

For a likely axis/extent confusion, the diagnostic shows both the axis number
and its extent. For a static index, it points at the selector. For a dynamic
index, the trap or `BoundsError` carries the normalized axis and extent without
formatting or allocating on a device hot path.

The compiler may suggest `.extent axis=i`, `.numel()`, `unsqueeze axis=i`, or a
named `slice` call when those operations match the user's apparent intent. It
must not silently apply the suggestion.

## Required tests

The first executable tensor slice is incomplete until it proves all of these
invariants:

- Index zero, final positive index, first negative index, and final valid
  negative index agree on every backend.
- One-beyond-positive and one-beyond-negative indices fail before memory access.
- Minimum signed integer indices normalize without compiler or runtime
  overflow.
- Missing trailing selectors preserve the expected residual rank and layout.
- Too many selectors fail during type checking.
- Complete integer selection returns an element; partial selection returns a
  correctly originated view.
- Positive, negative, and omitted slice endpoints normalize identically in
  compile-time and runtime paths.
- Step zero is rejected without constructing a descriptor.
- Positive and negative step extent formulas cover empty, singleton, exact,
  and uneven intervals.
- Every basic slice preserves Engine ownership, advances only the iterator, and
  performs zero allocation and zero element copies.
- Reverse, transpose, column, stepped, empty, broadcast, and hierarchical views
  map every live coordinate to the reference offset.
- Empty views never dereference a one-past Engine iterator.
- `shape`, `rank`, `extent`, and `numel()` report logical facts for dense,
  strided, reversed, empty, broadcast, and rank-zero values.
- `numel()` and `layout.cosize` are proven different on broadcast and padded
  layouts.
- Signed layouts never use `cosize` as a substitute for reachable storage-bound
  proof.
- Static shape and layout facts add no runtime descriptor fields.
- A dynamic descriptor materializes each required leaf exactly once and stores
  no redundant logical count.
- Bracket and `.at` access accept the same coordinates and differ only in
  failure channel.
- Mutable access rejects every non-injective or non-exclusive view.
- Proven disjoint mutable slices may coexist; overlapping slices are rejected.
- No backend inserts a hidden copy or contiguous conversion.
- Interpreter, MLIR, Cranelift, JavaScript, WebAssembly, and CuTe execution
  agree wherever that target supports the tensor operation.

The endpoint boundary matrix covers every combination of positive or negative
step; omitted, in-range, or out-of-range start and stop; empty extent; static or
dynamic endpoint; bracket clipping; and named strict failure. Every backend must
match the same normalized result or `BoundsError`.

## Deferred surface choices

These questions do not block the basic model, but the language page must close
them before code relies on their spelling:

- whether ellipsis syntax is worth adding for non-trailing omitted axes;
- whether a `newaxis` selector adds enough value beyond explicit
  `unsqueeze axis=`;
- the exact name and signature of a checked general-layout view constructor;
  and
- whether optional compile-time axis labels belong in tensor types or remain
  framework metadata.

Advanced index tensors and masks are deferred library design, not unresolved
bracket semantics. The first grammar rejects them and directs users to named
`std.tensor` operations.

## Why this composition

No single precedent meets Zop's systems, tensor, GPU, and pseudocode goals:

- [Python indexing](https://docs.python.org/3/reference/datamodel.html#the-standard-type-hierarchy)
  supplies the least surprising readable grammar, negative indices, and
  half-open slices. Zop does not inherit Python's dynamic dispatch or list-slice
  copying.
- [PyTorch tensor views](https://docs.pytorch.org/docs/main/tensor_view.html)
  establish that basic indexing can return noncontiguous views while advanced
  indexing copies. Zop makes allocation and `Mem` more explicit and represents
  every basic view with one language-native layout.
- The [Python Array API indexing
  specification](https://data-apis.org/array-api/latest/API_specification/indexing.html)
  provides portable terminology and exposes where array ecosystems still leave
  edge behavior unspecified.
- The [Go slice specification](https://go.dev/ref/spec#Slice_expressions) and
  [slice descriptor explanation](https://go.dev/blog/slices) demonstrate cheap
  shared-storage views and strict bounds. Zop omits capacity and generalizes the
  descriptor to ranked, fixed-shape tensors.
- [Java array length](https://docs.oracle.com/javase/specs/jls/se21/html/jls-10.html#jls-10.7)
  demonstrates a stable, constant-time logical bound. Zop generalizes one
  leading length into an immutable shape without storing references per element.
- Rust [`ndarray::Slice`](https://docs.rs/ndarray/latest/ndarray/struct.Slice.html)
  demonstrates exclusive stops, negative endpoints, nonzero steps, and strict
  view construction. Zop puts the common notation in the language rather than
  a macro.
- C++ [`submdspan`](https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2023/p2630r3.html)
  demonstrates preservation of static extents while separating extents, layout
  mapping, accessor, and data handle. Zop adds checked ownership and one
  high-level tensor contract.
- [PyCuTe tensor indexing](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/docs/05_tensor.md)
  demonstrates that partial coordinates naturally produce an offset and
  residual layout. Zop adopts that algebra as source semantics rather than only
  as an NVIDIA backend implementation detail.
- [CUTLASS CuTe Tensor](https://github.com/NVIDIA/cutlass/blob/6c68991985ca8b09594ac6fd43abbfd5830c4140/media/docs/cpp/cute/03_tensor.md)
  defines Tensor as Engine plus Layout, and its implementation advances the
  Engine iterator when slicing instead of storing a separate origin.
- MLIR [`tensor.extract_slice`](https://mlir.llvm.org/docs/Dialects/TensorOps/#tensorextract-slice-mlirtensorextractsliceop)
  and [`memref.subview`](https://mlir.llvm.org/docs/Dialects/MemRef/#memrefsubview-mlirmemrefsubviewop)
  provide direct lowering for representable offsets, sizes, strides, and rank
  reductions. Zop retains general layout semantics when those operations are
  narrower than the source contract.

The composition is stronger than any element in isolation because the syntax,
type facts, Engine advance, residual Layout algebra, ownership proof, runtime
value, and backend lowering all describe the same coordinate transformation.
There is no semantic seam where a Python-looking slice becomes a hidden copy, a
GPU-only CuTe object, or an unchecked raw pointer.
