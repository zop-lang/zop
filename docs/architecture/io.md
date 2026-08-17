# Explicit input and output

Every operation that may block or observe nondeterminism requires an explicit
input/output (I/O) capability named `Io`. Bedrock has no ambient global I/O
runtime.

> **Status:** This page defines target semantics. Names and example syntax are
> provisional until the standard-library interface is implemented.

## Scope

`Io` owns access to:

- Files, directories, and memory maps.
- Networking and name resolution.
- Time, sleep, and timers.
- Entropy and random sources.
- Process creation and process state.
- Blocking synchronization.
- Asynchronous submission, waiting, and cancellation.

Pure computation, ordinary memory access, and lock-free synchronization do not
require `Io`.

## Capability passing

The process entrypoint constructs one I/O implementation and passes it through
the application. Functions accept only the capabilities they need.

```bedrock
fn main init: Process -> App or fails with RunError
    return try to run init.io, init.mem

fn run io: Io, mem: Mem -> App or fails with RunError
    config = try to read_config io, mem, path: "bedrock.toml"
    return try to build io, mem, config
```

An ordinary `Io` parameter is an immutable borrow. A context object may retain
that borrow when its lifetime is connected to the process owner. Hiding `Io` in
a global variable is not permitted.

The implementation may synchronize its internal state. Borrowing the
capability immutably does not make an I/O operation pure.

## Implementations

The root program selects one implementation:

- `Io.Threaded` uses blocking operations and threads.
- `Io.Evented` integrates operations with an evented scheduler.
- `Io.Test` provides deterministic files, time, entropy, and failures.

These names are provisional. The semantic rule is not: an implementation
either performs the requested operation or returns a typed unsupported error.
It never switches to another implementation silently.

## Readers and writers

`Reader` and `Writer` are concrete, non-generic interface values. Their buffer,
cursor, capacity, and common hot-path logic are visible to the optimizer. A
small cold-path table handles refill, flush, submission, and implementation
specific operations.

Readers and writers borrow their underlying source or sink. The resource owner
retains the separate obligation to close a file, socket, or process stream.

Writing a byte that fits in the buffer performs direct loads and stores. It
does not make an indirect call. Filling or flushing the buffer crosses the
runtime-selected boundary.

Callers supply or explicitly allocate buffers:

```bedrock
fn copy(
    io: Io,
    mut src: Reader,
    mut dst: Writer,
) -> u64 or fails with IoError
    copied = 0
    while chunk = try to src.read io
        copied = copied + try to dst.write chunk

    try to dst.flush io
    return copied
```

This layout preserves runtime polymorphism without putting the common path
behind a virtual call.

## Flush and close obligations

Buffered output is never flushed by a destructor. Hidden destructor I/O cannot
report failure and makes latency appear at unrelated scope exits.

An owned writer is a must-finish resource. Every control-flow path must consume
it with one of these operations:

- `finish io` flushes, consumes the writer, and returns any error.
- `discard` abandons buffered output without performing I/O.

`flush io` writes pending bytes but keeps the writer usable. Borrowed writers
leave the finish obligation with their owner.

The ownership checker rejects an owned writer dropped without `finish` or
`discard`. Error paths may propagate an earlier error only after explicitly
discarding or otherwise resolving the writer.

Files, sockets, and other fallible resources are must-close values. Every path
closes them or transfers ownership; destructors never hide a fallible close.

## Asynchrony and cancellation

Asynchrony belongs to the `Io` library, not to a second class of functions.
Ordinary `fn` declarations may submit work when they receive `Io`.

```bedrock
future = io.async fetch, io, url

do_cpu_work

response = future.await io
```

The exact call syntax is provisional. An owned future must be awaited or
canceled before it is dropped. Cancellation is a request and may be rejected or
race with completion; the result reports which outcome occurred.

An implementation without concurrent execution may complete submitted work
immediately. This preserves program semantics and remains the behavior of that
selected implementation, not a fallback to another runtime.

## Effects and kernels

The frontend records every I/O operation as an effect in high-level
intermediate representation (HIR). A function proven pure cannot perform I/O.
Passing an `Io` capability does not make a function effectful until it uses an
effectful operation.

A graphics processing unit (GPU) `kn` cannot receive or use host `Io`.
Device-side debugging or target services require separate typed device
operations.

## Memory and lifetime integration

The process owns the root `Io` implementation. Readers and writers borrow their
buffers and implementation state. Futures and operation groups own all task
state until completion or cancellation.

Files, sockets, writers, and futures follow the
[memory-management contract](memory.md). Reference counting or garbage
collection cannot replace explicit finish, close, await, or cancel obligations.

## Backend contract

Host lowering preserves concrete reader and writer fields in Cranelift
intermediate representation (CLIF). It lowers only cold-path operations to
indirect calls through the selected I/O implementation.

The compiler keeps hot methods available for inlining across module boundaries
through cached HIR or optimization summaries. Splitting a package must not
silently turn buffered byte operations into virtual calls.

## Required tests

- Reject I/O without an `Io` capability.
- Reject `Io` use in a pure function or GPU kernel.
- Reject dropping an unfinished owned writer.
- Reject dropping an unresolved future.
- Surface flush, close, await, and cancellation failures.
- Prove `Io.Test` controls files, time, entropy, and injected failures.
- Prove buffered byte writes make no indirect call on the hot path.
- Prove one final flush emits the buffered data exactly once.
- Run the same suite against threaded and evented implementations.

## References

- Andrew Kelley's
  [Don't Forget to Flush](https://www.youtube.com/watch?v=f30PceqQWko&t=1227s)
- [Zig 0.15.1 Writer and Reader design](https://ziglang.org/download/0.15.1/release-notes.html#writergate)
- [Zig 0.16.0 I/O as an Interface](https://ziglang.org/download/0.16.0/release-notes.html#io-as-an-interface)
- [Zig's New Async I/O](https://andrewkelley.me/post/zig-new-async-io-text-version.html)
