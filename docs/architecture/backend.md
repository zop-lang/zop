# Backend

The backend translates restricted Multi-Level Intermediate Representation
(MLIR) to Cranelift intermediate representation (CLIF). Cranelift then emits
central processing unit (CPU) machine code. Rust owns this layer because
Cranelift's supported interface is a Rust application programming interface
(API). Melior provides Rust access to MLIR.

## Implemented bootstrap

The current backend consumes the verified in-memory MLIR module. It translates
supported host `fn` operations over `i64` into one shared scalar form. It then
emits either a Cranelift `JITModule` or `ObjectModule`. Direct calls and local
assignments are covered end to end. Typed just-in-time (JIT) invocation supports
zero, one, or two `i64` arguments. It checks the function signature before
entering its isolated unsafe call boundary.

The bootstrap arithmetic path does not yet emit the language's overflow traps.
Its implemented claim therefore covers non-overflowing executions only;
build-invariant trapping arithmetic is required before the scalar subset is
semantically complete. Its native `i64` `/` and `%` path also implements
truncating bootstrap behavior that is not valid target Zop source semantics.

Compilation targets the current host. `zop mlir` prints verified MLIR and
`zop object` writes a native object. Unsupported MLIR operations and types
fail with structured diagnostics; the compiler does not switch backends.

`zop javascript` lowers typed scalar high-level intermediate representation
(HIR) to a small ECMAScript abstract syntax tree and prints a deterministic
module. The implemented slice supports host functions over `i32`, `f64`,
`bool`, string, and unit values. It uses `Math.imul` and explicit 32-bit
coercion for exact `i32` arithmetic. It rejects `i64`, `f32`, concrete `i32`
division, and kernels instead of selecting a slower or semantically
weaker representation. It may fold `f64 / f64` while type information remains
available. Its generic `f64 % f64` path is also nonconforming scaffolding.
Unused pure expressions disappear, but any nested call remains because calls
are effectful until proven otherwise.

The JavaScript bootstrap's `i32` coercions currently wrap overflow. They are a
representation proof, not yet a conforming implementation of ordinary trapping
integer operators.

## Contract

The production backend accepts one verified CLIF-ready MLIR module and an
explicit target configuration. It returns a just-in-time (JIT) module or an
ahead-of-time (AOT) object file.

For each function, the backend:

1. Declares its name, symbol visibility, parameters, return values, and calling
   convention.
2. Creates all CLIF blocks before translating instructions.
3. Translates each supported MLIR operation exactly once.
4. Marks every control-flow block complete and verifies the resulting CLIF.
5. Defines the function in the selected Cranelift module.

Ordinary integer arithmetic emits a trap edge before an unrepresentable result
becomes observable. Wrapping and saturating operations use their explicit
machine semantics. Fallible numeric members such as `add` branch into the
function's typed error channel. Optimization profiles cannot remove or change
these behaviors.

Numeric division arrives with its source semantics already fixed. The backend:

1. Guards integer zero and signed minimum divided by `-1` before instructions
   for which those inputs trap or are undefined.
2. Uses Cranelift signed or unsigned division directly for truncating mode.
3. Corrects a truncating quotient only when floor, ceiling, or Euclidean mode
   and operand signs require it.
4. Lowers quotient and remainder together when the target exposes both.
5. Preserves the floating-point type and exact selected precision profile.

The backend never implements integer `/` by converting through floating point.
It never implements floor modulo with fixed-width multiply and subtract when
those intermediate operations could overflow. See the
[numeric lowering contract](numerics.md#hir-and-lowering).

JIT and AOT compilation share steps one through four. Only artifact emission
differs.

| Mode | Cranelift module | Output |
| --- | --- | --- |
| JIT | `JITModule` | Executable memory owned by the process |
| AOT | `ObjectModule` | Native object file for the system linker |

Mode selection is explicit. A failed JIT compilation does not retry through
AOT, LLVM, or an interpreter.

Compilation latency is a product requirement. Development and production may
use different explicit optimization profiles, but both preserve semantics.
Every pass is timed. A new pass must justify its compile-time cost with measured
runtime or code-size improvement.

A `known` parameter is resolved before backend entry and does not occupy a
runtime argument slot. When its value changes generated code, the compiler
caches a separate artifact under that value. Symbolic tensor extents do not
create separate artifacts by default. The
[compile-time-values contract](compile-time.md) defines that boundary.

The [explicit input/output contract](io.md) keeps reader and writer hot paths
concrete. Only cold refill, flush, and submission operations use runtime
dispatch.

The [callables contract](callables.md) keeps known functions and methods
symbolic. The backend creates pointers, receiver pairs, closure environments,
or method tables only when runtime selection requires them.

The [error contract](errors.md) reaches the backend as explicit values and
control flow. Ordinary language failures do not use platform exception
unwinding.

## SIMD

The production translator admits only the fixed-width vector types and
operations named by the selected target profile. Accepted MLIR `vector`
schedules map to CLIF vector values, loads, stores, splats, comparisons,
selection, shuffles, and target-supported arithmetic. Predicate aggregation
uses `vany_true` or `vall_true`. First-match lowering uses `vhigh_bits` followed
by scalar `ctz`, with lane-to-bit order covered by target conformance tests.

Vectorization legality is complete before backend entry. The backend does not
invent alias, alignment, bounds, reduction-order, or floating-point
permissions. An unsupported certified schedule is a typed backend diagnostic;
it is never scalarized, sent through LLVM, or retried on another backend. A
scalar schedule selected earlier by the cost model remains an ordinary primary
compilation path.

Cranelift performs legalization and instruction selection for explicit CLIF
vectors. Zop never lowers a required semantic algorithm to scalar CLIF and
expects Cranelift to rediscover its SIMD schedule.

Pinned x86-64 and AArch64 profiles are the first SIMD targets. Each artifact
records its required CPU features. A portable artifact may contain several
independently verified variants and one capability dispatch, as defined by the
[SIMD contract](simd.md); execution failure never triggers redispatch.

## Proper tail calls

The systems-core proper-tail-call milestone lowers self-recursion to loop
backedges. Other direct and indirect tail calls use Cranelift's `return_call`
terminators. The translator verifies matching result layouts and a
tail-call-capable calling convention.

Local destruction must run before the transfer. A target that cannot preserve
the bounded-stack guarantee will reject the program instead of emitting an
ordinary call.

## Errors

The bootstrap separates lowering and backend diagnostic codes. Source-backed
backend locations and distinct application binary interface (ABI), CLIF
verification, and target error variants remain required before the backend
contract is complete.

The first implementation targets the current host CPU. Cross-compilation
starts after Zop defines its data layout and runtime ABI. A future GPU
backend is a separate target with its own legal IR, not a fallback for failed
CPU compilation.

The [browser backend](web.md) is also separate. It branches from typed HIR to
preserve DOM objects, strings, promises, and events in optimized ECMAScript.
Self-contained numeric regions may lower through MLIR to WebAssembly. Cranelift
emits native machine code; it does not emit either browser artifact.

## GPU direction

NVIDIA's [CuTe IR dialect](https://github.com/NVIDIA/cutlass/pull/3426) is a
selected lowering target for `kn` layout operations. Zop's language-native
[Layout](layouts.md) has the same hierarchical coordinate-to-offset semantics
on every target. CPU code expands those operations into integer arithmetic for
Cranelift; device code preserves them as CuTe IR.

CuTe IR models layout algebra, tiling, partitioning, static and dynamic layout
data, and lowering through NVIDIA Virtual Machine (NVVM) operations to GPU
binaries. It does not yet model the full tensor-compute, copy, or
matrix-multiply instruction stack, so it is one component rather than a
complete Zop GPU backend.

The [`fn` and `kn` contract](gpu.md) defines the aspirational source model,
kernel boundary, runtime calls, and full backend trace. Zop owns language
semantics, layout safety, kernel extraction, and the CPU implementation. PyCuTe
supplies executable reference behavior; CuTe IR supplies the device lowering.

MLIR, Melior, Cranelift, CUTLASS, and CuTe IR revisions are pinned together.
Their compatibility is tested as one toolchain.
