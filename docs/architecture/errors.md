# Errors

Fallible functions return a success value or a typed error. They do not throw
exceptions or reserve a tuple position for failure.

> **Status:** The bootstrap parses failure signatures, `try to`, `fail with`,
> and patterned `catch`. Semantic error channels stop before HIR until the type
> checker and lowering exist.

## Signatures

```zop
fn boot io: Io -> App or fails with BootError
```

The function returns `App` or `BootError`. The phrase `or fails with` makes the
alternatives explicit. The semantic type is `Result[App, BootError]`.

| Signature | Contract |
| --- | --- |
| `-> T` | Infallible |
| `-> T or fails` | Infer the error type; private functions and closures only |
| `-> T or fails with E` | Declare `E`; required for exported functions |

An omitted failure clause means infallible. Adding a fallible call cannot
silently change that contract. Bare `or fails` opts into inference.

The error type is part of the function type. Interfaces, closures, and function
pointers cannot erase it.

## Error types

Errors are ordinary values of ordinary sum types:

```zop
type LoadError
    case Missing path: str
    case Invalid line: int, message: str
    case Denied path: str
```

Zop has no separate error declaration or exception hierarchy. The same `type`
and `case` grammar defines protocol states, option-like values, and domain
errors. A `catch` over a sum type must handle every reachable case.

## Error boundaries

An exported fallible function exposes one named domain error. That error may be
a sum of the failures its caller can act on. The function maps lower-level
errors at the boundary instead of leaking its dependencies.

A growing inferred error type often means that a function crosses too many
responsibilities. Split the function or define a deliberate domain boundary.
Top-level orchestration may require a broader error type.

## Construction

`fail with` exits through the current function's error channel:

```zop
fn load path: str -> Config or fails with LoadError
    if not exists path
        fail with Missing path

    parse path
```

The error value must match the declared error type. An infallible function
cannot use `fail with`. The expression terminates its branch after destroying
every live local value owned by that branch.

`fail` without `with` and `fail with` without a value are not part of the
grammar.

## Propagation

`try to` propagates a compatible error through the current function:

```zop
fn boot io: Io -> App or fails with BootError
    config = try to load io, path="zop.toml"
    return App config
```

Every propagation point is explicit. Zop has no automatic propagation or
lexical `try` region. `to` belongs to the grammar; the function remains `load`,
not `to_load` or `try_load`.

A caller may instead handle the error or preserve the complete `Result[T, E]`
as a value. Discarding a fallible result is a compile error.

## Recovery

`catch` always requires an error pattern:

```zop
config = load(io, path) catch error
    report error
    default_config
```

The pattern may bind the complete error or select one error variant. Multiple
`catch` clauses must cover the complete error type. Every handler yields the
operation's success type.

A bare `catch` is not part of the grammar. The parser cannot construct a catch
node without a pattern. Code that intentionally ignores the error must state a
wildcard pattern explicitly.

## Success values

A tuple belongs wholly to the success channel:

```zop
fn split input: str -> (str, str) or fails with ParseError
```

The function returns both strings or one `ParseError`.

## Numeric failures

Ordinary trapping arithmetic does not add an error channel. Recoverable numeric
members use ordinary typed failures:

- `Overflow` means the mathematical result does not fit the result type.
- `DivideByZero` means an integer divisor is zero.
- `InexactDivision` means exact integer division has a nonzero remainder.
- `PrecisionLoss`, `Underflow`, `InvalidConversion`, and conversion `Overflow`
  describe explicit numeric-conversion failures.

Quotient and remainder remain one success tuple when requested together; the
failure channel never occupies another tuple position. The complete operation
and conversion rules live in the [numeric contract](numerics.md).

## Device faults

A `kn` kernel has no language error channel in the first GPU contract. It cannot
declare `or fails`, and `try to` cannot propagate through the kernel boundary.
A fallible numeric or layout operation inside `kn` must be handled locally and
converted into ordinary output data, a mask, an explicit error buffer, or an
unrecoverable trap.

Host launch and completion operations are fallible runtime operations. Their
`DeviceError` sum distinguishes:

- `LaunchRejected`, which occurs before device execution and preserves existing
  values when the target can prove no work began;
- `DeviceFault`, which reports a trap or target failure after execution began
  and invalidates the complete device execution context; and
- `DeviceLost`, which rejects later access through a handle whose context is
  already invalid.

The host may catch or map `DeviceError` because it is outside the failed device
domain. Catching it does not catch the device trap, restore allocations, or make
partial outputs valid. Recovery creates a fresh context and reconstructs data
explicitly under the [`fn` and `kn` contract](gpu.md#kernel-traps-and-host-recovery).

## Lowering

High-level intermediate representation (HIR) records the success and error
types separately. Handling and propagation become explicit control flow before
the restricted Multi-Level Intermediate Representation (MLIR) boundary.

Native code uses ordinary branches and values. Language errors do not use
platform exception tables or stack unwinding. Compiler diagnostics remain
separate and stop compilation.

## Open decisions

- Cross-module application binary interface representation.

## Required tests

- Reject a discarded fallible result.
- Reject `fail with` in an infallible function.
- Reject an error value outside the declared error type.
- Construct error cases through the ordinary sum-type contract.
- Reject incomplete `fail with` forms during parsing.
- Reject propagation without a compatible error channel.
- Reject inferred error types on exported functions.
- Reject bare `catch` during parsing.
- Require exhaustive `catch` patterns with one success type.
- Preserve tuples as complete success values.
- Preserve error types through callable values and interfaces.
- Lower propagation to explicit control flow without unwinding.
- Reject a `kn` error signature and unhandled kernel-local fallible result.
- Distinguish pre-execution `LaunchRejected` from context-invalidating
  `DeviceFault` and subsequent `DeviceLost`.
