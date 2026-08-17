# Runtime

The runtime provides services that generated machine code cannot own alone.
It stays behind a small native application binary interface (ABI) shared by
just-in-time (JIT) and ahead-of-time (AOT) code.

## Responsibilities

The runtime owns:

- Process entry and termination.
- Storage requested through `Mem` by generated code.
- Tensor storage and lifetime operations.
- Standard input and output.
- The selected explicit input/output implementation.
- Registration of symbols callable from JIT code.

The runtime does not parse source, optimize programs, select compiler modes,
or repair unsupported code generation.

## Tensor boundary

A tensor value eventually needs a data pointer, element type, rank, shape,
strides, and ownership state. The exact descriptor remains open until the
fixed-rank matrix multiplication slice requires it. Defining it earlier would
freeze a memory model without executable evidence.

Static shape and layout data stay in the type system. The runtime descriptor
materializes only values that remain dynamic after compilation. This contract
must be proven against CPU and GPU layouts before it becomes stable.

The runtime ABI must state who allocates every buffer, who frees it, and how a
callee may retain it. Generated code must not infer ownership from a pointer.

The [memory-management contract](memory.md) defines ownership, borrowing,
destruction, shared values, arenas, and optional managed regions.

The [explicit input/output contract](io.md) defines capability passing, buffering,
flush and close obligations, asynchronous work, cancellation, and test
implementations.

## Errors and symbols

Language errors follow the typed [`fails` contract](errors.md) and remain
ordinary values. Runtime faults such as `mem.alloc` failure return through a
declared error channel or terminate according to a documented contract. They
do not unwind as language exceptions or silently switch execution modes.

JIT compilation registers every permitted runtime symbol before finalization.
The compiler rejects undeclared imports instead of searching the host process
for a similarly named function.
