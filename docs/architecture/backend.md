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

Compilation targets the current host. `bedrock mlir` prints verified MLIR and
`bedrock object` writes a native object. Unsupported MLIR operations and types
fail with structured diagnostics; the compiler does not switch backends.

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
caches a separate artifact under that value. Symbolic tensor dimensions do not
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

## Future tail calls

The future proper-tail-call milestone will lower self-recursion to loop
backedges. Other direct and indirect tail calls will use Cranelift's
`return_call` terminators. The translator will verify matching result layouts
and a tail-call-capable calling convention.

Local destruction must run before the transfer. A target that cannot preserve
the bounded-stack guarantee will reject the program instead of emitting an
ordinary call.

## Errors

The bootstrap separates lowering and backend diagnostic codes. Source-backed
backend locations and distinct application binary interface (ABI), CLIF
verification, and target error variants remain required before the backend
contract is complete.

The first implementation targets the current host CPU. Cross-compilation
starts after Bedrock defines its data layout and runtime ABI. A future GPU
backend is a separate target with its own legal IR, not a fallback for failed
CPU compilation.

## GPU direction

NVIDIA's [CuTe IR dialect](https://github.com/NVIDIA/cutlass/pull/3426) is a
candidate lowering target for device layouts. It models hierarchical layout
algebra, tiling, partitioning, static and dynamic layout data, and lowering
through NVVM to GPU binaries. It does not yet model the full tensor-compute,
copy, or matrix-multiply instruction stack, so it is a component rather than a
complete Bedrock GPU backend.

The [`fn` and `kn` contract](gpu.md) defines the aspirational source model,
kernel boundary, runtime calls, and full backend trace. Bedrock owns language
semantics and kernel extraction. It does not reimplement CuTe layout algebra.

MLIR, Melior, Cranelift, CUTLASS, and CuTe IR revisions are pinned together.
Their compatibility is tested as one toolchain.
