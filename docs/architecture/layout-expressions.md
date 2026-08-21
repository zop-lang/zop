# Layout expressions

Zop uses one public `Layout` concept with two exact internal forms. Affine
layouts are hierarchical `Shape:Stride` maps. Composed layouts preserve an
outer map, an offset inside that map, and an inner layout. This distinction is
required for swizzles and any other coordinate map that is not linear over
ordinary integer addition.

> **Status:** This page corrects and extends the target tensor contract. Neither
> layout form is implemented in the Rust bootstrap. The first tensor slice must
> implement these forms before accepting swizzled views or general composition.

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

Sources: [CUTLASS swizzle slicing](https://github.com/NVIDIA/cutlass/blob/7107b05535f8977f5ecb9d01ee203205b1fd9bc4/include/cute/swizzle_layout.hpp)
and [`tensor-layouts` composed slicing](https://github.com/jduprat/tensor-layouts/blob/d9f51a435c02eb600a05f72508e681bd33dadee9/src/tensor_layouts/layouts/algebra.py#L1121).
