# Tensor-layout examples

These examples make the [tensor-layout contract](layouts.md) concrete. They are
Zop pseudocode until tensor parsing, indexing, views, and `Layout` exist in the
Rust bootstrap. CuTe algebra and partial-coordinate results were checked against
PyCuTe revision
[`f14cb106`](https://github.com/NVlabs/CuTe/commit/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32).
Zop's stepped and negative slices extend PyCuTe's current full-axis slicing and
are checked against the formulas in the [indexing contract](indexing.md).

## Dense tensor literal

A nested literal uses a right-major layout. The final logical axis is
contiguous:

```zop
test "dense literals are right-major"
    matrix = [[1, 2, 3], [4, 5, 6]]

    expect equal(matrix.layout, Layout(
        shape=(2, 3),
        stride=(3, 1),
    ))

    expect equal(matrix.layout(0, 0), 0)
    expect equal(matrix.layout(0, 2), 2)
    expect equal(matrix.layout(1, 0), 3)
    expect equal(matrix.layout(1, 2), 5)
    expect equal(matrix[1, 2], 6)
```

Conceptually, indexing performs two operations:

```text
offset = matrix.layout(1, 2)  # 5
value = matrix.engine[offset] # 6
```

The compiler may fuse both operations into one address calculation. The
separate model remains visible through `.engine` and `.layout`.

Reference: [PyCuTe tensor construction and indexing](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/docs/05_tensor.md).

## Logical size and physical span

Tensor size queries name the exact logical fact being requested:

```zop
test "tensor queries are not layout spans"
    matrix = [[1, 2, 3], [4, 5, 6]]

    expect equal(matrix.rank, 2)
    expect equal(matrix.shape, (2, 3))
    expect equal(matrix.extent(axis=0), 2)
    expect equal(matrix.extent(axis=1), 3)
    expect equal(matrix.numel(), 6)
    expect equal(matrix.layout.cosize, 6)
```

`numel()` is the product of logical extents. For these nonnegative integral
layouts, `layout.cosize` is the scalar codomain span. A broadcast proves why
one name cannot safely stand for both:

```zop
test "broadcast numel exceeds stored span"
    values = [10, 20, 30]
    repeated = values.expand (4, 3)

    expect equal(repeated.shape, (4, 3))
    expect equal(repeated.numel(), 12)
    expect equal(repeated.layout, Layout(
        shape=(4, 3),
        stride=(0, 1),
    ))
    expect equal(repeated.layout.cosize, 3)
```

The runtime does not store both results. Shape is the logical source of truth;
`numel()` derives its product, and the layout derives its codomain. Static
queries fold completely.

## Basic slices derive residual layouts

An integer fixes one coordinate and removes its axis. A slice retains its axis
and restricts it to an arithmetic progression:

```zop
test "rows and columns are zero-copy views"
    matrix = [[0, 1, 2], [3, 4, 5], [6, 7, 8]]

    row = matrix[1]
    column = matrix[:, 1]
    corner = matrix[1:, 1:]

    expect equal(row.shape, (3,))
    expect equal(row.layout.stride, (1,))
    expect equal(row[0], 3)
    expect equal(row[2], 5)

    expect equal(column.shape, (3,))
    expect equal(column.layout.stride, (3,))
    expect equal(column[0], 1)
    expect equal(column[2], 7)

    expect equal(corner.shape, (2, 2))
    expect equal(corner.layout.stride, (3, 1))
    expect equal(corner[0, 0], 4)
    expect equal(corner[1, 1], 8)
```

All three affine results borrow `matrix`'s Engine and record `matrix` as their
Zop ownership origin. Their fixed-coordinate contributions advance the Engine,
and their free coordinates remain in residual affine Layouts. No test needs an
allocation expectation because basic slice lowering is invalid if it emits an
allocation or element copy at all.

Stepping multiplies the selected axis's source stride:

```zop
test "slice step multiplies source stride"
    values = [0, 1, 2, 3, 4, 5]
    evens = values[::2]

    expect equal(evens.shape, (3,))
    expect equal(evens.layout.stride, (2,))
    expect equal(evens[0], 0)
    expect equal(evens[1], 2)
    expect equal(evens[2], 4)
```

A negative step first advances the Engine, then uses a negative stride:

```zop
test "reverse is a negative-stride view"
    values = [10, 20, 30, 40]
    reverse = values[::-1]

    expect equal(reverse.shape, (4,))
    expect equal(reverse.layout.stride, (-1,))
    expect equal(reverse[0], 40)
    expect equal(reverse[-1], 10)
```

Negative integer normalization and negative layout stride are separate. The
first converts a user coordinate such as `-1` into a valid logical coordinate.
The second maps increasing logical coordinates in `reverse` to decreasing
Engine offsets. Bounds checking happens in the logical domain before Layout
evaluation dereferences the Engine.

The complete [indexing contract](indexing.md) defines clipped bracket endpoints,
explicit strict slicing, formulas, ownership proof, lowering, diagnostics, and
the required boundary matrix.

## Printing tensors and views

Formatting is built into the tensor contract; writing the result still requires
`Io`:

```zop
fn show io: Io -> Unit or fails with IoError
    matrix = [[1, 2, 3], [4, 5, 6]]
    try to print io, matrix
```

```text
i64[2, 3] cpu engine=ArrayEngine[host] layout=(2, 3):(3, 1)
[[1, 2, 3],
 [4, 5, 6]]
```

A view prints in logical order and identifies itself:

```zop
# Continuing inside `show`.
column = matrix[:, 1]
try to print io, column
```

```text
i64[2] cpu engine=ViewEngine[host] layout=2:3
[2, 5]
```

Large output is deterministic:

```text
f32[1024, 4096] cpu engine=ArrayEngine[host] layout=(1024, 4096):(4096, 1)
[[0.12, 0.08, 0.31, ..., 0.17, 0.26, 0.09],
 [0.22, 0.14, 0.05, ..., 0.29, 0.10, 0.44],
 [0.03, 0.18, 0.27, ..., 0.06, 0.21, 0.38],
 ...,
 [0.18, 0.25, 0.13, ..., 0.32, 0.16, 0.28],
 [0.04, 0.37, 0.20, ..., 0.15, 0.24, 0.30],
 [0.02, 0.19, 0.41, ..., 0.11, 0.33, 0.07]]
4,194,304 elements
```

The default prints at most 1,000 elements and keeps three entries at each edge.
One call may override that policy:

```zop
try to print io, matrix, limit=64, edge=2
```

Printing `matrix.layout` emits `(2, 3):(3, 1)`. Use `zop layout show` when an
offset grid, SVG, or thread/value visualization is wanted.

## Row-major and column-major

Shape does not determine storage order. Stride does:

```zop
test "stride determines storage order"
    row_major = Layout shape=(4, 8), stride=(8, 1)
    column_major = Layout shape=(4, 8), stride=(1, 4)

    expect equal(row_major(2, 3), 19)    # 2 * 8 + 3 * 1
    expect equal(column_major(2, 3), 14) # 2 * 1 + 3 * 4
```

`zop layout show` renders the column-major offsets as:

```text
(4, 8):(1, 4)
 0   4   8  12  16  20  24  28
 1   5   9  13  17  21  25  29
 2   6  10  14  18  22  26  30
 3   7  11  15  19  23  27  31
```

Reference: [PyCuTe layout construction](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/docs/03_layout.md#constructing-a-layout).

## Hierarchical coordinates

Hierarchy records tiling without changing the fact that a layout is one pure
function:

```zop
test "flat and hierarchical coordinates agree"
    layout = Layout(
        shape=(3, (2, 4)),
        stride=(2, (1, 6)),
    )

    expect equal(layout(7), 8)
    expect equal(layout(1, 2), 8)
    expect equal(layout(1, (0, 1)), 8)
```

The first call uses one integral coordinate. The second uses one coordinate per
top-level mode. The third mirrors the nested shape exactly. All three identify
the same logical point.

Reference: [PyCuTe coordinate forms](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/docs/03_layout.md#calling-a-layout).

## Coordinate layouts

A basis stride places each contribution into a named codomain mode instead of
adding every contribution into one integer:

```zop
from core.layout import basis

test "basis strides preserve logical coordinates"
    identity = Layout(
        shape=(3, 4),
        stride=(basis(0), basis(1)),
    )

    expect equal(identity(1, 2), (1, 2))
    expect equal(identity.coshape, (3, 4))
```

`basis(0)` and `basis(1)` are Zop's readable spelling of CuTe's `E(0)` and
`E(1)` basis strides. Identity layouts let a kernel partition coordinates and
construct bounds predicates with the same operations used for data layouts.

Reference: [PyCuTe identity tensors and coordinate strides](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/docs/05_tensor.md#identity_tensorshape).

## Algebra domain and storage bounds

Plain layout evaluation is total integer algebra, while tensor access is
bounded:

```zop
test "layout evaluation does not imply memory access"
    sparse = Layout shape=4, stride=2

    expect equal(sparse(10), 20) # legal algebra on the extended domain
```

The same coordinate is not a legal tensor index because `10` is outside the
logical extent `4`. For this finite nonnegative affine layout, storage capacity
is checked separately through the layout's codomain size:

```zop
test "layout codomain determines required storage"
    padded = Layout shape=(4, 8), stride=(16, 1)

    expect equal(padded(3, 7), 55)
    expect equal(padded.cosize, 56)
```

Safely binding `padded` to fewer than 56 elements is rejected. Calling
`padded(3, 7)` remains pure and safe because it computes an integer without
dereferencing storage.

References:

- [PyCuTe extended-domain evaluation](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/docs/03_layout.md#calling-on-out-of-bounds-coordinates)
- [PyCuTe codomain properties](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/docs/03_layout.md#properties-of-a-layout)

## Zero-copy rows and columns

These affine slices advance the Engine and retain a residual Layout:

```zop
fn inspect matrix: f32[4, 8]
    row = matrix[1, :]
    column = matrix[:, 2]

    # Same storage, starting eight elements later.
    row.layout     # Layout shape=8, stride=1

    # Same storage, starting two elements later.
    column.layout  # Layout shape=4, stride=8
```

No elements move. HIR records the source owner, advanced Engine, and residual
Layout. A row may be contiguous while a column is strided.

References:

- [PyCuTe layout slicing](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/docs/03_layout.md#slicing-a-layout)
- [PyCuTe tensor slicing](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/docs/05_tensor.md#reading-and-writing)

## Slicing through nonlinear composition

A fixed coordinate cannot always become an Engine displacement. Consider
`S(x) = x xor (x >> 2)` composed outside a row-major `4 x 4` layout:

```text
parent = Compose(outer=S, offset=0,
                 inner=Affine(shape=(4, 4), stride=(4, 1)))
parent(row, column) = S(row * 4 + column)
```

Fixing row `1` contributes `4` before `S`. The correct residual keeps that
contribution inside the composition:

```text
residual = Compose(outer=S, offset=4,
                   inner=Affine(shape=4, stride=1))
engine_delta = 0
```

Advancing the Engine would put the contribution after `S` and change the
address:

```text
correct(0) = S(4) = 5
wrong(0)   = 4 + S(0) = 4
```

Here `free` marks a coordinate that remains in the residual domain. The
invariant test compares every residual coordinate with its parent coordinate:

```text
residual, engine_delta = slice_and_offset((1, free), parent)

for column in range(4):
    assert engine_delta + residual(column) == parent(1, column)
```

Some restricted swizzled slices are provably affine. The compiler may decay
the residual to `Shape:Stride` and externalize a displacement only after
proving the same pointwise identity.

References:

- [CUTLASS swizzle slicing](https://github.com/NVIDIA/cutlass/blob/7107b05535f8977f5ecb9d01ee203205b1fd9bc4/include/cute/swizzle_layout.hpp)
- [`tensor-layouts` composed slicing](https://github.com/jduprat/tensor-layouts/blob/d9f51a435c02eb600a05f72508e681bd33dadee9/src/tensor_layouts/layouts/algebra.py#L1121)

## Coalescing

`coalesce` removes hierarchy and size-one modes only when integral coordinate
evaluation stays identical:

```zop
test "coalescing preserves integral evaluation"
    nested = Layout(
        shape=(2, (1, 6)),
        stride=(1, (6, 2)),
    )

    flat = nested.coalesce()
    expect equal(flat, Layout shape=12, stride=1)

    for index in range(12)
        expect equal(flat(index), nested(index))
```

Reference: [PyCuTe `coalesce`](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/docs/04_layout_algebra.md#coalesce).

## Composition

Composition applies the inner coordinate map before the outer storage map:

```zop
test "composition combines coordinate maps"
    outer = Layout shape=20, stride=2
    inner = Layout shape=(5, 4), stride=(4, 1)

    result = outer.compose inner

    expect equal(result, Layout(
        shape=(5, 4),
        stride=(8, 2),
    ))

    for index in range(inner.size)
        expect equal(result(index), outer(inner(index)))
```

Reference: [PyCuTe `composition`](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/docs/04_layout_algebra.md#composition).

## Tiling with logical divide

Logical divide splits a linear layout into the selected tile and the grid of
those tiles:

```zop
test "logical divide separates tile and grid"
    linear = Layout shape=24, stride=1
    tile = Layout shape=4, stride=2

    tiled = linear.logical_divide tile

    expect equal(tiled, Layout(
        shape=(4, (2, 3)),
        stride=(2, (1, 8)),
    ))
```

Mode zero enumerates the four elements selected by `tile`. Mode one enumerates
the interleaved groups needed to cover all 24 source elements.

Reference: [PyCuTe `logical_divide`](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/docs/04_layout_algebra.md#logical-divide).

## Zipped layout division

CuTe's `zipped_divide` is layout algebra, not iteration over several
collections. It groups all selected tile modes together and all remainder modes
together:

```zop
test "zipped divide groups tile and remainder modes"
    source = Layout shape=(9, 32), stride=(1, 9)
    tiler = (
        Layout shape=3, stride=3,
        Layout shape=(2, 4), stride=(1, 8),
    )

    divided = source.zipped_divide tiler

    expect equal(divided, Layout(
        shape=((3, (2, 4)), (3, 4)),
        stride=((3, (9, 72)), (1, 18)),
    ))
```

The first top-level mode contains the tiled coordinates. The second contains
their remainders. This operation is independent of the standard iterable
`zip`, whose `strict` argument controls unequal iterator lengths.

Reference: [PyCuTe `zipped_divide`](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/docs/04_layout_algebra.md#zipped_divide-tiled_divide-flat_divide).

## Broadcast and mutation

A tensor `expand` follows trailing axes and produces a zero-stride view:

```zop
test "expand prepends a zero-stride axis"
    bias = [1.0, 2.0, 3.0]
    expanded = bias.expand shape=(2, 3)

    expect equal(expanded.shape, (2, 3))
    expect equal(expanded.layout.stride, (0, 1))
    expect equal(expanded[0, 2], expanded[1, 2])
```

A zero stride repeats one storage location across a logical mode:

```zop
test "zero stride broadcasts without copying"
    broadcast = Layout shape=(3, 4), stride=(0, 1)

    expect equal(broadcast(0, 2), 2)
    expect equal(broadcast(1, 2), 2)
    expect equal(broadcast(2, 2), 2)
```

Binding this layout to four stored values creates a read-only `3 x 4` view.
Writing `view[0, 2]` would also change `view[1, 2]` and `view[2, 2]`, so the type
checker rejects an ordinary mutable borrow:

```zop
fn overwrite mut values: f32[3, 4]
    values[1, 2] = 0
    # compile error when values.layout is non-injective
```

Reduction, accumulation, or an atomic operation may define the collision
semantics explicitly.

Reference: [PyCuTe COPY broadcast cases](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/test/test_alg_copy.py).

## Thread and value mapping

A thread/value layout maps one thread and one thread-local value to a position
in a matrix-instruction tile:

```zop
test "thread and value pairs cover an sm80 accumulator tile"
    tv = Layout(
        shape=((4, 8), (2, 2)),
        stride=((32, 1), (16, 8)),
    )

    expect equal(tv(0, 0), 0)
    expect equal(tv(1, 0), 32)
    expect equal(tv(0, 1), 16)
    expect equal(tv(1, 1), 48)
    expect equal(tv(31, 3), 127)
```

The first top-level mode is a hierarchical thread identifier. The second is a
hierarchical value identifier. Together they cover 128 positions in an SM80
`16 x 8` accumulator tile.

Reference: [PyCuTe thread/value visualization](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/README.md#visualization).

## Composing schedule and data Layout

The same algebra connects a thread/value schedule to a tensor's data Layout:

```zop
test "thread schedule composes with data layout"
    data_layout = Layout shape=(16, 8), stride=(8, 1)
    tv = Layout(
        shape=((4, 8), (2, 2)),
        stride=((32, 1), (16, 8)),
    )

    access = data_layout.compose tv

    expect equal(access, Layout(
        shape=((4, 8), (2, 2)),
        stride=((2, 8), (1, 64)),
    ))

    expect equal(access(1, 1), 3)
    expect equal(access(31, 3), 127)
```

This result is the exact address map used to inspect coalescing, vector width,
and memory-bank behavior before code generation.

## Mode rearrangement

PyCuTe's `einfold` example demonstrates that permutation, grouping, and
ungrouping can rebuild only the layout hierarchy. Zop should expose the same
capability through a tensor-library API after its source spelling is settled:

```zop
# Illustrative tensor-library spelling, not accepted grammar.
reordered = rearrange tensor, "(ab)cde -> c(ade)b"

# Input shape:  ((12, 4), 42, 5, 7)
# Output shape: (42, (12, 5, 7), 4)
# Storage:      unchanged
```

Repeating or dropping a mode may create overlap or discard reachability. The
layout and ownership checker must preserve view origins and reject unsafe
mutation.

Reference: [PyCuTe `einfold`](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/examples/einfold.py).

## Tensor contraction

`einsum` is tensor-library source rather than a language keyword:

```zop
fn contraction left: f32[m, k], right: f32[n, k] -> f32[m, n]
    einsum "mk,nk->mn", left, right
```

The implementation classifies labels and builds zero-copy canonical views:

```text
left  -> (M, K, L)
right -> (N, K, L)
out   -> (M, N, L)

batch_gemm(left_view, right_view, out_view)
```

The first implementation may support a narrower contraction grammar than
NumPy. Unsupported repetition, broadcasting, or output allocation fails with a
typed diagnostic instead of selecting another execution path.

Reference: [PyCuTe `einsum`](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/examples/einsum.py).

## One copy operation, many mappings

Users write one logical copy:

```zop
copy source, destination
```

The layouts determine the concrete operation:

| Source layout | Destination layout | Result |
| --- | --- | --- |
| `(n):1` | `(n):1` | Contiguous or vectorized copy |
| `(m,n):(n,1)` | `(m,n):(1,m)` | Transpose |
| `(m,n):(0,1)` | `(m,n):(n,1)` | Broadcast |
| Regular sparse strides | Compact strides | Regular gather |
| Compact strides | Regular sparse strides | Regular scatter |

Compiler pseudocode makes the hidden analysis inspectable:

```text
CopyPlan {
    domain: greatest_common_domain(source.layout, destination.layout)
    collisions: nullspace(destination.layout)
    source_order: right_inverse(source.layout)
    vector_width: common_alignment(source.layout, destination.layout)
}
```

The plan is compiler IR rather than source code users must write. A collision
without an explicit reduction, accumulation, or atomic policy is rejected.

References:

- [PyCuTe COPY walkthrough](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/examples/algorithms/copy.ipynb)
- [PyCuTe COPY implementation](https://github.com/NVlabs/CuTe/blob/f14cb1062f8bbdeeded8f6d52b04dbdea7092a32/pycute/alg/copy.py)

## Lowering trace

One source layout has one semantic value through every target:

```zop
layout = Layout shape=(2, 3), stride=(3, 1)
offset = layout(1, 2)
```

```text
Zop HIR
    Layout<(2, 3):(3, 1)>
    offset = LayoutEval(layout, (1, 2))

CPU lowering
    offset = 1 * 3 + 2 * 1

CuTe IR lowering for kn
    !cute.layout<"(2,3):(3,1)">

Result
    offset = 5
```

The interpreter, Cranelift path, and CuTe intermediate representation path must
produce the same offset for every coordinate in the conformance corpus.

## Dynamic leaves

The profile remains static while individual extents and strides may remain
dynamic:

```zop
fn inspect matrix: f32[m, n] -> Layout
    matrix.layout

# right-major layout: (m, n):(n, 1)
```

Conceptual runtime representation:

```text
TensorValue {
    engine
    m
    n
    placement
    ownership
}
```

`engine` is the dynamic iterator state of a statically known Engine profile.
The Stride `n` reuses the same dynamic value as the extent. The runtime value
does not store another copy merely because `n` occurs twice in the Layout
expression. A fully static `(2,3):(3,1)` Layout contributes no runtime Layout
field.

Reference: [CuTe IR sparse dynamic-leaf lowering](https://github.com/NVIDIA/cutlass/blob/9d79de097be048b67be8e527ceced4ba017e5e1d/cutlass_compiler/cute_ir/lib/Conversion/CuteToBase/CuteTypeConverter.cpp#L37-L63).

## CuTe-native ABI profiles

Foreign boundaries constrain the complete `Layout`, not a nominal tensor class.
The source spelling below is illustrative while the semantic contract is
canonical.

A dense right-major boundary states its profile:

```zop
foreign fn sum values: f32[m, n] -> f32
where values.layout matches Layout(
    shape=(m, n),
    stride=(n, 1),
)
```

Only `m` and `n` remain dynamic. The ABI does not pass rank, hierarchy, unit
stride, or another copy of `n`.

A general two-mode integral boundary exposes independent dynamic leaves:

```zop
foreign fn sum_general values: f32[m, n] -> f32
where values.layout matches Layout(
    shape=(m, n),
    stride=(row_stride, column_stride),
)
```

The lowered fields are an advanced `ViewEngine`, `m`, `n`, `row_stride`, and
`column_stride`. They instantiate one Engine-plus-Layout profile; they are not
a universal sizes-and-strides tensor object. A slice has already accumulated
its offset into the Engine, exactly as CUTLASS CuTe and PyCuTe do.

A tiled boundary preserves mode hierarchy:

```zop
foreign fn consume_tiles values: f32[m, n]
where values.layout matches Layout(
    shape=((tiles, tile), n),
    stride=((tile * row_stride, row_stride), column_stride),
)
and m == tiles * tile
and tile is known
```

Here `((tiles, tile), n)` is observable ABI structure. The foreign contract can
address the outer row tile, inner row coordinate, and column independently.
Flattening it to `(m, n)` would discard information the consumer explicitly
requested.

An incompatible tensor must create a proven view or explicit owned conversion:

```zop
dense = try to values.relayout mem, DenseRowMajor
result = sum dense
```

No foreign call inserts that `relayout` silently. See the complete
[layout ABI contract](layouts.md#runtime-and-application-binary-interface).
