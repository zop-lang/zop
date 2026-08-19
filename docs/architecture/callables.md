# Members, calls, and callable values

Member selection and function invocation are separate operations. Functions are
values in the language model, but they do not all share one runtime
representation.

> **Status:** This page defines target grammar and lowering semantics. The
> bootstrap implements the call boundary and named-argument spelling; defaults
> and closures remain target contracts.

## Grammar

`.` selects a member. It never invokes that member. Whitespace or parentheses
start an argument list.

```zop
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

```zop
matmul left, right
matmul(left, right)
```

An unparenthesized call ends at the newline. Nested calls use parentheses or an
intermediate binding when their boundaries would otherwise be ambiguous. A
zero-argument call always retains `()`.

The parentheses around a call delimit arguments; they do not construct a
tuple. `f((x, y))` passes one tuple while `f(x, y)` passes two arguments.

Member selection binds tighter than function application:

```zop
print mem.kind
mem.alloc f32, count
```

Each argument is a complete expression. It continues through infix operators
until a comma, newline, or closing delimiter:

```zop
print x + y
(print x) + y
```

The first line prints `x + y`. The second adds `y` to the result of `print x`.
Indexing and member selection remain part of the current argument.

Nested calls require grouping when their boundary would be ambiguous:

```zop
print (mem.alloc f32, count)
```

The parser does not consult types to decide where a call begins or ends.

## Declarations

Commas also separate parameters. Short declarations omit parentheses:

```zop
fn read_config io: Io, mem: Mem, path: str -> Config or fails with LoadError
```

Multiline parameter lists require parentheses and accept a trailing comma:

```zop
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

```zop
kn blocked_matmul a: f32[m, k], b: f32[k, n], tile: known int
```

The caller supplies `tile` normally. The compiler rejects the call unless that
argument is available during compilation. `known` remains part of a stored
callable's function type. See the
[compile-time-values contract](compile-time.md).

## Named arguments

`label=value` supplies an argument by parameter name:

```zop
config = read_config io, mem, path="zop.toml"
```

`:` always states a type. `=` supplies a value in a binding, default, or named
argument. Assignment is not an expression and therefore cannot appear where a
call argument is expected.

Positional arguments precede named arguments. An argument cannot be supplied
twice. Unknown and missing labels are compile errors.

Arguments evaluate from left to right in source order. Name resolution then
records each destination parameter without reordering high-level intermediate
representation (HIR). Lowering evaluates the recorded sequence, then places the
resulting values in calling-convention order. Parameter labels are part of an
exported source contract; renaming one is a breaking change.

## Default arguments

A parameter supplies a default with `=`:

```zop
fn search query: str, limit: int = 100, exact: bool = false -> Results

all = search "zop"
few = search "zop", limit=20
exact = search("zop", limit=20, exact=true)
```

Defaulted parameters are trailing and their expressions must be pure
compile-time values. The compiler inserts an omitted default at the call site,
and the expression participates in the exported interface hash. A stored
function pointer retains the full parameter list; defaults apply only when
calling a declaration whose source contract is known.

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

```zop
mem.free buffer
```

The compiler resolves it as:

```text
Mem.free(&mem, buffer)
```

Storing the member creates a bound callable:

```zop
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
A capturing closure owns or borrows an explicit environment record. The
frontend infers the capture set and the least-powerful valid access mode.
Immutable borrowing is preferred over mutable borrowing, and borrowing is
preferred over ownership transfer.

The compiler never invents shared ownership to make a closure compile. A
closure that outlives a borrowed capture is rejected. An escaping closure that
must consume a named owner is also rejected until source gains an explicit
capture-transfer form; the compiler never hides a move in inference. Explicitly
shared values may be captured through their ordinary sharing contract.

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
- Reject `label: value` as a named argument.
- Reject positional arguments after a named argument.
- Reject unknown, missing, and duplicate argument labels.
- Insert pure compile-time defaults only for direct declaration calls.
- Reject a non-trailing or effectful default.
- Infer closure capture sets without hiding an ownership transfer.
- Resolve fields, methods, module functions, and callable fields distinctly.
- Reject a bound method that outlives its receiver.
- Prove a direct method call allocates no closure object.
- Prove known callables remain direct calls across package boundaries.
- Materialize a function pointer only for a runtime-selected function.
- Preserve ownership, effects, and target checks through call lowering.
