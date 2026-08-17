# Members, calls, and callable values

Member selection and function invocation are separate operations. Functions are
values in the language model, but they do not all share one runtime
representation.

> **Status:** This page defines target grammar and lowering semantics. Exact
> closure syntax remains open.

## Grammar

`.` selects a member. It never invokes that member. Whitespace or parentheses
start an argument list.

```bedrock
mem.kind
mem.alloc f32, count
mem.alloc(f32, count)
mem.flush()
```

A call with multiple arguments requires commas. A trailing comma is valid, but
the formatter removes it from a single-line call and requires it in a multiline
call. Whitespace separates the callable from its first argument; no comma or
colon follows the callable. A zero-argument call always requires `()`.

Parentheses are optional for a single-line call and required for a multiline
call. Both forms have the same semantics:

```bedrock
matmul left, right
matmul(left, right)
```

The parentheses around a call delimit arguments; they do not construct a
tuple. `f((x, y))` passes one tuple while `f(x, y)` passes two arguments.

Member selection binds tighter than function application:

```bedrock
print mem.kind
mem.alloc f32, count
```

Each argument is a complete expression. It continues through infix operators
until a comma, newline, or closing delimiter:

```bedrock
print x + y
(print x) + y
```

The first line prints `x + y`. The second adds `y` to the result of `print x`.
Indexing and member selection remain part of the current argument.

Nested calls require grouping when their boundary would be ambiguous:

```bedrock
print (mem.alloc f32, count)
```

The parser does not consult types to decide where a call begins or ends.

## Declarations

Commas also separate parameters. Short declarations omit parentheses:

```bedrock
fn read_config io: Io, mem: Mem, path: str -> Config or fails with LoadError
```

Multiline parameter lists require parentheses and accept a trailing comma:

```bedrock
fn read_config(
    io: Io,
    mem: Mem,
    path: str,
) -> Config or fails with LoadError
```

Every parameter list accepts a trailing comma. The formatter removes it from a
single-line declaration and requires it in a multiline declaration. No comma
follows the function name. `->` introduces the success type.

`known` follows the colon because it qualifies when a parameter value becomes
available:

```bedrock
kn blocked_matmul a: f32[m, k], b: f32[k, n], tile: known int
```

The caller supplies `tile` normally. The compiler rejects the call unless that
argument is available during compilation. `known` remains part of a stored
callable's function type. See the
[compile-time-values contract](compile-time.md).

## Named arguments

`label: value` supplies an argument by parameter name:

```bedrock
config = read_config io, mem, path: "bedrock.toml"
```

Positional arguments precede named arguments. An argument cannot be supplied
twice. Unknown and missing labels are compile errors.

Arguments evaluate from left to right in source order. Name resolution then
orders them for high-level intermediate representation (HIR) and backend
lowering. Parameter labels are part of an exported source contract; renaming
one is a breaking change. Default arguments remain a separate language
decision.

## Resolution

The parser emits only `Member` and `Call` expressions:

```text
mem.free          -> Member(mem, free)
mem.free buffer   -> Call(Member(mem, free), [buffer])
mem.free()        -> Call(Member(mem, free), [])
```

Name and type resolution later determine whether a member is a field, method,
module function, or field whose value is callable. A type cannot define a field
and method with the same member name.

Fields never auto-invoke. A computed getter is an ordinary function and uses
call syntax. This prevents field access from hiding effects, errors, or latency.

## Callable forms

The frontend keeps these forms distinct:

| Form | Meaning | Typical lowering |
| --- | --- | --- |
| Function item | Known declaration | Direct symbol call |
| Function pointer | Runtime-selected function | Code pointer |
| Bound method | Function plus receiver borrow | Pair or direct call |
| Closure | Function plus captured values | Code and environment |
| Dynamic callable | Erased implementation | Data and method table |

They satisfy one callable type contract without sharing one machine layout.

## Bound methods

A direct method call needs no bound-method object:

```bedrock
mem.free buffer
```

The compiler resolves it as:

```text
Mem.free(&mem, buffer)
```

Storing the member creates a bound callable:

```bedrock
release = mem.free
release buffer
```

Conceptually, `release` contains the function and a borrow of `mem`:

```text
BoundMethod {
  function: Mem.free
  receiver: &mem
}
```

The bound method cannot outlive `mem`. This representation does not require a
heap allocation. When every use is visible, the compiler may eliminate the
pair and emit a direct call.

## Closures

A non-capturing closure can lower to a known function item or function pointer.
A capturing closure owns or borrows an explicit environment record. Capture
mode follows the [memory-management contract](memory.md).

The compiler never invents shared ownership to make a closure compile. A
closure that outlives a borrowed capture is rejected unless the program moves
or explicitly shares that value.

## HIR contract

High-level intermediate representation (HIR) preserves the callable form until
runtime representation is necessary:

```text
FunctionItem
FunctionPointer
BoundMethod { function, receiver, access }
Closure { function, environment, captures }
DynamicCallable { data, methods }
```

Every call records argument labels, source evaluation order, ownership modes,
binding times, effects, target domain, return type, error type, and source
location. Method desugaring happens after member resolution, not during
parsing.

## Backend contract

Known function items and methods lower to direct calls and remain eligible for
inlining. The backend materializes a code pointer, receiver pair, closure
environment, or method table only when runtime selection requires it.

Cross-module metadata keeps small callable bodies and effect summaries visible
to release optimization. Package boundaries must not turn a statically known
hot path into indirect dispatch.

A CPU function pointer cannot be called from a GPU kernel. Device-callable
helpers require a device specialization proven legal by the [`fn` and `kn`
contract](gpu.md).

## `Mem` release

Safe owned storage normally releases through deterministic destruction or an
explicit `drop`. `mem.free` is primarily a low-level operation for raw storage
or explicit early release. It follows the same member and callable rules as any
other function.

## Required tests

- Parse member selection without invocation.
- Require `()` for zero-argument calls.
- Require commas between multiple arguments and parameters.
- Accept trailing commas without changing arity.
- Prove member selection binds tighter than application.
- Keep infix operators inside the current argument.
- Reject ambiguous nested calls without grouping.
- Resolve named arguments and preserve source evaluation order.
- Reject positional arguments after a named argument.
- Reject unknown, missing, and duplicate argument labels.
- Resolve fields, methods, module functions, and callable fields distinctly.
- Reject a bound method that outlives its receiver.
- Prove a direct method call allocates no closure object.
- Prove known callables remain direct calls across package boundaries.
- Materialize a function pointer only for a runtime-selected function.
- Preserve ownership, effects, and target checks through call lowering.
