# Compile-time values

Zop distinguishes a value's data type from when that value becomes
available. The `known` qualifier requires a function argument during
compilation:

```zop
kn blocked_matmul a: f32[m, k], b: f32[k, n], tile: known int
```

`tile` has type `int`. `known` is a binding-time qualifier on that parameter,
not part of its name or access mode. An unqualified parameter is available at
runtime by default. Only function parameter types accept `known` in the initial
contract.

Floating-point profiles use the same mechanism instead of a second annotation
grammar or ambient compiler switch:

```zop
kn dot(
    left: f32[n],
    right: f32[n],
    float_profile: known FloatProfile = Strict,
) -> f32
    left @ right
```

Passing `float_profile=Native` changes the permitted arithmetic rewrites and
artifact identity. It contributes no runtime argument.

> **Status:** This page defines target semantics. The Rust bootstrap does not
> implement `known` or compile-time evaluation yet.

## Calls

The caller does not repeat `known`:

```zop
blocked_matmul a, b, tile=16
```

The call is valid only when the compiler can evaluate the supplied expression.
Passing a value read from a file, network, clock, or other runtime source is a
compile error.

`known` is part of the function type. A stored callable cannot erase a
compile-time parameter requirement. The qualifier is erased from the runtime
application binary interface because the generated function does not receive
that argument.

## Symbolic extents

A symbolic tensor extent is not automatically a compile-time value:

```zop
fn normalize values: f32[n] -> f32[n]
```

`n` states that the input and output have exactly the same extent. The
function is checked once for every valid `n`. The compiler may emit one
implementation that receives its concrete size instead of generating code for
every size.

Use a `known` parameter only when its numerical value must determine a concrete
layout, instruction, or generated control flow. Merely appearing as a symbolic
extent does not require `known`.

This separation preserves shape safety without making every tensor extent a
specialization key.

Tensor [layouts](layouts.md) follow the same staged rule. Their hierarchy and
static leaves remain compiler facts, while symbolic or runtime leaves remain
values. A fully static layout adds no runtime fields. Requiring a complete
layout as `known` is reserved for an interface whose generated code truly
depends on every leaf. Different dynamic leaf values do not force
specialization. A different static layout profile may when it changes generated
control flow.

## Evaluation

The formal feature is pure compile-time evaluation:

- Compile-time expressions use the ordinary Zop expression language.
- Every input to the expression must already be compile-time-known.
- Compile-time evaluation cannot use `Io`, runtime `Mem`, time, entropy,
  processes, or other nondeterministic state.
- A runtime value cannot convert to a compile-time value.
- A compile-time value may be embedded as an ordinary constant in generated
  code.
- Using `known` on record fields or return types is deferred.
- Compile-time reflection and arbitrary type generation are deferred.

Numeric evaluation uses the same type, quotient mode, conversion policy, and
floating-point profile as runtime evaluation. Known integer division by zero or
signed minimum divided by `-1` is a diagnostic. Literal-only `/` folds at its
contextually selected floating-point type; it is never evaluated as an
unbounded rational and rounded later. See the [numeric contract](numerics.md).

The compiler may evaluate any proven-pure constant expression automatically.
`known` exists for an interface that requires the caller to provide such a
value. It is not a constant-folding hint.

## Elaboration boundary

Compile-time evaluation consumes verified typed HIR. The lexer, parser, name
index, and signature resolver never execute arbitrary user expressions. This
keeps syntax recovery, declaration lookup, and overload diagnostics independent
from the compile-time interpreter.

Before creating a concrete instance, the compiler simplifies checked
polymorphic HIR and computes its content key. It then evaluates the required
`known` expressions, substitutes their results, and verifies the concrete HIR.
Independent keys may elaborate in parallel, but names, diagnostics, cache
entries, and artifacts remain identical to single-threaded elaboration.

The [Mojo 1.0 compiler pipeline](https://github.com/modular/modular/blob/f66d4d522c34be0a961ffac3dbfc81e30f67942e/KGEN/docs/MojoCompilerWalkthrough.md)
validates this separation with a parametric IR, pre-elaboration optimization,
and a parallel interpreter-backed elaborator. Zop retains the same boundary
while keeping symbolic tensor extents unspecialized unless generated code
requires their values.

## Specialization

A `known` value may select a tensor layout, GPU tile size, vector width,
instruction, or unrolled control-flow shape. Each distinct value may require a
separate machine-code version. The compiler caches those versions by function,
target, and compile-time arguments.

Specialization is explicit and narrow. Users cannot provide different function
bodies for selected compile-time values. Zop does not provide code
quotation, template recursion, or a fallback from failed specialization to
runtime code.

## Required tests

- Accept a literal for a `known` parameter.
- Reject a runtime expression for a `known` parameter.
- Preserve `known` through first-class function types.
- Erase compile-time parameters from the runtime calling convention.
- Reject compile-time input/output and nondeterminism.
- Reuse one cached specialization for identical arguments.
- Reject any attempt to execute user code during parsing or signature
  resolution.
- Produce identical results and diagnostics under serial and parallel
  elaboration.
- Prove symbolic extents do not require separate code versions.
- Reject unsupported compile-time reflection and type generation.
