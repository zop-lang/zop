# Layout expressions

Zop uses one public `Layout` concept with two exact internal forms. Affine
layouts are hierarchical `Shape:Stride` maps. Composed layouts preserve an
outer map, an offset inside that map, and an inner layout. This distinction is
required for swizzles and any other coordinate map that is not linear over
ordinary integer addition.

> **Status:** This page corrects and extends the target tensor contract. Neither
> layout form is complete in the Rust bootstrap. Flat affine and nested
> swizzle-composed evaluation and fixed/free slicing are implemented. Hierarchy,
> tensors, HIR, storage, analysis, atoms, and lowering remain contracts.

The executable invariant compares `engine_delta + residual(free)` with
`parent(fixed, free)` for affine, composed, and nested-composed layouts.

## Representation

Compiler pseudocode uses this tagged value:

```text
LayoutExpr =
    Affine {
        shape: Shape,
        stride: Stride,
    }
  | Compose {
        outer: LayoutMap,
        offset: Coordinate,
        inner: LayoutExpr,
    }
```

`LayoutMap` may be an affine layout, a swizzle, or another compiler-known pure
coordinate map. Evaluation is exact:

```text
eval(Affine(shape, stride), coordinate)
    = dot(natural_coordinate(coordinate, shape), stride)

eval(Compose(outer, offset, inner), coordinate)
    = outer(offset + eval(inner, coordinate))
```

The inner layout owns the logical domain. The outer map owns the final
codomain transformation. The offset is applied between them. Reordering those
three operations changes the function.

Source continues to use the familiar name `Layout`. The compiler knows the
concrete layout-expression profile statically, just as it knows a tensor's
rank and element type. Users encounter the distinction only when requesting an
affine-only property or inspecting a composed value.

## Affine layouts

An affine layout is the ordinary CuTe `Shape:Stride` pair:

```zop
matrix = Layout shape=(4, 8), stride=(8, 1)

expect equal(matrix(2, 3), 19)
expect equal(matrix.stride, (8, 1))
```

Its coordinate map is an integer inner product. Transpose, padding, broadcast,
regular striding, and negative-stride reversal remain affine. These operations
can derive another congruent shape and stride tree directly.

A function that reads `.stride` requires an affine layout constraint. The
compiler does not return an optional stride or wait until runtime to discover
the form. Exact constraint spelling remains part of the general layout-profile
grammar, but the static requirement is fixed.

## Composed layouts

A composed layout preserves function composition rather than pretending every
map has a stride tree:

```text
swizzled = Compose(
    outer=Swizzle(bits=2, base=0, shift=2),
    offset=0,
    inner=Affine(shape=(8, 8), stride=(8, 1)),
)
```

The canonical text form is:

```text
outer o {offset} o inner
```

A composed layout has no universal `.stride`. A swizzle is a bitwise
permutation, not multiplication by one integer stride per mode. Code that
requires a stride tree must require an affine layout profile.

The compiler may simplify a composition to an affine layout only after proving
pointwise equivalence over its complete domain. A failed simplification leaves
the exact `Compose` value intact; it never drops the outer map or offset.

## Slicing and offsets

Tensor storage uses an external Engine displacement:

```text
tensor_address(coordinate) = engine.base + layout(coordinate)
```

A composed layout also has an internal offset:

```text
layout(coordinate) = outer(internal_offset + inner(coordinate))
```

The two offsets sit on opposite sides of `outer`. They may be combined only
when algebra proves that moving the internal contribution through `outer`
preserves every address.

Consider the illustrative nonlinear bit map `S(x) = x xor (x >> 2)` and a
row-major inner layout. Fixing row `1` contributes `4`:

```text
parent(1, column) = S(4 + column)
```

Keeping `4` inside the composition is correct:

```text
child(column) = S(4 + column)
```

Blindly advancing the Engine is wrong:

```text
wrong(column) = 4 + S(column)

child(0) = S(4) = 5
wrong(0) = 4 + S(0) = 4
```

The general slicing primitive therefore returns both a residual layout
expression and an external Engine displacement:

```text
residual, engine_delta = slice_and_offset(coordinate, layout)
```

It must prove:

```text
parent(fixed, free) = engine_delta + residual(free)
```

For an affine layout, `engine_delta` normally carries the fixed-coordinate
contribution and `residual` remains affine. For a nonlinear composition,
`engine_delta` may be zero while the contribution is accumulated into the
composition's internal offset. A swizzled slice may decay to an affine residual
only when the compiler proves the restricted map is affine.

This does not reintroduce a tensor `origin` field. The Engine displacement is
external storage state. A composition offset is part of the Layout function.
They are distinct facts with distinct algebraic positions.

Sources for representation and slicing:
[CUTLASS swizzle slicing](https://github.com/NVIDIA/cutlass/blob/7107b05535f8977f5ecb9d01ee203205b1fd9bc4/include/cute/swizzle_layout.hpp)
and [`tensor-layouts` composed slicing](https://github.com/jduprat/tensor-layouts/blob/d9f51a435c02eb600a05f72508e681bd33dadee9/src/tensor_layouts/layouts/algebra.py#L1121).

## Query contracts

<!-- markdownlint-disable MD013 -->

| Query | Affine | Composed |
| --- | --- | --- |
| `.shape`, `.rank`, `.depth`, `.size` | Derived from `shape` | Derived from the inner domain |
| `.stride` | Congruent stride tree | Illegal unless the expression first proves affine |
| `.cosize` | CuTe codomain size | Profile-specific algebraic query, not addressed storage bounds |
| `.injective` | Proven symbolically or exhaustively | Proven from the complete expression |
| `.compact` | Proven from shape and stride | Proven pointwise or by a composition-specific rule |
| storage bounds | Engine plus affine min/max | Engine plus the complete composed image |

<!-- markdownlint-enable MD013 -->

CuTe `cosize` is the size of a function's codomain, not necessarily the number
of visited addresses. For a swizzled composed layout, upstream CuTe returns the
`cosize` of the inner layout. It is not an exact minimum and maximum address
calculation for an arbitrary outer map. A general composition whose profile
does not define `cosize` rejects the query at compile time.

Zop never uses `.cosize` alone as a storage-safety proof. The required sets are:

```text
logical_domain = coordinates(layout.shape)
addressed = { engine.base + layout(c) | c in logical_domain }
```

Every addressed index must be nonnegative and smaller than the Engine's backing
allocation. Static layouts may prove this symbolically or by exhaustive
evaluation. Dynamic layouts carry the minimum proof or guard required by their
profile.

## Inverses and negative internal offsets

Inverting an offset-bearing composed layout may move the outer map into the
inner position and negate the offset. The result can be valid algebra while
producing negative values for early coordinates.

```text
forward = Swizzle o {4} o Affine(32:1)
inverse = Affine(32:1) o {-4} o Swizzle
```

The inverse exists to compose with the forward map, where the intermediate
negative contribution cancels. It is not automatically a valid direct storage
layout. Attaching an Engine requires the ordinary addressed-set proof.

Operations whose mathematics is undefined for an inverse-form nonlinear
layout fail explicitly. `complement`, `logical_divide`, and `logical_product`
remain unavailable until a sound definition and an upstream oracle exist. The
compiler never approximates them with the inner affine layout.

## Equality and analysis

Structural equality asks whether two layout-expression trees are identical.
Functional equality asks whether they map every logical coordinate to the same
result. Algebraic rewrites need the second property even when their structures
differ.

```text
functionally_equal(left, right)
    = left.size == right.size
      and every i in [0, left.size) satisfies left(i) == right(i)
```

The compiler uses symbolic proofs when available and exhaustive evaluation for
small static domains. `zop layout check` may expose functional equality,
address image, injectivity, surjectivity, contiguous runs, gaps, and aliasing
groups. These remain tooling and compiler facts rather than duplicate syntax.

Bank-conflict and coalescing reports take their hardware geometry from the
target profile. They never hardcode one vendor's warp size, bank width, bank
count, issue-group size, or transaction segment size into `Layout`.

## Derived algebra surface

`tensor-layouts` exposes variants such as `tiled_divide`, `flat_divide`,
`zipped_product`, `blocked_product`, `raked_product`, `group`, `sort`, `upcast`,
and `downcast`. Zop does not make every helper a core primitive.

Divide and product variants rearrange hierarchy around canonical logical
operations. Grouping and sorting are layout-library transformations. Upcast and
downcast are instances of byte-preserving `recast`. Maximum common layout and
vector calculations belong to compiler passes.

## Binary linear-layout analysis

Swizzles and many thread/value maps are linear over the finite field GF(2),
whose only values are zero and one and whose addition is exclusive-or. The
compiler may lower that subset to a binary matrix:

```text
output_bits = matrix * coordinate_bits  over GF(2)
```

This is an analysis representation, not source syntax or tensor ABI. A nonzero
composition offset is ordinarily integer addition with carries, not a GF(2)
translation; conversion rejects it unless a stronger proof removes the carries.

This provides a bridge to Triton-style linear layouts without replacing CuTe
`Engine + Layout` as the source model.

## Hardware atoms

Matrix-multiply and copy instructions enter the backend through target data:

```text
MmaAtom {
    instruction,
    shape_mnk,
    a_thread_value_layout,
    b_thread_value_layout,
    c_thread_value_layout,
    element_types,
    target_features,
}

CopyAtom {
    instruction,
    source_thread_value_layout,
    destination_thread_value_layout,
    element_types,
    target_features,
}
```

Atoms are compiler-known instruction descriptions, not tensor types or kernel
syntax. The initial registry derives from vendor specifications and is checked
against executable oracles for NVIDIA, AMD, Intel Xe, and Intel Advanced Matrix
Extensions (AMX) where those tools are available.

## Conformance

CUTLASS CuTe remains normative. PyCuTe and `tensor-layouts` are independent
executable references. Zop generates small fixtures outside the build and
checks reviewed results into the repository. A compiler build never imports
Python or downloads an oracle.

Required cases include:

- affine versus composed structural and functional equality;
- slicing before and after a swizzle with internal nonzero offsets;
- parent and residual address equivalence at every coordinate;
- composed `.stride` rejection and `cosize` versus addressed bounds;
- inverse-form negative offsets and unsupported nonlinear operations;
- GF(2) round trips for eligible layouts and rejection otherwise;
- thread/value coverage for every registered atom; and
- bank-conflict and coalescing reports under actual target geometry.

## Further references

<!-- markdownlint-disable MD013 -->

- [CUTLASS CuTe layout definitions](https://github.com/NVIDIA/cutlass/blob/7107b05535f8977f5ecb9d01ee203205b1fd9bc4/media/docs/cpp/cute/01_layout.md)
- [`tensor-layouts` composed representation](https://github.com/jduprat/tensor-layouts/blob/d9f51a435c02eb600a05f72508e681bd33dadee9/src/tensor_layouts/layouts/expr.py)
- [`tensor-layouts` analysis helpers](https://github.com/jduprat/tensor-layouts/blob/d9f51a435c02eb600a05f72508e681bd33dadee9/src/tensor_layouts/analysis.py)
- [`tensor-layouts` hardware atoms](https://github.com/jduprat/tensor-layouts/tree/d9f51a435c02eb600a05f72508e681bd33dadee9/src/tensor_layouts)
- [Linear Layouts paper](https://arxiv.org/abs/2505.23819)

<!-- markdownlint-enable MD013 -->
