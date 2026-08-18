# Errors

Fallible functions return a success value or a typed error. They do not throw
exceptions or reserve a tuple position for failure.

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
    config = try to load io, path: "zop.toml"
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

## Lowering

High-level intermediate representation (HIR) records the success and error
types separately. Handling and propagation become explicit control flow before
the restricted Multi-Level Intermediate Representation (MLIR) boundary.

Native code uses ordinary branches and values. Language errors do not use
platform exception tables or stack unwinding. Compiler diagnostics remain
separate and stop compilation.

## Open decisions

- Error declaration, payload, and sum syntax.
- Device-kernel failures.
- Cross-module application binary interface representation.

## Required tests

- Reject a discarded fallible result.
- Reject `fail with` in an infallible function.
- Reject an error value outside the declared error type.
- Reject incomplete `fail with` forms during parsing.
- Reject propagation without a compatible error channel.
- Reject inferred error types on exported functions.
- Reject bare `catch` during parsing.
- Require exhaustive `catch` patterns with one success type.
- Preserve tuples as complete success values.
- Preserve error types through callable values and interfaces.
- Lower propagation to explicit control flow without unwinding.
