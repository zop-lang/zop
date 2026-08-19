# Generics

Generics let one checked definition preserve types across different concrete
inputs. Zop specifies them before implementing them because their source,
coherence, intermediate representation, package, and code-generation contracts
must be stable before self-hosting.

> **Status:** This is the target language contract and conformance plan. The
> Rust bootstrap does not implement user-defined generics yet.
>
> **Tracking:** [Proposal #2](https://github.com/zop-lang/zop/issues/2) is
> accepted for the [0.4.0 milestone](https://github.com/zop-lang/zop/milestone/1).
> Its entry gate keeps implementation behind the fixed-`f32` tensor CPU slice
> and three proven type-only duplications.

## Scope

The first generic system supports types, free functions, and statically
dispatched methods parameterized by types. Type arguments normally infer from
ordinary arguments and an expected result type.

It does not initially include higher-kinded types, variadic type packs,
associated types, default type arguments, reflection over type parameters,
partial specialization, or template metaprogramming. Symbolic tensor
extents and `known` values remain separate concepts.

Physical tensor [layout](layouts.md) is also a value property rather than a
generic parameter. A generic tensor algorithm may inspect or constrain a
layout without multiplying the public tensor type by backend, placement, or
schedule parameters.

Implementation starts only after at least three useful definitions would
otherwise be duplicated with only their types changed. It must land before
freezing `core`, collections, or the self-hosted compiler surface, but it is not
required for the first scalar or fixed-`f32` MLIR tensor slices.

## Syntax

Square brackets declare and apply type parameters:

```zop
type Pair[Left, Right]
    left: Left
    right: Right

type Option[Value]
    case Some value: Value
    case None

fn identity[Value] value: Value -> Value
    value
```

Ordinary calls infer type arguments:

```zop
pair = Pair left=1, right="one"       # Pair[i64, str]
answer = identity 42                  # identity[i64]
name = empty[str]()                   # explicit because no value constrains it
```

The brackets are type application, not call punctuation. `()` remains required
for a zero-argument call. Trailing commas follow the ordinary single-line and
multiline formatting rules.

Tensor shapes occupy a separate argument kind:

```zop
fn map[Input, Output] values: Input[n], transform: fn value: Input -> Output
    -> Output[n]

options: Option[f32][n]
```

`Input` and `Output` are type parameters. `n` is a symbolic extent.
`Option[f32][n]` is a tensor of `Option[f32]` values. Name resolution assigns
each bracket argument a kind before type checking; ambiguity is a diagnostic.

## Constraints

A `type` body containing required function signatures defines a behavioral
contract:

```zop
type Ordered
    fn compare self, other: Self -> Order

fn maximum[Value] values: Value[n] -> Value where Value is Ordered
    best = values[0]
    for value in values[1:]
        if value.compare best is Greater
            best = value
    best
```

`where` is the only constraint spelling. A type satisfies a contract
structurally when its callable members match every required signature. Methods
belong to the type that defines them; another package cannot retroactively add
members to a foreign type. Zop therefore needs no global implementation
registry, overlapping blanket implementations, or orphan exception.

Constraints state available operations, not enumerations of permitted concrete
types. The initial system has no Go-style underlying-type operator, union of
types, or special syntax equivalent to `~int | ~string`. A finite set of values
is a sum type; a reusable operation is a behavioral contract.

`where Type is Contract` is the only constraint spelling. Multiple
requirements form an intersection:

```zop
fn sorted_copy[Value] values: Value[n] -> Value[n]
where Value is Ordered and Copyable
    ...
```

Constraint satisfaction never participates in overload selection. A callable
name resolves before generic inference, so adding a dependency cannot silently
select a different implementation. Alternate policies such as descending sort
order are explicit callable arguments rather than competing global instances.

## Definition checking

The frontend checks a generic body once against only its declared constraints.
An unconstrained operation is an error at the definition, even if every current
caller happens to support it:

```zop
fn add[Value] left: Value, right: Value -> Value
    left + right  # error: Value has no declared addition contract
```

Call sites prove each inferred or explicit type argument satisfies every
constraint. Diagnostics point to the generic declaration, the failed clause,
and the missing or mismatched operation. Instantiation never discovers a new
source-level type error.

Generic inference is local and bidirectional. It uses argument types, expected
result types, and constraints. It does not infer a generic declaration, invent
a union, choose a runtime-dynamic representation, or generalize a local
binding. If more than one substitution remains possible, the call supplies
explicit brackets.

## Ownership, effects, and errors

A generic body obeys the same ownership modes as concrete code. A type
parameter is borrowed unless its parameter says `mut` or `give`. Operations
that require copying, sharing, destruction, sending, or synchronization appear
as constraints and are proven before HIR.

Effects and error channels do not hide inside a type parameter. The generic
declaration exposes its `Io`, `Mem`, mutation, suspension, unsafe, success, and
failure contracts exactly as a concrete declaration does. Substitution cannot
widen them.

View origins are compiler facts rather than lifetime type parameters. A generic
view follows the same inferred or explicit `from` contract as a concrete view.
Raw pointer construction composes normally, including `Value*` and
`Value*[n]`.

## Methods and dynamic dispatch

A statically known receiver may have a method with its own type parameters:

```zop
fn map[Output] self: Series[Input], transform: fn value: Input -> Output
    -> Series[Output]
```

The receiver supplies `Input`; the call infers `Output`. A runtime method table
cannot require a generic method because its unbounded set of instantiations is
not a finite application binary interface. A dynamic boundary uses
non-generic required methods or an explicitly erased adapter.

This permits useful generic methods without making every generic contract
object-safe or banning them globally.

## Compilation model

The frontend emits one polymorphic typed-HIR body. Packages store that body plus
its constraints and interface hash. Downstream packages never include source,
reparse a header, or re-type-check the body.

Code generation uses adaptive specialization:

- Layout, application binary interface, address space, tensor rank, GPU device
  code, and `known` values force a concrete specialization.
- Compatible representations may share machine code with a typed operation
  table when doing so preserves unboxed values and the published performance
  floor.
- Release optimization may clone and inline hot instances within an explicit
  code-size budget. Development builds prefer cached shared instances.
- No specialization may change overload resolution, effects, errors, ownership,
  arithmetic, or observable behavior.

Every instance has a content key over the generic HIR, concrete type and layout
identities, resolved contract witnesses, target, compiler version, and relevant
`known` values. The artifact store deduplicates the key across a workspace.
Recursive instantiation must reach an existing key; unbounded instance growth
is a compile error.

This model keeps the runtime path statically optimizable without requiring one
machine-code copy for every source-level type combination.

### Elaboration pipeline

Elaboration is the compiler stage that binds the concrete types, layouts,
evidence that each type satisfies its contracts, and `known` values required by
one code-generation instance. It never performs source parsing or discovers a
missing generic constraint.

The pipeline is ordered:

1. Check the polymorphic HIR body, including ownership, effects, errors,
   origins, and constraints.
2. Simplify that checked body before instance expansion by removing unused
   parameters, dead pure operations, and proven redundant abstractions.
3. Compute the content-addressed instance key and reject an unbounded recursive
   expansion before generating another body.
4. Substitute concrete arguments and evaluate pure `known` expressions.
5. Verify the resulting concrete HIR before target lowering.
6. Share or specialize machine code according to the documented adaptive
   policy.

Independent instance keys may elaborate in parallel. Symbol identity,
diagnostic order, cache entries, and emitted artifacts remain deterministic.
The compiler never makes concurrent discovery order part of type identity.

Mojo 1.0 validates the value of a distinct checked parametric IR,
pre-elaboration simplification, a compile-time interpreter, and parallel
instance expansion. Zop adopts those phase boundaries without adopting Mojo's
universal monomorphization or parser-emitted MLIR.

## Type identity and application binary interfaces

`Container[T]` and `Container[U]` are distinct types unless `T` and `U` are the
same after transparent-alias resolution. A `distinct` wrapper remains distinct.
User-defined generic containers are invariant: `Container[Child]` is not a
subtype of `Container[Parent]`.

Zop has no raw generic type. Omitting required type arguments is valid only when
inference supplies them; it never erases them to an unchecked universal value.
Concrete type identity survives compilation even when machine code is shared.
Values are not boxed merely because they pass through generic code.

An unspecialized generic declaration has no C application binary interface.
Foreign exports name concrete instances or provide a concrete wrapper. Symbol
mangling and package interfaces include the complete concrete type identity.

## Why this composition

Go correctly delayed generics until repeated code demonstrated the need and
made call-site inference ordinary. Zop keeps that discipline and structural
behavioral contracts. It avoids unions of underlying types, special operators
such as `~`, and implementation concepts that leak into the language model.

Rust proves that definition-time trait checking, coherence, and specialized
machine code can be zero-overhead. Zop keeps those properties without requiring
lifetime parameters in ordinary source, multiple constraint spellings, a
global trait-implementation registry, or universal monomorphization.

Java prioritized migration compatibility through erasure. Its raw types,
unchecked conversions, bridge methods, and lost generic representation show
why Zop must reify concrete type identity and reject missing type arguments.

C++ templates prove the performance ceiling but distribute definitions,
discover dependent errors during instantiation, and expose specialization and
metaprogramming as a second language. Zop distributes checked HIR, diagnoses
the definition once, and keeps specialization compiler-owned.

## Delivery plan

Generics progress through five ordered slices. A later slice cannot compensate
for a failed earlier contract:

1. **Evidence.** Complete the fixed-`f32` tensor CPU slice, name three concrete
   type-only duplications, and record frontend, memory, artifact-size, and
   runtime baselines. No generic syntax is implemented in this slice.
2. **Frontend.** Parse type parameters and `where` clauses, resolve type and
   shape-argument kinds, then check inference, constraints, ownership, effects,
   errors, and origins into one polymorphic HIR body.
3. **Elaboration.** Simplify checked polymorphic HIR, store its interface hash,
   define content-addressed instance keys, evaluate `known` expressions, and
   reject unbounded recursive expansion.
4. **Code generation.** Implement deterministic parallel full specialization
   as the correctness baseline. Measure shape sharing and typed operation tables
   against it before selecting development and release policies.
5. **Adoption.** Migrate the three motivating consumers, delete their concrete
   duplication, and pass diagnostics, conformance, fuzzing, compile-time,
   binary-size, and runtime gates.

The milestone closes only after adoption. Writing a parser production or
generating one successful instance is not delivery.

## Required tests

- Infer type arguments from values and an expected result.
- Require explicit arguments when inference is ambiguous or unconstrained.
- Reject a generic body that uses an operation absent from its constraints.
- Diagnose a failed constraint at the call without an instantiation backtrace.
- Prove structural satisfaction uses the type's own members only.
- Reject raw generic types, overlapping resolution, and retroactive members.
- Preserve transparent aliases and distinguish nominal wrappers.
- Keep user-defined generic containers invariant.
- Preserve ownership, effects, errors, and view origins through substitution.
- Distinguish type arguments from symbolic tensor extents.
- Deduplicate equal instances across package and workspace boundaries.
- Reject unbounded recursive instantiation.
- Prove pre-elaboration simplification preserves generic semantics and reduces
  the measured instance workload.
- Produce identical instances, symbols, diagnostics, and artifacts under
  single-threaded and parallel elaboration.
- Compare shared and specialized instances against the reference interpreter.
- Enforce compile-time, binary-size, and runtime budgets for the generic corpus.
- Require concrete wrappers for foreign exports.

## References

- [Go generics proposal](https://go.dev/blog/generics-proposal)
- [Go: when to use generics](https://go.dev/blog/when-generics)
- [Go type parameters and inference](https://go.dev/ref/spec#Type_parameter_declarations)
- [Go: deconstructing type parameters](https://go.dev/blog/deconstructing-type-parameters)
- [Go: removing core types](https://go.dev/blog/coretypes)
- [Go dictionary and shape implementation](https://go.googlesource.com/proposal/+/master/design/generics-implementation-dictionaries-go1.18.md)
- [Rust generic parameters](https://doc.rust-lang.org/reference/items/generics.html)
- [Rust bounds](https://doc.rust-lang.org/reference/trait-bounds.html)
- [Rust coherence](https://doc.rust-lang.org/reference/items/implementations.html#trait-implementation-coherence)
- [Rust monomorphization](https://rustc-dev-guide.rust-lang.org/backend/monomorph.html)
- [Java type erasure and raw types](https://docs.oracle.com/en/java/javase/26/docs/specs/jls/jls-4.html)
- [ISO C++ template FAQ](https://isocpp.org/wiki/faq/templates)
- [Swift intermediate representation and witness tables](https://github.com/swiftlang/swift/blob/main/docs/SIL/SIL.md)
- [Mojo compiler elaboration pipeline](https://github.com/modular/modular/blob/f66d4d522c34be0a961ffac3dbfc81e30f67942e/KGEN/docs/MojoCompilerWalkthrough.md)
