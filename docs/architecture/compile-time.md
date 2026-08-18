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

## Symbolic dimensions

A symbolic tensor dimension is not automatically a compile-time value:

```zop
fn normalize values: f32[n] -> f32[n]
```

`n` states that the input and output have exactly the same dimension. The
function is checked once for every valid `n`. The compiler may emit one
implementation that receives its concrete size instead of generating code for
every size.

Use a `known` parameter only when its numerical value must determine a concrete
layout, instruction, or generated control flow. Merely appearing as a symbolic
dimension does not require `known`.

This separation preserves shape safety without making every tensor dimension a
specialization key.

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

The compiler may evaluate any proven-pure constant expression automatically.
`known` exists for an interface that requires the caller to provide such a
value. It is not a constant-folding hint.

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
- Prove symbolic dimensions do not require separate code versions.
- Reject unsupported compile-time reflection and type generation.
