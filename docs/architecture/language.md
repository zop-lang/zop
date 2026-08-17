# Language

The language contract defines how a Bedrock program behaves before the
compiler chooses an intermediate representation or target.

## Goals

Bedrock is a strong systems language first. Tensor operations should feel
native enough that machine-learning frameworks grow directly from the
language. That is a core goal, not the only goal.

Performance is a premium. JAX-like purity is too, but Bedrock is not a
functional evangelist. Programs may use explicit mutation, pointers, and
effects. The compiler preserves those choices instead of pretending they are
pure.

Compilation should be as fast as possible. Source should read like executable
pseudocode, with succinct Python- and English-like word order. Researchers
should be able to move an idea from a whiteboard into a tensor program without
learning a framework-specific notation or giving up native performance.

Tensor semantics enter Multi-Level Intermediate Representation (MLIR) early.
Future graphics processing unit (GPU) work should extend target lowering
instead of introducing a second source language or framework boundary.
The aspirational [`fn` and `kn` contract](gpu.md) keeps that target choice
visible in source.

These are design goals, not implemented language guarantees.

## Static typing and inference

Every Bedrock expression has one compile-time type. Source annotations are
optional when inference determines that type uniquely. Ambiguous inference is a
compile error; the compiler never inserts a runtime-dynamic value to keep going.

| Source choice | Compiler contract |
| --- | --- |
| Inferred | Prove the fact at compile time |
| Annotated | Verify the stated fact with no runtime cost |
| Explicitly dynamic | Insert the required runtime representation and checks |

Annotations may state types, shapes, placement, effects, or view origins. They
are checked assertions, not overrides. A mismatch is a compile error.

A recoverable runtime check introduced by dynamic code appears in a typed error
channel. Safe code has no `trust me` annotation. Unchecked claims require an
explicit unsafe boundary.

Prototype and production builds use the same static semantics. Prototyping gets
its short feedback loop from inference, just-in-time (JIT) compilation,
symbolic tensor dimensions, and interactive tooling rather than a second dynamic
language mode.

A future runtime-dynamic value must use an explicit type. Ordinary bindings do
not change type after initialization.

Editors may display inferred facts as inlay hints. A programmer can promote an
inferred fact into source when it should become a durable contract.

## Numeric literals

An unsuffixed numeric literal has no concrete machine type until its immediate
context supplies one. Function results, call parameters, existing bindings,
and typed arithmetic operands provide that context:

```bedrock
fn add_one value: f32 -> f32
    value + 1
```

The literal `1` becomes `f32`. No conversion occurs for `value`, which already
has a concrete type. The same rule applies through an arithmetic expression
made only from numeric literals.

Only literal expressions are contextual. Named values never promote:

```bedrock
fn add left: f32, right: i32 -> f32
    left + right  # compile error
```

A new binding ends contextual inference. Without an expected type, integer
literals default to `i64` and floating-point literals default to `f64`:

```bedrock
count = 1    # i64
rate = 0.5  # f64
```

Later uses cannot reinterpret either binding. Mixed literal kinds without an
expected type are also an error; `1 + 2.0` does not invent a promotion rule.

Literal conversion is fail-closed:

- An integer literal must fit an integer context.
- An integer literal entering a floating-point context must be exactly
  representable.
- A floating-point literal may round to the selected floating-point precision.
- A floating-point literal must parse as a finite `f64`. Narrowing to `f32`
  rejects overflow and a nonzero value that would become zero.
- Literal values never select an operation's result type.
- Boolean and numeric literals do not convert between one another.

The parser retains enough precision to represent the magnitude of every signed
`i64` literal. Unary negation therefore accepts `-9223372036854775808` while
rejecting an out-of-range positive value. Typed HIR contains the final concrete
literal type before lowering begins.

## Tensors and structured values

A tensor is Bedrock's homogeneous array type, generalized to any number of
dimensions:

```bedrock
vector = [1, 2, 3]
matrix = [[1, 2], [3, 4]]
```

Square brackets hold tensor data. A tensor literal must be rectangular, and
its elements must resolve to one compatible type. The element type and rank are
always static. Every dimension has a concrete size before the tensor is
created, and that shape remains fixed for the tensor's lifetime.

Dimensions may be integer literals, named constants, or symbolic parameters:

```bedrock
fn transpose value: f32[m, n] -> f32[n, m]
```

Repeated dimension names require exact equality. A symbolic dimension records
a type relationship; it does not require a separate machine-code version for
every size. Unknown element types, unknown rank, resizable tensors, ragged
tensor literals, and data-dependent output shapes are not part of the first
tensor contract.

Parentheses group a fixed number of values and may mix types. Records provide
the same structural role with names:

```bedrock
pair = (weights, bias)
point = Point(x: 1, y: 2)
```

Tuples and records organize values. They are not tensors and do not have a
single element type, shape, placement, or tensor operation. Compiler
transformations may flatten their tensor leaves internally and reconstruct the
same source structure afterward.

## Compile-time values

`name: known Type` requires an argument to be available during compilation:

```bedrock
kn blocked_matmul a: f32[m, k], b: f32[k, n], tile: known int
```

`known` qualifies when the value exists, not whether it is mutable. It is part
of the function type and disappears from the runtime calling convention.
Symbolic dimensions are not `known` unless an interface explicitly requires
their numerical values during compilation. See the
[compile-time-values contract](compile-time.md).

## Purity and effects

Purity is a checked compiler property, not a language-wide ideology. Calls and
operations are effectful unless the compiler can prove the narrower contract.
Only proven-pure work may be discarded, duplicated, reordered, or evaluated at
compile time.

This rule gives pure tensor code the transformations associated with JAX while
keeping systems code honest about mutation and input or output.

## Mutation

`mut` is an access mode, not a type or local-binding qualifier. It grants an
exclusive writable borrow across a function or view boundary.

A uniquely owned local value may be reassigned or updated without a `mut`
declaration. Its type cannot change. The compiler versions local assignments in
high-level intermediate representation (HIR) and may convert them to static
single assignment form.

Local mutation does not make a function effectful when no caller can observe
it. Mutation through a borrowed value requires `mut` in the boundary contract.

## First executable subset

The executable bootstrap supports typed `i64` functions, calls, arithmetic,
local assignments, and returns. It excludes branches, dynamic values, closures,
pointers, and tensors. Control flow comes next. Fixed-rank tensors and one
matrix multiplication follow it.

Keeping the first subset small lets the same program prove parsing, typing,
MLIR lowering, Cranelift intermediate representation (CLIF) translation, JIT
execution, and ahead-of-time (AOT) linking.

## Memory

Bedrock uses checked single ownership, borrowing, explicit transfer, and
deterministic destruction. Functions that request storage receive an explicit `Mem`
capability. The core language does not require garbage collection. See the
[memory-management contract](memory.md).

## Tensor transformations

Automatic differentiation is an explicit compiler transformation that returns
owned gradients. Backend and autodiff capabilities do not appear as tensor
generic parameters. See the [automatic-differentiation contract](autodiff.md).

## Input, output, and nondeterminism

Functions require an explicit `Io` capability to access files, networks, time,
entropy, processes, blocking synchronization, or asynchronous work. Pure
functions and GPU kernels cannot use host input/output. See the
[explicit input/output contract](io.md).

## Members and calls

`.` selects a member and never invokes it. Whitespace or parentheses invoke a
callable value. Commas separate multiple arguments and parameters. Named
arguments use `label: value`. Functions, bound methods, closures, and dynamic
callables keep distinct runtime representations. See the
[callables contract](callables.md).

## Grammar and names

Bedrock uses controlled English: fixed word order with little punctuation, not
free-form prose. Action functions use base-form verb phrases such as `load`,
`read_config`, and `compile`. Names do not encode tense or grammar with forms
such as `to_load`, `try_load`, or `loaded_config`.

Pseudocode-like syntax must remain predictable. The parser rejects ambiguity
instead of guessing what the author meant.

Grammar supplies connecting words. `try to load` uses the function `load`;
`to` belongs to the propagation syntax. This keeps a function's name stable in
direct calls, methods, and stored callable values.

## Layout

Indentation is the only block delimiter. A block starts when the next logical
line is indented and ends at a matching dedent. A dedent to a column that did
not open an enclosing block is a compile error.

Leading indentation uses spaces. Tabs are rejected there. The formatter uses
four spaces per level.

A newline ends an expression unless it occurs inside an explicit delimiter.
Blank lines do not affect layout. Backslashes and trailing operators do not
continue a line.

Bedrock has no semicolons or brace-delimited blocks. Braces remain available
for future data syntax.

## Blocks and returns

Every block is an expression. Its final expression is the block's value:

```bedrock
fn square x: f32 -> f32
    x * x
```

A function body must yield its declared success type on every successful path.
`return` exits the function explicitly and remains valid anywhere in the body,
including the final expression. It is optional when the body already yields the
required value.

Branches used as values must yield compatible types. This rule applies to
`if`, pattern matching, and error recovery.

## Future: proper tail calls

Bedrock intends to guarantee proper tail calls for CPU `fn` code. A call in
tail position will use bounded stack space, including self-recursive, mutually
recursive, direct, and indirect calls with compatible function types.

A call is in tail position when the function returns the callee's complete
success-or-error result unchanged after destroying its local values. The
compiler detects this position without special syntax. Non-tail recursion may
grow the stack.

GPU `kn` recursion remains target-specific and is not covered by this
guarantee.

Proper tail calls are not part of the first executable subset or initial native
application binary interface.

## Errors

`-> T or fails with E` declares a typed error channel. Exported functions name
`E`; private functions and closures may request inference with bare
`or fails`. Every fallible result must be handled, propagated with `try to`, or
preserved as a value. `fail with` produces an error. Local recovery uses
`catch pattern`; bare `catch` is not valid syntax. See the [error
contract](errors.md).

## Open decisions

The frontend must not guess these semantics:

- The exact syntax for view-origin annotations and unsafe pointers.
- How functions capture values.
- Default argument semantics.
- Explicit numeric cast syntax.
- Broadcasting, indexing, slicing, and bounds behavior.
- Tensor layout at raw-pointer and foreign-function boundaries.
- How functions declare or infer purity and other effects.

Resolve each decision in this page before code depends on it.
