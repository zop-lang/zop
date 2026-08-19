# Language

The language contract defines how a Zop program behaves before the
compiler chooses an intermediate representation or target.

Zop takes its name from `Z`, the alphabet's final letter: it aims to be the
last language its users need. People who write Zop are Zoppers.

## Goals

Zop is a strong systems language first. Tensor operations should feel
native enough that machine-learning frameworks grow directly from the
language. That is a core goal, not the only goal.

Performance is a premium. JAX-like purity is too, but Zop is not a
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
visible in source. Tensor contractions retain enough structure to target
vector, matrix, and systolic hardware without changing the source program.

These are design goals, not implemented language guarantees.

## Static typing and bounded inference

Every Zop expression has one compile-time type. Source annotations are
optional for local expressions and bindings when inference determines that type
uniquely. Ambiguous inference is a compile error; the compiler never inserts a
runtime-dynamic value to keep going.

Zop does not use unrestricted Hindley-Milner inference. Its model is local,
bidirectional, and unification-based: an expression either synthesizes a type
from its syntax or is checked against an expected type supplied by its context.
Shape, placement, effects, ownership, and error channels add separate checked
constraints.

The annotation boundary is deliberate:

- Named function parameters always state their types and ownership modes.
- Exported functions state parameter, success, and error contracts. Capability
  parameters and checked purity state their public effects.
- A private non-recursive function may infer its return type when unique.
- A recursive function states its return type before checking its body.
- A closure parameter may infer from an expected callable type; otherwise it is
  annotated.
- Local bindings and intermediate expressions infer whenever the result is
  unique.
- User-defined generics are deferred until concrete duplication proves their
  required shape. When introduced, declarations will be explicit and call-site
  arguments may infer.

Local bindings are monomorphic unless source explicitly declares a generic.
The compiler does not automatically generalize a local function into a hidden
polymorphic value. This keeps compilation local and prevents later uses from
silently changing an earlier binding's contract.

| Source choice | Compiler contract |
| --- | --- |
| Inferred | Prove the fact at compile time |
| Annotated | Verify the stated fact with no runtime cost |

Annotations may state types, shapes, placement, effects, or view origins. They
are checked assertions, not overrides. A mismatch is a compile error.

Safe code has no `trust me` annotation. Unchecked claims require an explicit
unsafe boundary.

Prototype and production builds use the same static semantics. Prototyping gets
its short feedback loop from inference, just-in-time (JIT) compilation,
symbolic tensor extents, and interactive tooling rather than a second dynamic
language mode. Runtime-dynamic typing is neither supported nor promised. A
future proposal would need an explicit type and could not change the type of an
ordinary binding, but Zop reserves no syntax for it now.

Editors may display inferred facts as inlay hints. A programmer can promote an
inferred fact into source when it should become a durable contract.

## Numeric literals

An unsuffixed numeric literal has no concrete machine type until its immediate
context supplies one. Function results, call parameters, existing bindings,
and typed arithmetic operands provide that context:

```zop
fn add_one value: f32 -> f32
    value + 1
```

The literal `1` becomes `f32`. No conversion occurs for `value`, which already
has a concrete type. The same rule applies through an arithmetic expression
made only from numeric literals.

Fractional `/` supplies a floating-point context. Integer `//` and `%` supply
an integer context:

```zop
half = 1 / 2        # f64
half: f32 = 1 / 2   # f32
quotient = 7 // 2   # i64
```

Only literal expressions are contextual. Named values never promote:

```zop
fn add left: f32, right: i32 -> f32
    left + right  # compile error
```

A new binding ends contextual inference. Without an expected type, integer
literals and integer-only expressions default to `i64`. Floating-point
literals and literal-only fractional division default to `f64`:

```zop
count = 1    # i64
rate = 0.5  # f64
```

Later uses cannot reinterpret either binding. Mixed literal kinds without an
expected type are also an error; `1 + 2.0` does not invent a promotion rule.
`1 / 2` is not an exception to that rule: both operands are still literals when
`/` requires one floating-point type.

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

The complete [numeric contract](numerics.md) defines division, rounding, casts,
floating-point profiles, and target behavior.

## Integer arithmetic

Ordinary fixed-width integer arithmetic requires every mathematical result to
fit the result type for signed and unsigned values. Addition, subtraction,
multiplication, and negation trap when the result does not fit. Integer `//`
uses floor division and `%` is its paired modulo. They trap on a zero divisor
and on the minimum signed value divided by `-1`. Integer operands do not support
fractional `/`. A shift count outside the left operand's bit width also traps.

The rule is identical in development, release, CPU, GPU, browser, just-in-time,
and ahead-of-time builds. Optimization may remove a representability test only
after proving it cannot fail. `unsafe` does not disable arithmetic traps.

A compile-time-known invalid operation is a compile error. A runtime trap is
unrecoverable and does not silently add an error channel to the containing
function. Code that needs another contract selects it explicitly:

```zop
sum = left + right
wrapped = left.wrapping_add right
clamped = left.saturating_add right
recoverable = try to left.add right
quotient = left // right
ratio = 1 / 2
```

`wrapping_add` reduces modulo the type width. `saturating_add` clamps to the
nearest bound. The integer member `add` returns the exact result or fails with
`Overflow`; `try to` propagates that typed error. `add` performs the
representability test, while `try to` only handles its failure channel.
Subtraction, multiplication, negation, and shifts provide matching members
where applicable. Recoverable integer division requires an explicit rounding
mode; exact division additionally reports `InexactDivision`. See the
[numeric contract](numerics.md#recoverable-integer-division).

These members belong to fixed-width integer types. They are ordinary callable
members, not keywords or prelude functions. The compiler recognizes their
primitive implementations directly and emits no library call. Modules and
user-defined types remain free to declare `add`, `multiply`, or any other
arithmetic name. Ordinary operators do not desugar to these members:
`left + right` traps on overflow, while `left.add right` has a typed failure
channel.

Integer tensor operations use the same element semantics. A target that cannot
preserve the trap contract rejects the operation instead of wrapping. Floating
point follows the selected profile in the [numeric contract](numerics.md) and is
not covered by integer representability tests.

Scalar `min` and `max` live in the bundled `math` module rather than the
prelude:

```zop
from math import min, max

lower = min left, right
upper = max left, right, limit
minimums = values.min axis=0, order=Tree
```

They preserve concrete type, propagate floating NaN, distinguish signed zero,
and never control slice clipping. The public import path does not expose the
standard library's internal core layer. See the
[minimum and maximum contract](numerics.md#minimum-and-maximum).

## Type declarations

`type` is the only keyword that introduces a type. The declaration form states
which kind of type it defines:

```zop
type Point
    x: f32
    y: f32

type Shape
    case Circle radius: f32
    case Rectangle width: f32, height: f32
    case Empty

type Ordered
    fn compare self, other: Self -> Order

type UserId = u64
type FileId distinct u64
```

A field body defines a nominal product type. A `case` body defines a nominal
sum type whose value contains exactly one case. A required-function body
defines a behavioral contract for generic constraints. `type Name = Existing`
is a transparent alias. `type Name distinct Existing` defines a
layout-compatible nominal wrapper that requires explicit conversion.

Constructors are ordinary calls, so call punctuation stays uniform:

```zop
point = Point x=3, y=4
shape = Shape.Circle radius=10
```

Zop has no separate `struct`, `enum`, `record`, `interface`, `trait`, `alias`,
or error-declaration keyword. This means every source type declaration begins
with `type`; it does not make types ordinary runtime values or introduce
dependent typing.

## Tensors and structured values

A tensor is Zop's homogeneous array type, generalized to any number of axes:

```zop
vector = [1, 2, 3]
matrix = [[1, 2], [3, 4]]
```

Square brackets hold tensor data. A tensor literal must be rectangular, and
its elements must resolve to one compatible type. The element type and rank are
always static. Every axis has a concrete extent before the tensor is
created, and that shape remains fixed for the tensor's lifetime.

Compatibility does not invent numeric promotion. Named elements must already
have one element type; numeric literals may adopt that type through contextual
typing. Heterogeneous values use tuples or product types rather than tensors.

Every tensor is an inspectable `Engine` paired with an inspectable `Layout`,
using CUTLASS CuTe's canonical model. Engine supplies the offsettable and
dereferenceable data iterator; Layout maps logical coordinates to Engine
indices. Dense literals use a right-major Layout whose final axis has unit
Stride:

```zop
matrix = [[1, 2, 3], [4, 5, 6]]

matrix.layout.shape   # (2, 3)
matrix.layout.stride  # (3, 1)
matrix.layout(1, 2)   # 5
matrix.engine         # owned host ArrayEngine[i64]
```

Element type and logical shape determine tensor type identity. Physical layout
is an observable property of each value rather than a tensor generic parameter.
See the [tensor-layout contract](layouts.md).
The [worked examples](layout-examples.md) show the same source model from dense
literals through thread/value layouts and tensor contractions.

Logical size uses precise, non-overlapping queries:

```zop
matrix.rank                 # 2
matrix.shape                # (2, 3)
matrix.extent axis=0        # 2
matrix.numel()              # 6
```

Zop does not provide an ambiguous tensor `.length`, `.len()`, `.size`, or
`.count()` alias. `extent` answers one-axis size, `numel()` answers logical
element count, and `layout.cosize` answers the scalar codomain size when it is
defined. All are derived from the canonical shape and layout rather than stored
as duplicate mutable fields.

Elementwise operators use PyTorch-compatible trailing-axis broadcasting.
Logical directions are axes. Broadcasting creates an allocation-free
zero-stride view and follows the
[axis, mode, and expansion contract](layouts.md#broadcasting-and-expansion).

Integer indexing, negative indexing, half-open slices, rank reduction, bounds
failure, and residual views follow the [indexing and slicing
contract](indexing.md). Basic indexing never allocates or copies elements;
data-dependent gathers, masks, and scatters are named library operations.

Extents may be integer literals, named constants, or symbolic parameters:

```zop
fn transpose value: f32[m, n] -> f32[n, m]
```

Repeated extent names require exact equality. A symbolic extent records
a type relationship; it does not require a separate machine-code version for
every size. Unknown element types, unknown rank, resizable tensors, ragged
tensor literals, and data-dependent output shapes are not part of the first
tensor contract.

Parentheses group a fixed number of values and may mix types. Product types
provide the same structural role with names:

```zop
pair = (weights, bias)
point = Point x=1, y=2
```

Tuples and product values organize data. They are not tensors and do not have a
single element type, shape, placement, or tensor operation. Compiler
transformations may flatten their tensor leaves internally and reconstruct the
same source structure afterward.

Tuple nesting remains observable in source. Internal application binary
interface flattening never turns `(a, (b, c))` into `(a, b, c)`.

## Patterns and destructuring

Assignment and iteration accept structural patterns. A top-level tuple pattern
does not require outer parentheses because `=` or `in` already delimits it:

```zop
left, right = pair

weights, Stats(mean=mean, variance=variance) = parameters

for name, score in rows
    print name, score
```

The same patterns may be parenthesized for grouping, but the formatter removes
redundant outer parentheses on an assignment:

```zop
(left, right) = pair
# Formats as: left, right = pair
```

Nested tuples retain parentheses, and product patterns name their fields:

```zop
head, (left, right) = nested
Point(x=x, y=y) = point
```

A plain assignment requires an irrefutable pattern, one that must match its
value's static type. A sum-type case is refutable and therefore belongs in
pattern matching or `catch`, not an unchecked assignment. Tensor values do not
implicitly destructure into elements or axis views; source uses indexing and
slicing so symbolic extents never become an implicit number of bindings.

## Iteration and zip

A loop over several iterables uses the ordinary `zip` callable explicitly:

```zop
for left, right in zip(lefts, rights)
    consume left, right
```

`zip` follows Python's named strictness model. The compile-time `strict`
argument defaults to `false`, so the iterator stops with the shortest input:

```zop
for left, right in zip(lefts, rights, strict=false)
    consume left, right
```

Code that asserts equal exhaustion opts in visibly:

```zop
for left, right in zip(lefts, rights, strict=true)
    consume left, right
```

With `strict=true`, a statically known length mismatch is a compile error. A
dynamic mismatch traps when one iterator ends before another. The trap treats
equal length as a program invariant and avoids adding a hidden error channel to
every `for` loop. Code receiving recoverable untrusted lengths validates them
before iteration.

Zop has no auto-zipped `for left, right in lefts, rights` form. The comma after
`in` would hide both construction of tuple elements and the unequal-length
policy.

Iterable `zip` is unrelated to CuTe
[`Layout.zipped_divide`](layouts.md#algebra), which groups tile modes and
remainder modes inside one hierarchical layout.

## Semantic iteration operations

`core.algorithm` exposes compiler-known `find_first`, `any`, `all`, `count`,
`map`, `reduce`, and `scan` over allocation-free views. They preserve ordinary
types and logical iteration order while giving HIR a stable operation to lower
through scalar, fixed-width SIMD, parallel CPU, or device schedules when
semantics permit. An arbitrary loop remains legal but does not carry a
vectorization guarantee.

`find_first` is initially one-dimensional and returns `Option[int]` relative to
the input view. `count` evaluates a predicate; it is unrelated to tensor
`.numel()`. Data-dependent filtering remains an allocating `std.tensor`
operation. The complete order, legality, and backend contract lives in
[SIMD and vectorization](simd.md).

## Reductions and accumulation

Reductions are named tensor or iterator operations rather than source syntax:

```zop
total = values.sum axis=0, order=Source

custom = values.reduce(
    identity=0,
    combine=add,
    order=Tree,
)
```

`Source` preserves logical iteration order. `Tree` explicitly permits a tree
reduction suitable for vector, CPU-parallel, or GPU execution. This distinction
is observable for floating-point rounding and for trapping fixed-width integer
intermediates, so a backend never changes it silently.

Sequential algorithms use ordinary update assignment:

```zop
total = 0

for value in values
    total += value
```

Zop has no `<-` accumulation operator. An explicit reduction states scheduling
policy; `+=` already states sequential local mutation.

## Generics

The bootstrap does not expose user-defined generics. The target syntax uses
square-bracket type parameters and one `where` constraint form. Symbolic tensor
extents express shape relationships and are not general type parameters.

Zop will add generics when the same useful algorithm or data structure must be
implemented repeatedly with only its types changed. The first expected users
are core collections, callable abstractions, and the self-hosted compiler. The
design must infer type arguments at ordinary call sites, preserve separate
compilation, and keep compile cost measurable.

The initial generic system will not include template metaprogramming,
type-level reflection, user specialization rules, or arbitrary compile-time
code generation. See the complete [generics contract](generics.md).

## Compile-time values

`name: known Type` requires an argument to be available during compilation:

```zop
kn blocked_matmul a: f32[m, k], b: f32[k, n], tile: known int
```

`known` qualifies when the value exists, not whether it is mutable. It is part
of the function type and disappears from the runtime calling convention.
Symbolic extents are not `known` unless an interface explicitly requires
their numerical values during compilation. See the
[compile-time-values contract](compile-time.md).

## Purity and effects

Purity is a checked compiler property, not a language-wide ideology. Calls and
operations are effectful unless the compiler can prove the narrower contract.
Only proven-pure work may be discarded, duplicated, reordered, or evaluated at
compile time.

This rule gives pure tensor code the transformations associated with JAX while
keeping systems code honest about mutation and input or output.

`Io` and `Mem` parameters make authority visible in the function type. The
frontend infers the complete effect set in HIR from capabilities, mutation,
failure, suspension, target calls, and unsafe operations. It rejects an
operation when the caller lacks the required authority.

`pure fn` is an optional checked promise for APIs whose lack of observable
effects is part of their contract. `unsafe fn` states an obligation that every
caller must uphold. Zop does not duplicate the inferred effect set in a general
source-level effect list.

## Mutation

`mut` is an access mode, not a type or local-binding qualifier. It grants an
exclusive writable borrow across a function or view boundary.

A uniquely owned local value may be reassigned or updated without a `mut`
declaration. Its type cannot change. The compiler versions local assignments in
high-level intermediate representation (HIR) and may convert them to static
single assignment form.

Arithmetic update assignments use `+=`, `-=`, `*=`, `/=`, `//=`, and `%=`.
`/=` requires floating-point operands; `//=` and `%=` require integers. They
evaluate their target once and apply the corresponding ordinary operator. A
scalar result is computed before the write. A tensor update may write elements
before one lane traps, but the trap terminates or invalidates the complete
execution domain, so no continuing safe code observes partial state. Zop does
not insert rollback storage or a preflight pass. Recoverable in-place tensor
arithmetic requires a separate explicit unchanged-on-error or partial-state
contract and is not part of core. Assignment does not produce a value. See the
[numeric](numerics.md#tensors-and-vectors) and
[runtime](runtime.md#traps-and-execution-domains) contracts.

Zop has no `++`, `--`, prefix-increment, or postfix-increment forms;
`value += 1` and `value -= 1` state those updates. `=-` is not an operator:
`value =- amount` is formatted as `value = -amount`.

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

Zop uses checked single ownership, borrowing, explicit transfer, and
deterministic destruction. Functions that request storage receive an explicit `Mem`
capability. The core language does not require garbage collection. See the
[memory-management contract](memory.md).

## Framework transformations

Automatic differentiation, gradient tapes, batching, quantization, and similar
model strategies are not language features. Zop reserves no syntax, hidden
tensor fields, or private compiler intrinsic for them. Framework packages may
implement these strategies over ordinary typed tensors, layouts, functions,
and explicit effects.

## Input, output, and nondeterminism

Functions require an explicit `Io` capability to access files, networks, time,
entropy, processes, blocking synchronization, or asynchronous work. Pure
functions and GPU kernels cannot use host input/output. See the
[explicit input/output contract](io.md).

Tensor-aware formatting is canonical and compiler-known, but `print` remains a
fallible standard input/output function. It requires `Io`, never performs an
implicit device transfer, and follows the [tensor-formatting
contract](layouts.md#tensor-formatting).

## Members and calls

`.` selects a member and never invokes it. Whitespace or parentheses invoke a
callable value. Commas separate multiple arguments and parameters. Named
arguments use `label=value`. `:` states a type; `=` supplies a value. Functions,
bound methods, closures, and dynamic callables keep distinct runtime
representations. See the
[callables contract](callables.md).

## Grammar and names

Zop uses controlled English: fixed word order with little punctuation, not
free-form prose. Action functions use base-form verb phrases such as `load`,
`read_config`, and `compile`. Names do not encode tense or grammar with forms
such as `to_load`, `try_load`, or `loaded_config`.

Pseudocode-like syntax must remain predictable. The parser rejects ambiguity
instead of guessing what the author meant.

Grammar supplies connecting words. `try to load` uses the function `load`;
`to` belongs to the propagation syntax. This keeps a function's name stable in
direct calls, methods, and stored callable values.

## Strings

Double quotes create one-line interpreted strings. Triple quotes create
multiline interpreted strings:

```zop
message = """
    first line
    newline escape: \n
    """
```

The opening and closing delimiter lines are not part of the value. The closing
delimiter's indentation defines the margin removed from every nonblank content
line. Relative indentation and internal blank lines remain. Backslash escapes
use the same contract as a one-line string.

The opening delimiter must be followed by a physical newline, and the closing
delimiter must be the only non-comment token on its line. Every nonblank content
line must contain the closing margin; a line that escapes that indentation is a
compile error rather than a reason to guess another margin. Tabs after the
margin are string data, not source indentation.

`raw"""` creates multiline content without escape processing:

```zop
pattern = raw"""
    ^\d+\s+\w+$
    """
```

The result contains the backslashes exactly as written after indentation
normalization. `#`, `##`, and documentation tags inside either string are data,
not comments. Triple-quoted strings never become documentation because of their
position; documentation uses `##` exclusively.

The initial string contract has no interpolation. Formatting remains an
explicit operation so embedded research text, regular expressions, shader
source, and browser templates do not acquire a second expression grammar.

## Layout

Indentation is the only block delimiter. A block starts when the next logical
line is indented and ends at a matching dedent. A dedent to a column that did
not open an enclosing block is a compile error.

Leading indentation uses spaces. Tabs are rejected there. The formatter uses
four spaces per level.

A newline ends an expression unless it occurs inside an explicit delimiter.
Blank lines do not affect layout. Backslashes and trailing operators do not
continue a line.

Zop has no semicolons or brace-delimited blocks.

`#` starts an ordinary line comment outside a string. `##` at the first
non-whitespace position starts structured documentation for the following
declaration. Both extend through the physical newline, and comment-only lines
do not open or close an indentation block.

Several consecutive lines form multiline commentary. Zop has no block-comment
or docstring-as-comment form. Documentation uses compiler-checked `@param`,
`@returns`, `@fails`, `@example`, and related tags instead of recovering
structure from prose. See the complete [documentation
contract](documentation.md).

## Blocks and returns

Every block is an expression. Its final expression is the block's value:

```zop
fn square x: f32 -> f32
    x * x
```

A function body must yield its declared success type on every successful path.
`return` exits the function explicitly and remains valid anywhere in the body,
including the final expression. It is optional when the body already yields the
required value.

Branches used as values must yield compatible types. This rule applies to
`if`, pattern matching, and error recovery.

## Proper tail calls

Zop guarantees proper tail calls for CPU `fn` code once the systems-core
milestone implements the required calling convention. A call in tail position
uses bounded stack space, including self-recursive, mutually recursive, direct,
and indirect calls with compatible function types. The guarantee never depends
on an optimization profile.

Tail recursion needs no annotation:

```zop
fn sum_tail(
    values: i64[n],
    index: int = 0,
    total: i64 = 0,
) -> i64
    if index == values.numel()
        total
    else
        sum_tail(
            values,
            index=index + 1,
            total=total + values[index],
        )
```

The recursive call is the complete result of the `else` branch, so a tensor of
any length uses bounded call-stack space.

Mutual recursion has the same guarantee:

```zop
fn even value: u64 -> bool
    if value == 0
        true
    else
        odd value - 1

fn odd value: u64 -> bool
    if value == 0
        false
    else
        even value - 1
```

This recursion is not tail-recursive and may grow the stack:

```zop
fn factorial value: u64 -> u64
    if value <= 1
        1
    else
        value * factorial(value - 1)
```

A call is in tail position when the function returns the callee's complete
success-or-error result unchanged after destroying its local values. The
compiler detects this position without special syntax. Non-tail recursion may
grow the stack.

GPU `kn` recursion remains target-specific and is not covered by this
guarantee.

Proper tail calls are not part of the first executable subset. They enter the
native application binary interface before that interface freezes.

## Errors

`-> T or fails with E` declares a typed error channel. Exported functions name
`E`; private functions and closures may request inference with bare
`or fails`. Every fallible result must be handled, propagated with `try to`, or
preserved as a value. `fail with` produces an error. Local recovery uses
`catch pattern`; bare `catch` is not valid syntax. See the [error
contract](errors.md).

## Open decisions

The frontend must not guess these semantics:

- Strict cross-target accuracy for non-integral floating exponentiation and
  other transcendental standard functions.
- Layout-requirement syntax plus the concrete tensor ABI at raw-pointer and
  foreign-function boundaries.

Resolve each decision in this page before code depends on it.
