# Numeric semantics

Zop fixes numeric type, precision, rounding, and failure behavior before
lowering. A backend may optimize a proven operation, but it cannot choose a
different numeric representation from runtime values or target convenience.

> **Status:** This page defines target language semantics. The Rust bootstrap
> still accepts same-type `/` and `%` for every numeric type. Integer `/`,
> floating `%`, and truncating integer `%` are nonconforming scaffolding. The
> bootstrap does not implement `//`, numeric cast policy, arithmetic
> traps, or floating-point profiles yet.

## Invariants

- Every numeric expression has one type before high-level intermediate
  representation (HIR).
- A runtime value never selects between an integer and floating-point result.
- Named values never promote implicitly.
- Numeric literals may adopt an expected type only while they remain literals.
- An operation never changes element type merely because it targets a central
  processing unit (CPU), graphics processing unit (GPU), browser, or device.
- Rounding and precision loss are explicit semantics, not optimization choices.
- An ordinary integer trap terminates its execution domain; it never changes
  into wrapping arithmetic, a typed failure, or a floating-point sentinel.

## Division operators

Division separates fractional and integral intent:

| Source | Operand type | Result | Meaning |
| --- | --- | --- | --- |
| `left / right` | Same floating-point type | Same type | Fractional division |
| `left // right` | Same integer type | Same type | Floor division |
| `left % right` | Same integer type | Same type | Modulo paired with `//` |

All three operators have the precedence of multiplication and associate from
left to right. `//` is one source token, not a comment; `#` introduces ordinary
comments and the `##` structured-documentation form. `/=` updates
floating-point scalars. `//=` and `%=` update integer scalars. Each update
evaluates its target once.

Binary numeric operators evaluate the left operand and then the right operand,
each exactly once. An optimizer may reorder only proven-pure work when the
selected integer trap and floating-point profile remain observationally
identical.

`/` never accepts a concrete integer operand. `//` and `%` never accept a
floating-point operand. Mixed concrete types are compile errors.

## Literal typing

The operator may provide context while its operands are still literals:

```zop
half = 1 / 2          # f64 0.5
half: f32 = 1 / 2     # f32 0.5
quotient = 7 // 2     # i64 3

count: i64 = 7
size: i64 = 2

ratio = count / size  # compile error: named integers never promote
quotient = count // size
```

Without another expected floating-point type, a literal-only `/` expression
defaults to `f64`. Every integer literal must be exactly representable in the
selected floating-point type. For example, an unrepresentable large integer in
an `f64` division is a compile error rather than a rounded literal.

A binding ends contextual literal inference. The compiler never revisits a
binding's type after seeing a later division.

## Integer quotient and modulo

For nonzero divisor `b`, floor quotient `q` and modulo `r` satisfy:

```text
q = floor(a / b)
r = a - b * q
a = b * q + r
abs(r) < abs(b)
r = 0 or sign(r) = sign(b)
```

These equations use mathematical integers. They do not require evaluating
`abs`, multiply, or subtract in the fixed-width source type when an intermediate
would overflow.

The signed edge cases are fixed:

<!-- markdownlint-disable MD013 -->

| `a` | `b` | Floor `q` | Floor `r` | Trunc `q` | Trunc `r` | Ceil `q` | Ceil `r` | Euclidean `q` | Euclidean `r` |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 7 | 3 | 2 | 1 | 2 | 1 | 3 | -2 | 2 | 1 |
| -7 | 3 | -3 | 2 | -2 | -1 | -2 | -1 | -3 | 2 |
| 7 | -3 | -3 | -2 | -2 | 1 | -2 | 1 | -2 | 1 |
| -7 | -3 | 2 | -1 | 2 | -1 | 3 | 2 | 3 | 2 |

<!-- markdownlint-enable MD013 -->

Euclidean division chooses the unique remainder in `0 <= r < abs(b)`.
Unsigned floor, truncating, and Euclidean division are identical. Unsigned
ceiling division remains distinct when a remainder exists.

Ordinary `//` and `%` trap on a zero divisor. Signed minimum divided by `-1`
traps with overflow for every quotient mode. The paired `%` also traps for that
input even though its mathematical remainder is zero, because no quotient of
the same type satisfies the division contract. These rules are identical in
development and optimized builds.

If every operand is known during compilation, a zero divisor or signed
division overflow is a compile error. Otherwise the trap occurs before a result
or scalar update becomes observable.

## Recoverable integer division

Code that needs an error value uses integer members rather than changing the
operator with surrounding syntax:

```zop
type IntegerRounding
    case Floor
    case Trunc
    case Ceil
    case Euclidean
```

```zop
floor = try to left.divide right, rounding=Floor
truncated = try to left.divide right, rounding=Trunc
ceiling = try to left.divide right, rounding=Ceil
euclidean = try to left.divide right, rounding=Euclidean
exact = try to left.divide_exact right

quotient, remainder = try to left.divide_with_remainder(
    right,
    rounding=Floor,
)
```

`rounding` is required and known during compilation. It creates no runtime
mode branch. `divide_with_remainder` exposes a paired result so code that needs
both values does not require two divisions.

Ordinary rounded division can fail with `DivideByZero` or `Overflow`.
`divide_exact` additionally fails with `InexactDivision` when the divisor does
not divide the dividend. The failure channel remains outside the success tuple.
There is no default rounding argument whose meaning could change later.

These are ordinary members on integer types, not keywords. User modules and
types remain free to define members with the same names.

## Exponentiation

`**` is exponentiation. It associates from right to left and binds tighter than
unary negation:

```zop
power = base ** exponent
nested = 2 ** 3 ** 2       # 2 ** (3 ** 2)
negative = -(value ** 2)
squared_negative = (-value) ** 2
```

An integer base requires an unsigned integer exponent and returns the base type.
An integer literal exponent adopts the required unsigned type context. The
implementation uses exponentiation by squaring or a proven equivalent;
intermediate and final overflow follow the ordinary trapping integer contract.
`value ** 0` is `1` for every integer value, including `0 ** 0`.

A floating base requires the same floating type for its exponent. A direct
numeric literal may adopt that type, so `value: f32` permits `value ** 2` and
`value ** 0.5` without converting a named value. The result preserves the base
type and selected floating-point profile. Tensor exponentiation is elementwise
and follows the ordinary broadcasting contract.

The strict accuracy requirement for non-integral floating exponentiation
remains part of the standard mathematical-function contract; a target cannot
substitute a weaker approximation merely because the source uses operator
syntax.

## Floating-point division

Floating `/` preserves operand and element type. Zop never widens or narrows a
declared floating-point type, or converts an integer because a backend lacks the
requested type.

The default `strict` profile follows IEEE 754 round-to-nearest,
ties-to-even semantics. It preserves signed zero and subnormal values, which are
nonzero values smaller than the normal range. NaN means the IEEE "not a number"
value. Division does not create a typed failure or language trap:

| Inputs | Strict result |
| --- | --- |
| Finite `x`, finite nonzero `y` | Quotient rounded once to the declared type |
| Nonzero finite `x`, signed zero `y` | Signed infinity |
| Signed zero divided by signed zero | NaN |
| Infinity divided by infinity | NaN |
| Finite `x` divided by infinity | Signed zero |
| Infinity divided by finite nonzero `y` | Signed infinity |
| Either operand is NaN | NaN |

The result sign is the exclusive-or of operand signs where IEEE 754 defines a
signed result. NaN payload and NaN sign are not portable language semantics;
classification through `is_nan` is. Nonfinite source constants are explicitly
typed members rather than untyped literals:

```zop
missing = f32.nan
upper = f32.infinity
lower = -f32.infinity
```

Zop has no bare `nan`, `inf`, or `\inf` token. The type-qualified spelling
prevents literal inference from selecting precision accidentally. Ordinary
operations may produce the same values without naming a constant.

A target may expose an explicit `native` floating-point profile when it cannot
meet `strict` efficiently. The selected profile records permitted rounding
error, contraction, reassociation, subnormal flushing, signed-zero changes,
and NaN or infinity assumptions. It is part of HIR, public interface identity,
artifact identity, and cache identity. No compiler phase may introduce or widen
those permissions.

A declaration selects another profile through an ordinary trailing `known`
parameter:

```zop
kn fast_matmul(
    left: f32[m, k],
    right: f32[k, n],
    float_profile: known FloatProfile = Strict,
) -> f32[m, n]
    left @ right

result = fast_matmul left, right, float_profile=Native
```

The default keeps ordinary calls strict. A named argument makes weaker semantics
visible at the call. The `known` requirement eliminates runtime policy branches
and makes the selected profile part of specialization and artifact identity.
Code needing two profiles writes two calls or a small helper boundary rather
than changing ambient region state. A target that cannot implement the selected
type and profile rejects the call rather than silently changing precision.

## Minimum and maximum

`math.min` and `math.max` are pure allocation-free operations. They accept one
or more values of one concrete ordered numeric type:

```zop
from math import min, max

lower = min left, right
upper = max left, right, limit
same = min value
```

Code that prefers qualification uses `import math` and calls `math.min` or
`math.max`. The bundled module is available on every compatible target without
adding a package dependency.

A direct literal may adopt the other arguments' concrete type. Named values do
not promote, so `min(left_f32, right_f64)` is a type error. The one-argument form
is the identity and makes generated argument lists well-defined without an
empty call.

Integer `min` and `max` use mathematical ordering. Floating `min` and `max`
follow the NaN-propagating rules standardized by Go:

| Inputs | `min` | `max` |
| --- | --- | --- |
| `-0.0`, `+0.0` | `-0.0` | `+0.0` |
| `-infinity`, finite `y` | `-infinity` | `y` |
| `+infinity`, finite `y` | `y` | `+infinity` |
| NaN, any `y` | NaN | NaN |

Propagating NaN prevents a reduction from silently discarding invalid numeric
data. Algorithms that intentionally ignore NaN use a separately named library
operation such as `nanmin` or filter values explicitly; they do not change core
`min` or `max`.

Tensor methods apply the same scalar operation as a reduction:

```zop
minimum = values.min axis=0
maximum = values.max axis=0

minimum_or_bound = values.min axis=0, initial=f32.infinity
```

Without `initial`, an empty reduction domain is a static diagnostic when known
and a runtime trap when dynamic. Supplying `initial` makes the reduction total
and includes that value as a candidate. Reduction `order` remains explicit
under the [language contract](language.md#reductions-and-accumulation). `min`
and `max` are associative under the rules above, including NaN propagation and
signed-zero ordering.

Scalar and tensor `min` and `max` are legal in `kn`. A backend must preserve
NaN and signed-zero behavior under the selected floating-point profile or
reject the operation.

## Nonfinite values and validation

NaN and signed infinity are values of floating-point types. They are not hidden
errors. An elementwise floating operation completes with the IEEE result for
every element, even when some results are nonfinite:

```zop
quotient = numerator / denominator
# quotient may contain finite values, NaN, or signed infinity.
```

The scalar predicates `is_nan`, `is_infinite`, and `is_finite` are pure core
operations. Tensor predicates apply elementwise. Code that requires a complete
finite tensor uses an explicit checked boundary:

```zop
try to values.require_finite()
```

`require_finite` examines values in logical coordinate order. It succeeds with
`Unit` or returns `NonFinite` containing the first nonfinite coordinate and
whether that value is NaN, positive infinity, or negative infinity. The check
does not change values, allocate a replacement tensor, or reinterpret NaN as
missing data.

Success establishes a flow-sensitive finite fact for that tensor identity. The
fact remains valid through immutable borrows and is invalidated by mutation,
ownership transfer, an unanalyzed foreign call, or any operation that may
replace storage. Users do not write a refinement type or repeat the check while
the compiler can prove the same value remains unchanged.

Every safe standard-library tensor or linear-algebra operation documents its
behavior for nonfinite inputs. If a vendor implementation provides no such
guarantee, Zop may call it only after an explicit check has established its
finite-input precondition, through a source operation whose contract includes
that check, or after selecting a conforming implementation before lowering.
Otherwise it rejects the target. Safe code never inherits a backend contract
that may return arbitrary data, crash, or corrupt memory for a valid
floating-point value.

The toolchain may provide `--check-nonfinite` instrumentation, compiler-inserted
debugging checks, for `zop run` and `zop test`. Instrumentation reports the
source operation that first produces NaN or infinity and then traps. It is a
debugging aid, not a floating-point profile or a semantic requirement. It may
add device checks, synchronization, and large performance costs; its presence
enters artifact and cache identity. A target that cannot implement the requested
instrumentation rejects the build rather than silently checking less.

Zop does not adopt a mutable ambient policy like NumPy `seterr`. Numeric
behavior remains visible in the operation, declared floating-point profile, or
explicit tool invocation. A library call cannot change how unrelated arithmetic
handles nonfinite values.

## Numeric casts

A cast is an explicit numeric conversion. A division never casts a named value.
Every cast names its destination type and whether representational loss is
permitted:

```zop
exact = try to value.cast f32
rounded = try to value.cast f32, rounding=NearestEven
clamped = value.saturating_cast i16
wrapped = value.wrapping_cast u8
bits = value.bitcast u32
```

`cast` without a rounding argument is exact and fallible. A rounding argument
explicitly permits adjacent-value rounding but does not permit overflow or
underflow. Saturating and wrapping behavior use separate member names. `bitcast`
changes interpretation without performing numeric conversion. These members
are ordinary callable members, not keywords.

The semantic policies are:

<!-- markdownlint-disable MD013 -->

| Policy | Contract |
| --- | --- |
| Exact | Preserve the mathematical value or return `PrecisionLoss`, `Overflow`, or `InvalidConversion` |
| Rounded | Name a rounding mode; reject finite overflow and nonzero-to-zero underflow unless separately permitted |
| Saturating | Clamp to the destination range; never wrap |
| Wrapping | Integer-only reduction modulo the destination width |
| Bit reinterpretation | Preserve bits between equal-width representations; perform no numeric conversion |

<!-- markdownlint-enable MD013 -->

Exact integer-to-float conversion fails when the integer lies between two
representable floating-point values. Rounded integer-to-float conversion must
name `NearestEven`, `TowardZero`, `Floor`, or `Ceil`.

Integer-to-integer exact conversion succeeds only when the destination range
contains the value. A negative signed value therefore cannot convert exactly to
an unsigned type. Integer conversion has no rounding mode; code selects exact,
saturating, or wrapping policy.

Float-to-integer conversion first applies its named rounding mode, then checks
the destination range. NaN and infinity return `InvalidConversion`; an
out-of-range finite value returns `Overflow`. Negative floating-point zero may
convert exactly to integer zero because exact numeric conversion preserves
mathematical value rather than representation bits.

Float narrowing follows the same rule. Exact conversion rejects any changed
finite value. Rounded conversion permits ordinary adjacent-value rounding but
still rejects finite overflow to infinity and nonzero underflow to zero unless
the call explicitly selects those outcomes. Float widening is exact for finite
values. NaN payload preservation is never promised by numeric conversion.
Infinity converts exactly between floating-point types that represent it.
Floating negative zero remains negative zero when the destination is floating
point.

Saturating integer conversion clamps below the destination minimum or above its
maximum. Saturating float-to-integer conversion still requires a rounding mode,
rejects NaN, and maps infinities to the corresponding bound. Saturating float
narrowing maps finite overflow to the largest finite destination value rather
than infinity.

Wrapping conversion exists only between integers. It reduces the mathematical
value modulo the destination width and then interprets those bits with the
destination signedness. Bit reinterpretation requires equal bit widths and
preserves every bit, including a floating-point NaN payload; it makes no claim
that the numerical values are equal.

The compiler does not treat target placement, a target's preferred type, or a
package precision setting as cast permission. Casting to the source type returns
the original value and may be eliminated.

Representative exact-conversion boundaries are fixed:

<!-- markdownlint-disable MD013 -->

| Source value | Destination | Result |
| --- | --- | --- |
| Integer `2^53` | `f64` exact | `2^53` |
| Integer `2^53 + 1` | `f64` exact | `PrecisionLoss` |
| Signed `-1` | Any unsigned integer exact | `Overflow` |
| Floating `3.0` | Integer exact | `3` |
| Floating `3.5` | Integer exact | `PrecisionLoss` |
| Floating `-0.0` | Integer exact | `0` |
| NaN or infinity | Integer | `InvalidConversion` |
| Finite value above destination maximum | Narrower float | `Overflow` |
| Nonzero value below destination's smallest representable magnitude | Narrower float | `Underflow` unless explicitly permitted |

<!-- markdownlint-enable MD013 -->

## Tensors and vectors

Numeric operators apply elementwise without changing element type:

```text
f32[n] / f32[n] -> f32[n]
i32[n] // i32[n] -> i32[n]
i32[n] % i32[n] -> i32[n]
i32[n] / i32[n] -> compile error
```

Shape compatibility and broadcasting follow their separate contract. A scalar
literal may adopt the tensor element type only when that literal is a direct
operand. A named scalar never promotes to the tensor element type.

No implementation scans values to decide whether an integer tensor should
become floating point. A pure tensor quotient publishes no result if any lane
traps. An incomplete private result is destroyed with the terminating execution
domain.

Trapping in-place tensor updates are legal:

```zop
mut destination += right
mut quotient //= divisor
```

The implementation may have written some elements before a trap. Zop promises
no rollback because a trap is not recoverable: native CPU execution terminates,
and device execution invalidates its complete execution context. No continuing
safe Zop code may observe the partially written tensor. The
[runtime trap contract](runtime.md#traps-and-execution-domains) defines each
target's execution domain.

Wrapping and saturating in-place operations cannot fail and remain usable after
completion. A recoverable fallible tensor operation produces a fresh result and
publishes it only on success. Core does not initially provide a recoverable
in-place operation. Such an operation would need to guarantee unchanged storage
on failure through an explicit validation or temporary-output strategy, or
return a type that makes partial state impossible to mistake for a complete
tensor. The compiler never inserts a hidden preflight pass or transaction.

Inside `kn`, a fallible numeric member may be handled locally and converted into
ordinary data, a mask, or an explicitly designed error buffer. Its failure may
not propagate through the kernel boundary. An unhandled device trap follows the
execution-context invalidation contract rather than becoming a per-thread
language error.

## HIR and lowering

HIR records the operation kind, signedness, scalar or element type, quotient
mode, trap or typed-failure policy, and floating-point profile. There is no
generic division node whose meaning is chosen by a backend.

Before emitting Multi-Level Intermediate Representation (MLIR), lowering
guards every integer divisor against zero and every signed minimum divided by
`-1`. This converts MLIR undefined behavior or poison, an invalid value that may
taint later optimization, into the Zop trap or typed error contract.

| Zop operation | Primary lowering |
| --- | --- |
| Strict floating `/` | `arith.divf` with no unapproved fast-math flags |
| Signed integer `//` | `arith.floordivsi` |
| Unsigned integer `//` | `arith.divui` |
| Truncating division | `arith.divsi` or `arith.divui` |
| Ceiling division | `arith.ceildivsi` or `arith.ceildivui` |
| Exact division | Guard divisibility, then use an exact integer division fact |
| Floor modulo | `a - b * floor_divide(a, b)` or a proven equivalent |

The floor-modulo formula defines the result, not a required implementation.
Lowering uses a truncating remainder plus a conditional correction or widened
arithmetic so intermediate multiply or subtract cannot create an overflow that
the modulo operation itself does not have.

Cranelift signed division rounds toward zero. Floor and ceiling modes therefore
add the mathematically required correction only when signs and a nonzero
remainder require it. A proof that operands are nonnegative removes the floor
correction. Paired quotient-and-remainder operations lower together when the
target supports that result directly.

WebAssembly integer division is truncating, so floor and ceiling use the same
explicit correction. JavaScript integer lowering cannot use the JavaScript `/`
operator as integer division; it must preserve Zop width, rounding, and traps.

WebGPU Shading Language (WGSL) has materialized `f16` and `f32` but no `f64`,
and its floating-point accuracy is weaker than the `strict` profile. A WebGPU
kernel using unsupported `f64` or strict division is rejected unless its
declared target provides a conforming primary implementation. Selecting a
native WGSL profile never converts integer operands or changes element type.

## Required diagnostics

- Integer operands to `/` point to the operator and suggest `//`, an explicit
  cast, or a named division member.
- An expected integer result ranks `//`; an expected floating result ranks exact
  casts to that precise floating type. With no expected type, both remain
  labelled semantic alternatives.
- Floating-point operands to `//` or `%` state that those operators require
  integers.
- Mixed concrete types identify both operand types and never suggest an
  implicit promotion.
- Compile-time zero, overflow, inexact division, and invalid casts point
  to the operand that violates the contract.
- An unsupported target type or profile names the exact target requirement;
  it never suggests a silent narrower type.
- A known exact-cast failure shows the unrepresentable value and may suggest a
  rounded or saturating cast, but never applies one automatically.
- A finite-input requirement suggests `require_finite` and states its scan and
  synchronization cost; it never inserts the check silently.
- Nonfinite debug instrumentation identifies itself as an opt-in tool mode and
  never implies that ordinary IEEE execution would fail.

## Required tests

- Parse `/`, `//`, `%`, `/=`, `//=`, and `%=` with multiplication precedence
  and left associativity; prove `//` never starts a comment.
- Parse `**` above unary negation with right associativity; distinguish
  `-(x ** y)` from `(-x) ** y`.
- Exercise zero, one, maximum, nested, and overflowing integer exponents at
  every width; reject a signed or floating exponent for an integer base.
- Infer literal-only `/` as `f64` by default and as an expected floating type
  when supplied.
- Reject every concrete integer `/`, floating `//`, floating `%`, mixed named
  type, and implicit tensor cast.
- Exhaust the signed quotient table above at every integer width.
- Prove the quotient-remainder law for floor, truncating, ceiling, and
  Euclidean modes with property tests.
- Exercise zero, one, negative one, minimum, maximum, exact, and inexact inputs
  for every integer width.
- Prove compile-time invalid operations diagnose while runtime equivalents trap
  or return the declared error.
- Compare strict floating special values, signed zero, finite boundaries,
  subnormals, and rounding halfway cases with a software reference.
- Parse typed `f16`, `f32`, and `f64` NaN and infinity constants; reject bare or
  backslash-prefixed nonfinite tokens.
- Exercise every exact and rounded cast boundary, including adjacent
  unrepresentable integers, NaN, infinity, overflow, and underflow.
- Compare scalar, vector, and tensor results across the interpreter, Cranelift,
  JavaScript, WebAssembly, native GPU, and WebGPU target profiles.
- Prove development and optimized builds preserve type, trap, error, and
  precision semantics.
- Prove cache keys and public interface hashes change when the floating-point
  profile changes.
- Permit trapping tensor updates without inserting rollback, preflight, or
  temporary storage; prove a trap prevents every later access in that execution
  domain.
- Prove wrapping and saturating tensor updates complete without entering the
  trap path.
- Reject recoverable in-place tensor arithmetic until it has an explicit
  unchanged-on-error or partial-state type contract.
- Exercise `is_nan`, `is_infinite`, `is_finite`, and `require_finite` over
  dense, strided, broadcast, empty, CPU, and device tensors.
- Exercise scalar and tensor `min` and `max` over every integer width, finite
  floating boundaries, signed zero, infinities, NaNs, one argument, broadcasts,
  empty reductions, and explicit `initial`.
- Prove `--check-nonfinite` changes artifact identity, reports the originating
  operation, and fails rather than silently weakening unsupported targets.
- Snapshot intent-aware diagnostics for integer `/`, floating `//`, mixed
  concrete types, unhandled cast failure, precision loss, and unsupported target
  profiles.
- Apply every machine-applicable numeric suggestion and prove the edited source
  parses and type-checks without introducing an implicit cast.

## References

- [MLIR arithmetic operations](https://mlir.llvm.org/docs/Dialects/ArithOps/)
- [MLIR index operations](https://mlir.llvm.org/docs/Dialects/IndexOps/)
- [Cranelift signed division](https://docs.rs/cranelift-codegen/latest/cranelift_codegen/ir/struct.InsertBuilder.html#method.sdiv)
- [JAX true division](https://docs.jax.dev/en/latest/_autosummary/jax.numpy.true_divide.html)
- [GHC fractional and integral division](https://downloads.haskell.org/ghc/9.14.1/docs/libraries/ghc-9.14.1-da80/GHC-Prelude-Basic.html)
- [WebGPU Shading Language numeric types](https://www.w3.org/TR/WGSL/#scalar-types)
- [WebGPU Shading Language floating-point accuracy](https://www.w3.org/TR/WGSL/#floating-point-accuracy)
- [Swift overflow operators](https://docs.swift.org/swift-book/documentation/the-swift-programming-language/advancedoperators/#Overflow-Operators)
- [Zig integer overflow](https://ziglang.org/documentation/master/#Integer-Overflow)
- [Rust checked and strict integer addition](https://doc.rust-lang.org/std/primitive.i32.html#method.checked_add)
- [Rust IEEE floating-point semantics](https://doc.rust-lang.org/std/primitive.f32.html)
- [NumPy floating-point error handling](https://numpy.org/doc/2.4/reference/routines.err.html)
- [PyTorch numerical accuracy](https://docs.pytorch.org/docs/stable/notes/numerical_accuracy.html)
- [JAX NaN debugging](https://docs.jax.dev/en/latest/debugging/flags.html#jax-debug-nans-configuration-option-and-context-manager)
- [Go `min` and `max`](https://go.dev/ref/spec#Min_and_max)
