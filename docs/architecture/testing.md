# Testing

Bedrock is correct when every execution path agrees on language behavior.
Passing one backend or one happy-path program is not enough.

## Current suite

The bootstrap tests lexer layout and rejection rules, parser structure and
error recovery, scalar type checking, verified Multi-Level Intermediate
Representation (MLIR) emission, contextual numeric literals, direct calls,
local assignments, typed just-in-time (JIT) invocation, and native object
emission. One test executes generated machine code and proves `20 + 22 == 42`.
The suite also proves that a `kn` kernel cannot fall back to the central
processing unit (CPU) backend.

The semantic oracle, tensor corpus, ownership checker, effect system, and
graphics processing unit (GPU) execution suite below are release gates. They
are not implemented yet.

## Semantic oracle

A planned small interpreter will define the behavior of restricted MLIR
independently of Cranelift. Every supported CPU operation has one interpreter
case, one JIT case, and one ahead-of-time (AOT) case. Tests compare values,
output, and failure class.

The interpreter never runs because native compilation failed. It exists only
for tests and debugging.

## Test layers

| Layer | Required proof |
| --- | --- |
| Lexer and parser | Layout, delimiters, precedence, spans, and errors |
| Type and ownership checker | Valid programs plus exact rejection spans |
| Compile-time values | Binding-time checks, purity, caching, and ABI erasure |
| Callables | Members, calls, captures, lifetimes, and dispatch |
| Errors | Typed channels, mandatory handling, propagation, and recovery |
| Typed frontend intermediate representation to MLIR | Verified MLIR and canonical golden output |
| MLIR lowering | No high-level operation survives the base boundary |
| MLIR to Cranelift intermediate representation (CLIF) | Verified CLIF and differential interpreter results |
| Native interface | 0-8 arguments, recursion, callbacks, and alignment |
| Input and output | Deterministic effects, flush, close, and cancellation |
| Runtime | `Mem`, ownership, bounds, uninitialized reads, and faults |
| Autodiff | Activity, gradients, checkpointing, and numerical agreement |
| Graphics processor | Structural lowering plus execution on real hardware |

Correctness gates quality tests. Runtime speed, compile latency, memory use,
code size, and register pressure are measured only after the program passes the
semantic suite.

## Conformance corpus

Small tests isolate one rule. Larger programs prove the rules compose. The
corpus grows from arithmetic and control flow into parsers, graph algorithms,
renderers, systems utilities, tensor libraries, and model workloads.

Every fixed bug adds the smallest program that expresses the violated language
invariant. Test names describe the invariant, not the historical failure.

## MLIR boundaries

Each MLIR layer has its own verifier tool. The CLIF-ready verifier does not
register high-level Bedrock dialects, so leaked operations fail at parse or
verification time. End-to-end tests remain separate from these structural
tests because a plausible IR dump does not prove executable behavior.
